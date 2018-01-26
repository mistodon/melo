pub mod data;
pub mod error;


use std::borrow::Cow;
use std::iter::Peekable;
use std::slice::Iter;

use lexing::data::*;
use lexing::data::Token::*;
use trust::Trust;
use notes;


use self::data::*;
use self::error::{ ParsingError, ErrorType };


type TokenStream<'a> = Peekable<Iter<'a, MetaToken<'a>>>;


fn error_swizzle<T, E>(results: Vec<Result<T, E>>) -> Result<Vec<T>, Vec<E>>
where
    T: ::std::fmt::Debug,
    E: ::std::fmt::Debug,
{
    let any_errors = results.iter().any(Result::is_err);

    if any_errors
    {
        Err(results.into_iter().filter_map(Result::err).collect())
    }
    else
    {
        Ok(results.into_iter().map(Result::unwrap).collect())
    }
}


pub fn parse<'a>(tokens: &'a [MetaToken<'a>]) -> Result<ParseTree<'a>, ParsingError>
{
    assert_eq!(tokens.last().map(|meta| meta.token), Some(EOF), "Missing EOF from token stream.");

    let mut stream = tokens.iter().peekable();

    let pieces = match stream.peek().trust().token
    {
        Piece => {
            let mut piece_results = Vec::new();

            while stream.peek().trust().token != EOF
            {
                piece_results.push(parse_piece(&mut stream));
            }

            error_swizzle(piece_results)?
        }
        _ => vec![parse_piece_from_body(&mut stream)?]
    };

    match stream.next().trust()
    {
        meta if meta.token == EOF => Ok(ParseTree { pieces }),
        meta => Err(ParsingError::unexpected(meta, "after `piece`", "end of file".to_owned())),
    }
}


fn expect_token(
    stream: &mut TokenStream,
    token: Token,
    context: &'static str)-> Result<(), ParsingError>
{
    let meta = *stream.peek().trust();

    let result = match meta.token
    {
        EOF => return Err(ParsingError::eof(meta, context, token.readable_type().to_owned())),
        found if found == token => Ok(()),
        _ => Err(ParsingError::unexpected(meta, context, token.readable_type().to_owned()))
    };

    stream.next();

    result
}

fn skip_token(stream: &mut TokenStream, token: Token) -> bool
{
    if stream.peek().trust().token == token
    {
        stream.next();
        true
    }
    else
    {
        false
    }
}

fn poison_scope(stream: &mut TokenStream, open_delim: Token, close_delim: Token)
{
    let mut nest = 1;

    while nest > 0
    {
        match stream.peek().trust().token
        {
            EOF => break,
            t if t == open_delim => nest += 1,
            t if t == close_delim => nest -= 1,
            _ => (),
        }

        stream.next();
    }
}


fn parse_piece<'a>(stream: &mut TokenStream<'a>) -> Result<PieceNode<'a>, ParsingError>
{
    expect_token(stream, Piece, "in top-level of file")?;
    expect_token(stream, LeftBrace, "at `piece`")?;

    let piece_node = parse_piece_from_body(stream);
    if piece_node.is_err()
    {
        poison_scope(stream, LeftBrace, RightBrace);
    }
    let piece_node = piece_node?;

    expect_token(stream, RightBrace, "after `piece`")?;

    Ok(piece_node)
}

fn parse_piece_from_body<'a>(stream: &mut TokenStream<'a>) -> Result<PieceNode<'a>, ParsingError>
{
    let mut piece_node = PieceNode::default();

    let mut voice_results = Vec::new();
    let mut play_results = Vec::new();

    loop
    {
        let meta = *stream.peek().trust();
        match meta.token
        {
            EOF => break,
            BlankLine => { stream.next(); () },
            RightBrace => break,
            Voice => {
                let voice = parse_voice(stream);
                if voice.is_err()
                {
                    poison_scope(stream, LeftBrace, RightBrace);
                }
                voice_results.push(voice);
            }
            Play => {
                let play = parse_play(stream);
                if play.is_err()
                {
                    poison_scope(stream, LeftBrace, RightBrace);
                }
                play_results.push(play);
            }
            _ => {
                let attribute_key = parse_attribute_key(stream, "in `piece`")?;
                let (line, col) = (meta.line, meta.col);
                match attribute_key
                {
                    Key("title") => piece_node.title = Some(try_parse_name(stream, "after `title:`")?),
                    Key("composer") => piece_node.composer = Some(try_parse_name(stream, "after `composer:`")?),
                    Key("tempo") => piece_node.tempo = Some(try_parse_num(stream, "after `tempo:`")? as u64),
                    Key("beats") => piece_node.beats = Some(try_parse_num(stream, "after `beats:`")? as u64),
                    Key(key) => return Err(ParsingError { line, col, error: ErrorType::InvalidAttribute { attribute: key.to_owned(), structure: "piece" } }),
                    Ident(key) => return Err(ParsingError { line, col, error: ErrorType::InvalidAttribute { attribute: key.to_owned(), structure: "piece" } }),
                    _ => unreachable!()
                }

                let keep_finding_attributes = skip_token(stream, Comma);
                if !keep_finding_attributes
                {
                    break
                }
            }
        }
    }

    let voice_nodes = error_swizzle(voice_results)?;
    let play_nodes = error_swizzle(play_results)?;

    piece_node.voices = voice_nodes;
    piece_node.plays = play_nodes;

    Ok(piece_node)
}

