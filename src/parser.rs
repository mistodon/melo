use std::borrow::Cow;
use std::iter::Peekable;
use std::slice::Iter;

use lexer::{ Token, MetaToken };
use lexer::Token::*;


type TokenStream<'a> = Peekable<Iter<'a, MetaToken<'a>>>;


#[derive(Debug, PartialEq, Eq)]
pub struct ParseTree<'a>
{
    pub pieces: Vec<PieceNode<'a>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PieceNode<'a>
{
    pub title: Option<&'a str>,
    pub composer: Option<&'a str>,
    pub tempo: Option<u64>,
    pub beats: Option<u64>,

    pub voices: Vec<VoiceNode<'a>>,
    pub plays: Vec<PlayNode<'a>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct VoiceNode<'a>
{
    pub name: &'a str,
    pub program: Option<u8>,
    pub channel: Option<u8>,
    pub octave: Option<i8>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PlayNode<'a>
{
    pub voice: Option<&'a str>,
    pub staves: Vec<StaveNode<'a>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct StaveNode<'a>
{
    pub prefix: Cow<'a, str>,
    pub bars: Vec<BarNode>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct BarNode
{
    pub notes: Vec<NoteNode>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NoteNode
{
    Rest
}


pub fn parse<'a>(tokens: &'a [MetaToken<'a>]) -> ParseTree<'a>
{
    let mut pieces = Vec::new();
    let mut stream = tokens.iter().peekable();

    match stream.peek()
    {
        Some(&&MetaToken { token: Piece, .. }) => {
            while stream.peek().is_some()
            {
                pieces.push(parse_piece(&mut stream));
            }
        }
        _ => {
            pieces.push(parse_piece_from_body(&mut stream));
        }
    }

    assert!(stream.next().is_none());

    ParseTree { pieces }
}


fn expect_token(stream: &mut TokenStream, token: Token)
{
    let found = stream.next().map(|meta| meta.token);
    if found != Some(token)
    {
        panic!("error: Expected {:?} token, but found {:?}", Some(token), found)
    }
}

fn skip_token(stream: &mut TokenStream, token: Token) -> bool
{
    if stream.peek().map(|meta| meta.token) == Some(token)
    {
        stream.next();
        true
    }
    else
    {
        false
    }
}


fn parse_piece<'a>(stream: &mut TokenStream<'a>) -> PieceNode<'a>
{
    expect_token(stream, Piece);
    expect_token(stream, LeftBrace);

    let piece_node = parse_piece_from_body(stream);

    expect_token(stream, RightBrace);

    piece_node
}

fn parse_piece_from_body<'a>(stream: &mut TokenStream<'a>) -> PieceNode<'a>
{
    let mut piece_node = PieceNode::default();

    loop
    {
        match stream.peek()
        {
            Some(&&MetaToken { token: RightBrace, .. }) => break,
            None => break,
            Some(&&MetaToken { token: Voice, ..}) => piece_node.voices.push(parse_voice(stream)),
            Some(&&MetaToken { token: Play, ..}) => piece_node.plays.push(parse_play(stream)),
            _ => {
                let attribute_key = parse_attribute_key(stream);
                match attribute_key
                {
                    Key("title") => piece_node.title = Some(try_parse_name(stream).unwrap()),
                    Key("composer") => piece_node.composer = Some(try_parse_name(stream).unwrap()),
                    Key("tempo") => piece_node.tempo = Some(try_parse_num(stream).unwrap() as u64),
                    Key("beats") => piece_node.beats = Some(try_parse_num(stream).unwrap() as u64),
                    attr => panic!("error: Invalid attribute for `piece`: {:?}", attr)
                }

                let keep_finding_attributes = skip_token(stream, Comma);
                if !keep_finding_attributes
                {
                    break
                }
            }
        }
    }

    piece_node
}

fn parse_attribute_key<'a>(stream: &mut TokenStream<'a>) -> Token<'a>
{
    match stream.next()
    {
        Some(meta) => {
            match meta.token
            {
                Key(_) | Ident(_) => meta.token,
                unexpected => panic!("error: Unexpected token in `piece`, expected attribute but found {:?}", unexpected)
            }
        }
        None => panic!("error: Unexpected end of file, expected attribute")
    }
}

fn try_parse_name<'a>(stream: &mut TokenStream<'a>) -> Option<&'a str>
{
    match stream.peek().map(|meta| meta.token)
    {
        Some(Ident(s)) => {
            stream.next();
            Some(s)
        }
        Some(Str(s)) => {
            stream.next();
            Some(s)
        }
        _ => None
    }
}

fn try_parse_num<'a>(stream: &mut TokenStream) -> Option<i64>
{
    match stream.peek().map(|meta| meta.token)
    {
        Some(Num(n)) => {
            stream.next();
            Some(n)
        }
        _ => None
    }
}

fn parse_voice<'a>(stream: &mut TokenStream<'a>) -> VoiceNode<'a>
{
    expect_token(stream, Voice);

    let name = try_parse_name(stream).unwrap();
    let mut voice_node = VoiceNode { name, .. Default::default() };

    expect_token(stream, LeftBrace);

    loop
    {
        if skip_token(stream, RightBrace)
        {
            break
        }

        let attribute_key = parse_attribute_key(stream);
        match attribute_key
        {
            Ident("drums") => {
                voice_node.channel = Some(10);
                voice_node.octave = Some(-2);
            }
            Key("channel") => voice_node.channel = Some(try_parse_num(stream).unwrap() as u8),
            Key("program") => voice_node.program = Some(try_parse_num(stream).unwrap() as u8),
            Key("octave") => voice_node.octave = Some(try_parse_num(stream).unwrap() as i8),
            attr => panic!("error: Invalid attribute for `voice`: {:?}", attr)
        }

        if !skip_token(stream, Comma)
        {
            expect_token(stream, RightBrace);
            break
        }
    }

    voice_node
}

fn parse_play<'a>(stream: &mut TokenStream<'a>) -> PlayNode<'a>
{
    expect_token(stream, Play);

    let voice = try_parse_name(stream);
    let mut play_node = PlayNode { voice, .. Default::default() };

    expect_token(stream, LeftBrace);

    loop
    {
        if skip_token(stream, RightBrace)
        {
            break
        }

        match stream.next()
        {
            Some(&MetaToken { token: Key(prefix), .. }) => {

                expect_token(stream, Barline);

                let mut stave = StaveNode { prefix: Cow::Borrowed(prefix), ..  Default::default() };

                let mut bar = BarNode::default();

                loop
                {
                    let mut bar_full = false;
                    let mut stave_full = false;

                    match stream.peek().map(|meta| meta.token)
                    {
                        Some(Rest) => bar.notes.push(NoteNode::Rest),
                        Some(Barline) => bar_full = true,
                        Some(Key(_)) | Some(BlankLine) | Some(RightBrace) => stave_full = true,
                        None => panic!("error: Unexpected end of file, expected notes or end of `play` block"),
                        unexpected => panic!("error: Expected stave contents, found {:?}", unexpected)
                    }

                    if bar_full || stave_full
                    {
                        if !bar.notes.is_empty()
                        {
                            let complete_bar = ::std::mem::replace(&mut bar, BarNode::default());
                            stave.bars.push(complete_bar);
                        }
                    }

                    if stave_full
                    {
                        break
                    }

                    stream.next();
                }

                play_node.staves.push(stave);
            }
            unexpected => panic!("error: Expected stave prefix, found {:?}", unexpected)
        }
    }

    play_node
}


#[cfg(test)]
mod tests
{
    use super::*;
    use lexer::Span;


