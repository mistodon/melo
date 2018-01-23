use regex::{ Regex };

use trust::Trust;


#[derive(Debug, PartialEq, Eq)]
pub struct MetaToken<'a>
{
    pub token: Token<'a>,
    pub span: Span<'a>,
    pub line_col: (usize, usize),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Span<'a>(pub usize, pub &'a str);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Token<'a>
{
    Piece,
    Voice,
    Section,
    Part,
    Play,
    LeftBrace,
    RightBrace,
    Comma,
    BlankLine,
    Num(i64),
    Key(&'a str),
    Ident(&'a str),
    Str(&'a str),

    Barline,
    Rest,
    Hit,
    Ditto,
    Repeat,
    Length(u64),
    Note(&'a str),
    PlayPart(&'a str),
}


#[derive(Debug, Fail, PartialEq, Eq)]
pub enum LexingError
{
    #[fail(display = "error: Unexpected character '{}' in {} at {}:{}.", text, context, line, col)]
    UnexpectedCharacter
    {
        text: String,
        context: &'static str,
        line: usize,
        col: usize,
    }
}


fn line_col_at(source: &str, position: usize) -> (usize, usize)
{
    let mut bytes = 0;
    for (line_no, line) in source.lines().enumerate()
    {
        if position >= bytes && position < bytes + line.len()
        {
            let col = position - bytes;
            return (line_no + 1, col + 1)
        }
        bytes += line.len();
    }
    (0, source.len())
}


pub fn lex<'a>(source: &'a str) -> Result<Vec<MetaToken<'a>>, LexingError>
{
    use self::Token::*;

    let mut tokens = Vec::new();

    const CAPTURE_PRIORITIES: &[&str] = &[
        "key", "ident", "string", "number", "delim", "staveline",
        "blank", "whitespace", "comment", "error"
    ];

    const STAVE_CAPTURE_PRIORITIES: &[&str] = &[
        "note", "part", "barline", "symbol", "length", "whitespace", "comment", "error"
    ];

    for capture in STRUCTURE_REGEX.captures_iter(source)
    {
        let mut group = None;

        for group_name in CAPTURE_PRIORITIES
        {
            if let Some(m) = capture.name(group_name)
            {
                group = group.or(Some((group_name, m)));
            }
        }

        let (&group_name, m) = group.trust();
        let text = m.as_str();
        let text_len = text.len();
        let span = Span(m.start(), text);
        let line_col = line_col_at(source, m.start());

        match group_name
        {
            "key" => tokens.push(MetaToken { token: Key(text[..(text_len-1)].trim()), span, line_col }),
            "ident" => {
                let token = match text
                {
                    "piece" => Piece,
                    "voice" => Voice,
                    "part" => Part,
                    "section" => Section,
                    "play" => Play,
                    s => Ident(s),
                };
                tokens.push(MetaToken { token, span, line_col });
            }
            "string" => tokens.push(MetaToken { token: Str(&text[1..(text_len-1)]), span, line_col }),
            "number" => {
                let number = text.parse().trust();
                tokens.push(MetaToken { token: Num(number), span, line_col });
            }
            "delim" => {
                let token = match text
                {
                    "{" => LeftBrace,
                    "}" => RightBrace,
                    "," => Comma,
                    _ => unreachable!()
                };
                tokens.push(MetaToken { token, span, line_col });
            }
            "staveline" => {
                let start = span.0;

                for capture in MUSIC_REGEX.captures_iter(text)
                {
                    let mut group = None;

                    for group_name in STAVE_CAPTURE_PRIORITIES
                    {
                        if let Some(m) = capture.name(group_name)
                        {
                            group = group.or(Some((group_name, m)));
                        }
                    }

                    let (&group_name, m) = group.trust();
                    let text = m.as_str();
                    let start = start + m.start();
                    let span = Span(start, text);
                    let line_col = line_col_at(source, start);

                    match group_name
                    {
                        "note" => tokens.push(MetaToken { token: Note(text), span, line_col }),
                        "part" => tokens.push(MetaToken { token: PlayPart(&text[1..]), span, line_col }),
                        "barline" => tokens.push(MetaToken { token: Barline, span, line_col }),
                        "symbol" => {
                            let token = match text
                            {
                                "-" => Rest,
                                "x" => Hit,
                                "\"" => Ditto,
                                "%" => Repeat,
                                _ => unreachable!()
                            };
                            tokens.push(MetaToken { token, span, line_col });
                        }
                        "length" => {
                            let size = match text
                            {
                                "." => 2,
                                s if s.as_bytes()[1] == b'.' => s.len() + 1,
                                s => s[1..].parse().trust()
                            };
                            tokens.push(MetaToken { token: Length(size as u64), span, line_col });
                        }
                        "whitespace" | "comment" => (),
                        "error" => {
                            return Err(
                                LexingError::UnexpectedCharacter
                                {
                                    text: text.to_owned(),
                                    context: "stave",
                                    line: line_col.0,
                                    col: line_col.1,
                                });
                        }
                        _ => unreachable!()
                    }
                }
            }
            "blank" => tokens.push(MetaToken { token: BlankLine, span, line_col }),
            "whitespace" | "comment" => (),
            "error" => {
                return Err(
                    LexingError::UnexpectedCharacter
                    {
                        text: text.to_owned(),
                        context: "file",
                        line: line_col.0,
                        col: line_col.1,
                    })
            }
            _ => unreachable!()
        }
    }

    Ok(tokens)
}


lazy_static!
{
    static ref STRUCTURE_REGEX: Regex = Regex::new("\
        (?P<key>([a-zA-Z_][a-zA-Z0-9_^,'=\\-]*\\s*|:)?:)|\
        (?P<ident>[a-zA-Z_][a-zA-Z0-9_]*)|\
        (?P<string>\"((\\\\\")|[^\"])*\")|\
        (?P<number>[+\\-]?\\d+)|\
        (?P<delim>[{},])|\
        (?P<staveline>\\|([^;}\n]*))|\
        (?P<comment>//[^\n]*)|\
        (?P<blank>\n\\s*\n)|\
        (?P<whitespace>(\\s+|;))|\
        (?P<error>.)\
        ").trust();
}

lazy_static!
{
    static ref MUSIC_REGEX: Regex = Regex::new("\
        (?P<note>[a-gA-G][=_\\^]*[,']*)|\
        (?P<part>\\*[a-zA-Z_][a-zA-Z0-9_]*)|\
        (?P<symbol>[\\-x\"%])|\
        (?P<length>\\.(\\d+|\\.*))|\
        (?P<barline>\\|)|\
        (?P<comment>//.*)|\
        (?P<whitespace>(\\s|;)+)|\
        (?P<error>.)\
        ").trust();
}


#[cfg(test)]
mod tests
{
    use super::*;
    use super::Token::*;


    fn lextest(source: &str, result: Vec<Token>)
    {
        assert_eq!(
            lex(source).unwrap().into_iter().map(|meta| meta.token).collect::<Vec<_>>(),
            result
        );
    }

    #[test]
    fn empty_file()
    {
        lextest("", vec![]);
    }

    #[test]
    fn invalid_tokens()
    {
        assert_eq!(
            lex("@").unwrap_err(),
            LexingError::UnexpectedCharacter
            {
                text: "@".to_owned(),
                context: "file",
                line: 1,
                col: 1,
            });
    }

    #[test]
    fn invalid_tokens_in_stave()
    {
        assert_eq!(
            lex("   :|{}|").unwrap_err(),
            LexingError::UnexpectedCharacter
            {
                text: "{".to_owned(),
                context: "stave",
                line: 1,
                col: 6,
            });
    }

    #[test]
    fn lex_piece()
    {
        lextest("piece{}", vec![Piece, LeftBrace, RightBrace])
    }

    #[test]
    fn lex_section()
    {
        lextest("section {}", vec![Section, LeftBrace, RightBrace])
    }

    #[test]
    fn lex_voice()
    {
        lextest("voice {}", vec![Voice, LeftBrace, RightBrace])
    }

    #[test]
    fn lex_part()
    {
        lextest("part {}", vec![Part, LeftBrace, RightBrace])
    }

    #[test]
    fn lex_play()
    {
        lextest("play {}", vec![Play, LeftBrace, RightBrace])
    }

    #[test]
    fn comments_ignored()
    {
        lextest(
            "// This is a piece\npiece{}",
            vec![Piece, LeftBrace, RightBrace])
    }

    #[test]
    fn insignificant_whitespace_ignored()
    {
        lextest("piece {\n}", vec![Piece, LeftBrace, RightBrace])
    }

    #[test]
    fn lex_name()
    {
        lextest("piece Heroine {}", vec![
                Piece,
                Ident("Heroine"),
                LeftBrace,
                RightBrace,
        ]);
    }

    #[test]
    fn lex_quoted_name()
    {
        lextest(r#"piece "Lust for Life" {}"#, vec![
                Piece,
                Str("Lust for Life"),
                LeftBrace,
                RightBrace,
        ]);
    }

    #[test]
    fn lex_quoted_name_with_quotes_in_it()
    {
        lextest(r#"piece "\"Lust\" for \"Life\"" {}"#, vec![
                Piece,
                Str(r#"\"Lust\" for \"Life\""#),
                LeftBrace,
                RightBrace,
        ]);
    }

    #[test]
    fn lex_empty_key()
    {
        lextest("{ : A }", vec![
                LeftBrace,
                Key(""),
                Ident("A"),
                RightBrace,
        ]);
    }

    #[test]
    fn lex_all_staves_key()
    {
        lextest(":: | -", vec![
                Key(":"),
                Barline,
                Rest,
        ]);
    }

    #[test]
    fn lex_field_in_block()
    {
        lextest(r#"piece LFL { title: "Party Girl" }"#, vec![
                Piece,
                Ident("LFL"),
                LeftBrace,
                Key("title"),
                Str("Party Girl"),
                RightBrace,
        ]);
    }

    #[test]
    fn lex_ridiculous_field_name()
    {
        lextest(r#"piece LFL { F^_=,,''   : "Party Girl" }"#, vec![
                Piece,
                Ident("LFL"),
                LeftBrace,
                Key("F^_=,,''"),
                Str("Party Girl"),
                RightBrace,
        ]);
    }

    #[test]
    fn lex_multiple_fields()
    {
        lextest("{ drums, name: drum_voice }", vec![
                LeftBrace,
                Ident("drums"),
                Comma,
                Key("name"),
                Ident("drum_voice"),
                RightBrace,
        ]);
    }

    #[test]
    fn lex_numbers()
    {
        lextest("{ channel: 0, octave: -1 }", vec![
                LeftBrace,
                Key("channel"),
                Num(0),
                Comma,
                Key("octave"),
                Num(-1),
                RightBrace,
        ]);
    }

    #[test]
    fn lex_blank_lines()
    {
        lextest("{ a: 0,\n\nb: 1 }", vec![
                LeftBrace,
                Key("a"),
                Num(0),
                Comma,
                BlankLine,
                Key("b"),
                Num(1),
                RightBrace,
        ]);
    }

    #[test]
    fn lex_note()
    {
        lextest(": | A", vec![
                Key(""),
                Barline,
                Note("A"),
        ]);
    }

    #[test]
    fn lex_complex_notes()
    {
        lextest(": | B^,,c_''d=", vec![
                Key(""),
                Barline,
                Note("B^,,"),
                Note("c_''"),
                Note("d="),
        ]);
    }

    #[test]
    fn lex_note_length()
    {
        lextest(": | A... B.4 | C.", vec![
                Key(""),
                Barline,
                Note("A"),
                Length(4),
                Note("B"),
                Length(4),
                Barline,
                Note("C"),
                Length(2),
        ]);
    }

    #[test]
    fn lex_symbols()
    {
        lextest("C : | x - x-| % | \" |", vec![
                Key("C"),
                Barline,
                Hit,
                Rest,
                Hit,
                Rest,
                Barline,
                Repeat,
                Barline,
                Ditto,
                Barline,
        ]);
    }

    #[test]
    fn lex_play_part()
    {
        lextest(":| *Theme", vec![
                Key(""),
                Barline,
                PlayPart("Theme"),
        ]);
    }

    #[test]
    fn semicolon_can_break_stave_within_one_line()
    {
        lextest("A:|x;B:|x;", vec![
                Key("A"),
                Barline,
                Hit,
                Key("B"),
                Barline,
                Hit,
        ]);
    }

    #[test]
    fn right_brace_can_break_stave_within_one_line()
    {
        lextest("{ A:|x }", vec![
                LeftBrace,
                Key("A"),
                Barline,
                Hit,
                RightBrace,
        ]);
    }
}