fn parse_attribute_key<'a>(
    stream: &mut TokenStream<'a>,
    context: &'static str) -> Result<Token<'a>, ParsingError>
{
    let meta = stream.next().trust();
    match meta.token
    {
        EOF => Err(ParsingError::eof(meta, context, format!("{}", "an attribute key".to_owned()))),
        Key(_) | Ident(_) => Ok(meta.token),
        _ => Err(ParsingError::unexpected( meta, context, "an attribute key".to_owned()))
    }
}

fn try_parse_name<'a>(
    stream: &mut TokenStream<'a>,
    context: &'static str) -> Result<&'a str, ParsingError>
{
    let meta = *stream.peek().trust();

    match meta.token
    {
        EOF => Err(ParsingError::eof(meta, context, "a name".to_owned())),
        Ident(s) => {
            stream.next();
            Ok(s)
        }
        Str(s) => {
            stream.next();
            Ok(s)
        }
        _ => Err(ParsingError::unexpected(meta, context, "a name".to_owned())),
    }
}

fn try_parse_num<'a>(
    stream: &mut TokenStream,
    context: &'static str) -> Result<i64, ParsingError>
{
    let meta = *stream.peek().trust();

    match meta.token
    {
        EOF => Err(ParsingError::eof(meta, context, "a number".to_owned())),
        Num(n) => {
            stream.next();
            Ok(n)
        }
        _ => Err(ParsingError::unexpected(meta, context, "a number".to_owned())),
    }
}

fn parse_voice<'a>(stream: &mut TokenStream<'a>) -> Result<VoiceNode<'a>, ParsingError>
{
    expect_token(stream, Voice, "in `piece`")?;

    let name = try_parse_name(stream, "in `voice`")?;
    let mut voice_node = VoiceNode { name, .. Default::default() };

    expect_token(stream, LeftBrace, "at `voice`")?;

    loop
    {
        if skip_token(stream, RightBrace)
        {
            break
        }

        let attribute_key = parse_attribute_key(stream, "in `voice`")?;
        let meta = *stream.peek().trust();
        let (line, col) = (meta.line, meta.col);

        match attribute_key
        {
            Ident("drums") => {
                voice_node.channel = Some(10);
                voice_node.octave = Some(-2);
            }
            Key("channel") => voice_node.channel = Some(try_parse_num(stream, "after `channel:`")? as u8),
            Key("program") => voice_node.program = Some(try_parse_num(stream, "after `program:`")? as u8),
            Key("octave") => voice_node.octave = Some(try_parse_num(stream, "after `octave:`")? as i8),
            Key("volume") => voice_node.volume = Some(try_parse_num(stream, "after `volume:`")? as u8),
            Key(key) => return Err(ParsingError { line, col, error: ErrorType::InvalidAttribute { attribute: key.to_owned(), structure: "voice" } }),
            Ident(key) => return Err(ParsingError { line, col, error: ErrorType::InvalidAttribute { attribute: key.to_owned(), structure: "voice" } }),
            _ => unreachable!()
        }

        if !skip_token(stream, Comma)
        {
            expect_token(stream, RightBrace, "after `voice`")?;
            break
        }
    }

    Ok(voice_node)
}