    fn parsetest(tokens: Vec<Token>, expected: PieceNode)
    {
        let meta_tokens = tokens.into_iter().map(|token| MetaToken { token, span: Span(0, 0) })
            .collect::<Vec<MetaToken>>();
        let result = parse(&meta_tokens);
        assert_eq!(result.pieces.len(), 1);
        assert_eq!(result.pieces[0], expected);
    }

    fn parsefailtest(tokens: Vec<Token>)
    {
        let meta_tokens = tokens.into_iter().map(|token| MetaToken { token, span: Span(0, 0) })
            .collect::<Vec<MetaToken>>();
        parse(&meta_tokens);
    }

    fn multiparsetest(tokens: Vec<Token>, expected: Vec<PieceNode>)
    {
        let meta_tokens = tokens.into_iter().map(|token| MetaToken { token, span: Span(0, 0) })
            .collect::<Vec<MetaToken>>();
        let result = parse(&meta_tokens);
        assert_eq!(result.pieces, expected);
    }

    #[test]
    fn parse_empty_file()
    {
        parsetest(vec![], PieceNode::default());
    }

    #[test]
    fn parse_empty_piece()
    {
        parsetest(vec![Piece, LeftBrace, RightBrace], PieceNode::default());
    }

    #[test]
    #[should_panic]
    fn parse_empty_piece_with_trailing_tokens_fails()
    {
        parsetest(vec![Piece, LeftBrace, RightBrace, RightBrace], PieceNode::default());
    }