fn parse_play<'a>(stream: &mut TokenStream<'a>) -> Result<PlayNode<'a>, ParsingError>
{
    expect_token(stream, Play, "in `piece`")?;

    let voice = try_parse_name(stream, "in `play`").ok();
    let mut play_node = PlayNode { voice, .. Default::default() };

    let mut anonymous_stave_count = 0;
    let mut allow_new_staves = true;

    expect_token(stream, LeftBrace, "at `play`")?;

    loop
    {
        if skip_token(stream, RightBrace)
        {
            break
        }

        let meta = stream.next().trust();

        match meta.token
        {
            EOF => return Err(ParsingError::eof(meta, "in `play`", "a stave prefix".to_owned())),
            BlankLine => {
                let already_have_some_staves = !play_node.staves.is_empty();
                if already_have_some_staves
                {
                    allow_new_staves = false;
                    anonymous_stave_count = 0;
                }
            }
            Key(raw_prefix) => {

                expect_token(stream, Barline, "after stave prefix")?;

                let prefix = match raw_prefix
                {
                    "" => {
                        let anonymous_prefix = format!("V{}", anonymous_stave_count);
                        anonymous_stave_count += 1;
                        Cow::Owned(anonymous_prefix)
                    }
                    prefix => Cow::Borrowed(prefix)
                };

                let stave = {
                    let existing_stave = play_node.staves.iter()
                        .enumerate()
                        .find(|&(_, stave)| stave.prefix == prefix)
                        .map(|(i, _)| i);

                    let stave_index = existing_stave.unwrap_or(play_node.staves.len());

                    if existing_stave.is_none()
                    {
                        if allow_new_staves
                        {
                            play_node.staves.push(StaveNode { prefix, ..  Default::default() });
                        }
                        else
                        {
                            let (line, col) = (meta.line, meta.col);
                            return Err(
                                ParsingError
                                {
                                    line,
                                    col,
                                    error: ErrorType::UndeclaredStave
                                    {
                                        stave_prefix: raw_prefix.to_owned(),
                                    }
                                })
                        }
                    }

                    &mut play_node.staves[stave_index]
                };


                let stave_note = notes::note_to_midi(&stave.prefix);
                let mut bar = BarNode::default();

                loop
                {
                    let mut bar_full = false;
                    let mut stave_full = false;

                    let meta = *stream.peek().trust();
                    let (line, col) = (meta.line, meta.col);

                    match meta.token
                    {
                        EOF => return Err(ParsingError::eof(meta, "in stave", "stave contents".to_owned())),
                        Rest => bar.notes.push(NoteNode::Rest { length: 1 }),
                        Hit => {
                            let midi = stave_note.ok_or_else(|| ParsingError { line, col, error: ErrorType::InvalidHit { stave_prefix: raw_prefix.to_owned() }})?;
                            bar.notes.push(NoteNode::Note { midi, length: 1 });
                        }
                        Note(note) => {
                            let midi = notes::note_to_midi(note)
                                .ok_or_else(|| ParsingError { line, col, error: ErrorType::InvalidNote { note: note.to_owned() } })?;
                            bar.notes.push(NoteNode::Note { midi, length: 1 });
                        }
                        ExtendNote => {
                            bar.notes.push(NoteNode::Extension { length: 1 });
                        }
                        Num(num) => {
                            if num <= 0 || num >= 255
                            {
                                return Err(ParsingError { line, col, error: ErrorType::InvalidLength { length: num } })
                            }

                            let previous_note = bar.notes.last_mut().ok_or(ParsingError { line, col, error: ErrorType::UnexpectedLength { length: num } })?;

                            match previous_note
                            {
                                &mut NoteNode::Rest { ref mut length } => *length = num as u8,
                                &mut NoteNode::Extension { ref mut length } => *length = num as u8,
                                &mut NoteNode::Note { ref mut length, .. } => *length = num as u8,
                            }
                        }
                        Barline => bar_full = true,
                        Key(_) | BlankLine | RightBrace => stave_full = true,
                        _ => return Err(
                            ParsingError::unexpected(
                                meta, "in stave", "stave contents".to_owned())),
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
            }
            _ => return Err(
                ParsingError::unexpected(
                    meta, "in `play`", "a stave prefix".to_owned())),
        }
    }

    Ok(play_node)
}


#[cfg(test)]
mod tests
{
    use super::*;
    use test_helpers::stave;


    fn parsetest(mut tokens: Vec<Token>, expected: PieceNode)
    {
        tokens.push(EOF);
        let meta_tokens = tokens.into_iter().map(|token| MetaToken { token, span: Span(0, ""), line: 0, col: 0 })
            .collect::<Vec<MetaToken>>();
        let result = parse(&meta_tokens).unwrap();
        assert_eq!(result.pieces.len(), 1);
        assert_eq!(result.pieces[0], expected);
    }

    fn parsefailtest(mut tokens: Vec<Token>)
    {
        tokens.push(EOF);
        let meta_tokens = tokens.into_iter().map(|token| MetaToken { token, span: Span(0, ""), line: 0, col: 0 })
            .collect::<Vec<MetaToken>>();
        assert!(parse(&meta_tokens).is_err());
    }

    fn multiparsetest(mut tokens: Vec<Token>, expected: Vec<PieceNode>)
    {
        tokens.push(EOF);
        let meta_tokens = tokens.into_iter().map(|token| MetaToken { token, span: Span(0, ""), line: 0, col: 0 })
            .collect::<Vec<MetaToken>>();
        let result = parse(&meta_tokens).unwrap();
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
                Key("composer"), Ident("Claire"), Comma,
                Key("beats"), Num(5), Comma,
                Key("tempo"), Num(123)
            ],

            PieceNode {
                title: Some("The Title of the Piece"),
                composer: Some("Claire"),
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
                Key("volume"),
                Num(99), Comma,
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
                        volume: Some(99),
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
            });
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
            });
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
                        staves: vec![stave("V0", vec![vec![NoteNode::Rest { length: 1 }]]) ],
                        .. Default::default()
                    }
                ],
                .. Default::default()
            });
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
                        staves: vec![stave("V0", vec![vec![NoteNode::Rest { length: 1 }]]) ],
                        .. Default::default()
                    }
                ],
                .. Default::default()
            });
    }

    #[test]
    fn parse_play_node_with_two_staves()
    {
        parsetest(
            vec![Play, LeftBrace, Key("C"), Barline, Rest, Key("D"), Barline, Rest, RightBrace],
            PieceNode
            {
                plays: vec![
                    PlayNode
                    {
                        staves: vec![
                            stave("C", vec![vec![NoteNode::Rest { length: 1 }]]),
                            stave("D", vec![vec![NoteNode::Rest { length: 1 }]])],
                        .. Default::default()
                    }
                ],
                .. Default::default()
            });
    }

    #[test]
    fn parse_play_node_with_percussive_notes()
    {
        parsetest(
            vec![Play, LeftBrace, Key("C"), Barline, Hit, RightBrace],
            PieceNode
            {
                plays: vec![
                    PlayNode
                    {
                        staves: vec![stave("C", vec![vec![NoteNode::Note { midi: 60, length: 1 }]])],
                        .. Default::default()
                    }
                ],
                .. Default::default()
            });
    }

    #[test]
    fn parse_play_node_with_melody_notes()
    {
        parsetest(
            vec![Play, LeftBrace, Key(""), Barline, Note("C"), Note("D"), RightBrace],
            PieceNode
            {
                plays: vec![
                    PlayNode
                    {
                        staves: vec![stave("V0", vec![vec![NoteNode::Note { midi: 60, length: 1 }, NoteNode::Note { midi: 62, length: 1 }]])],
                        .. Default::default()
                    }
                ],
                .. Default::default()
            });
    }

    #[test]
    fn parse_stave_split_over_multiple_lines()
    {
        parsetest(
            vec![Play, LeftBrace, Key("C"), Barline, Hit, Key("C"), Barline, Hit, RightBrace],
            PieceNode
            {
                plays: vec![
                    PlayNode
                    {
                        staves: vec![stave("C", vec![vec![NoteNode::Note { midi: 60, length: 1 }], vec![NoteNode::Note { midi: 60, length: 1 }]])],
                        .. Default::default()
                    }
                ],
                .. Default::default()
            });
    }

    #[test]
    fn parse_multiple_concurrent_melody_lines()
    {
        parsetest(
            vec![Play, LeftBrace, Key(""), Barline, Note("C"), Key(""), Barline, Note("G"), RightBrace],
            PieceNode
            {
                plays: vec![
                    PlayNode
                    {
                        staves: vec![
                            stave("V0", vec![vec![NoteNode::Note { midi: 60, length: 1 }]]),
                            stave("V1", vec![vec![NoteNode::Note { midi: 67, length: 1 }]]),
                        ],
                        .. Default::default()
                    }
                ],
                .. Default::default()
            });
    }

    #[test]
    fn parse_multiple_concurrent_melody_lines_broken_up_by_blank_line()
    {
        parsetest(
            vec![
                Play,
                LeftBrace,
                Key(""),
                Barline,
                Note("C"),
                Key(""),
                Barline,
                Note("G"),
                BlankLine,

                Key(""),
                Barline,
                Note("G"),
                Key(""),
                Barline,
                Note("d"),
                RightBrace
            ],
            PieceNode
            {
                plays: vec![
                    PlayNode
                    {
                        staves: vec![
                            stave("V0", vec![vec![NoteNode::Note { midi: 60, length: 1 }], vec![NoteNode::Note { midi: 67, length: 1 }]]),
                            stave("V1", vec![vec![NoteNode::Note { midi: 67, length: 1 }], vec![NoteNode::Note { midi: 74, length: 1 }]]),
                        ],
                        .. Default::default()
                    }
                ],
                .. Default::default()
            });
    }

    #[test]
    fn fail_parsing_too_many_staves_after_blank_line()
    {
        parsefailtest(
            vec![
                Play,
                LeftBrace,
                Key(""),
                Barline,
                Note("C"),
                BlankLine,

                Key(""),
                Barline,
                Note("G"),
                Key(""),
                Barline,
                Note("d"),
                RightBrace
            ]);
    }

    #[test]
    fn fail_when_hit_notes_are_encountered_in_incompatible_staves()
    {
        parsefailtest(vec![Play, LeftBrace, Key(""), Barline, Hit, RightBrace]);
    }

    #[test]
    fn parse_note_with_length()
    {
        parsetest(vec![Play, LeftBrace, Key(""), Barline, Note("C"), Num(4), RightBrace],
            PieceNode
            {
                plays: vec![
                    PlayNode
                    {
                        staves: vec![
                            stave("V0", vec![vec![NoteNode::Note { midi: 60, length: 4 }]]),
                        ],
                        .. Default::default()
                    }
                ],
                .. Default::default()
            })
    }

    #[test]
    fn parse_rest_with_length()
    {
        parsetest(vec![Play, LeftBrace, Key(""), Barline, Rest, Num(4), RightBrace],
            PieceNode
            {
                plays: vec![
                    PlayNode
                    {
                        staves: vec![
                            stave("V0", vec![vec![NoteNode::Rest { length: 4 }]]),
                        ],
                        .. Default::default()
                    }
                ],
                .. Default::default()
            })
    }

    #[test]
    fn fail_on_unexpected_length()
    {
        parsefailtest(vec![Play, LeftBrace, Key(""), Barline, Num(4), RightBrace]);
    }

    #[test]
    fn fail_on_overflowed_length()
    {
        parsefailtest(vec![Play, LeftBrace, Key(""), Barline, Rest, Num(300), RightBrace]);
    }

    #[test]
    fn fail_on_underflowed_length()
    {
        parsefailtest(vec![Play, LeftBrace, Key(""), Barline, Rest, Num(0), RightBrace]);
    }
}