    #[test]
    fn parse_multiple_empty_pieces()
    {
        multiparsetest(
            vec![Piece, LeftBrace, RightBrace, Piece, LeftBrace, RightBrace],
            vec![PieceNode::default(), PieceNode::default()]);
    }

    #[test]
    fn parse_attributes_in_piece()
    {
        parsetest(
            vec![
                Key("title"), Str("The Title of the Piece"), Comma,
                Key("composer"), Ident("***Realname***"), Comma,
                Key("beats"), Num(5), Comma,
                Key("tempo"), Num(123)
            ],

            PieceNode {
                title: Some("The Title of the Piece"),
                composer: Some("***Realname***"),
                beats: Some(5),
                tempo: Some(123),
                .. Default::default()
            });
    }

    #[test]
    fn parse_attributes_with_trailing_comma()
    {
        parsetest(
            vec![Key("title"), Ident("Title"), Comma],
            PieceNode
            {
                title: Some("Title"),
                .. Default::default()
            });
    }

    #[test]
    #[should_panic]
    fn fail_to_parse_invalid_attributes()
    {
        parsefailtest(vec![Key("titel"), Ident("Title"), Comma]);
    }

    #[test]
    fn parse_empty_voice()
    {
        parsetest(
            vec![Voice, Ident("Drums"), LeftBrace, RightBrace],
            PieceNode
            {
                voices: vec![VoiceNode { name: "Drums", .. Default::default() }],
                .. Default::default()
            })
    }

    #[test]
    fn parse_voice_attributes()
    {
        parsetest(
            vec![
                Voice,
                Ident("Lead"),
                LeftBrace,
                Key("channel"),
                Num(1), Comma,
                Key("program"),
                Num(0), Comma,
                Key("octave"),
                Num(-2), Comma,
                RightBrace
            ],
            PieceNode
            {
                voices: vec![
                    VoiceNode
                    {
                        name: "Lead",
                        channel: Some(1),
                        program: Some(0),
                        octave: Some(-2),
                    }],
                .. Default::default()
            })
    }

    #[test]
    fn parse_valueless_attribute()
    {
        parsetest(
            vec![Voice, Ident("Drums"), LeftBrace, Ident("drums"), RightBrace],
            PieceNode
            {
                voices: vec![
                    VoiceNode
                    {
                        name: "Drums",
                        channel: Some(10),
                        octave: Some(-2),
                        .. Default::default()
                    }],
                .. Default::default()
            }
        );
    }

    #[test]
    fn parse_empty_play_node()
    {
        parsetest(
            vec![Play, Ident("Drums"), LeftBrace, RightBrace],
            PieceNode
            {
                plays: vec![
                    PlayNode
                    {
                        voice: Some("Drums"),
                        .. Default::default()
                    }
                ],
                .. Default::default()
            })
    }

    #[test]
    fn parse_empty_voiceless_play_node()
    {
        parsetest(
            vec![Play, LeftBrace, RightBrace],
            PieceNode
            {
                plays: vec![
                    PlayNode
                    {
                        voice: None,
                        .. Default::default()
                    }
                ],
                .. Default::default()
            })
    }

    #[test]
    fn parse_play_node_with_stave()
    {
        parsetest(
            vec![Play, LeftBrace, Key(""), Barline, Rest, RightBrace],
            PieceNode
            {
                plays: vec![
                    PlayNode
                    {
                        staves: vec![
                            StaveNode
                            {
                                prefix: Cow::Borrowed(""),
                                bars: vec![BarNode { notes: vec![NoteNode::Rest] }],
                            }
                        ],
                        .. Default::default()
                    }
                ],
                .. Default::default()
            })
    }

    #[test]
    fn parse_play_node_with_extra_barlines()
    {
        parsetest(
            vec![Play, LeftBrace, Key(""), Barline, Barline, Rest, Barline, Barline, RightBrace],
            PieceNode
            {
                plays: vec![
                    PlayNode
                    {
                        staves: vec![
                            StaveNode
                            {
                                prefix: Cow::Borrowed(""),
                                bars: vec![BarNode { notes: vec![NoteNode::Rest] }],
                            }
                        ],
                        .. Default::default()
                    }
                ],
                .. Default::default()
            })
    }
}

