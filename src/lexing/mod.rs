pub mod data;
pub mod error;

use self::data::*;
use self::error::{ErrorType, LexingError};

use crate::error::{SourceInfo, SourceLoc, SourceMap};
use crate::trust::Trust;
use regex::Regex;

// TODO(claire): This code assumes that a newline is a single byte
fn line_col_at(source: &str, position: usize) -> (usize, usize) {
    let mut bytes = 0;
    for (line_no, line) in source.lines().enumerate() {
        if position >= bytes && position < bytes + line.len() + 1 {
            let col = position - bytes;
            return (line_no + 1, col + 1);
        }
        bytes += line.len() + 1;
    }
    (1, source.len())
}

lazy_static! {
    static ref STRUCTURE_REGEX: Regex = Regex::new(
        "\
         (?P<keyword>part|piece|play|section|voice)|\
         (?P<key>([a-zA-Z_][a-zA-Z0-9_#,'=\\-]*\\s*|:)?:)|\
         (?P<ident>[a-zA-Z_][a-zA-Z0-9_ ]*)|\
         (?P<string>\"((\\\\\")|[^\"])*\")|\
         (?P<number>[+\\-]?\\d+)|\
         (?P<delim>[{},])|\
         (?P<staveline>\\|([^;}\n]*))|\
         (?P<comment>//[^\n]*)|\
         (?P<blank>\n\\s*\n)|\
         (?P<newline>\n)|\
         (?P<whitespace>([\t ]|;)+)|\
         (?P<error>.)\
         "
    )
    .trust();
    static ref MUSIC_REGEX: Regex = Regex::new(
        "\
         (?P<note>[a-gA-G][=_\\#]*[,']*)|\
         (?P<part>\\*[a-zA-Z_][a-zA-Z0-9_]*)|\
         (?P<symbol>[\\.\\-x\"%])|\
         (?P<number>\\d+)|\
         (?P<barline>\\|)|\
         (?P<comment>//[^\n]*)|\
         (?P<whitespace>([\t ]|;)+)|\
         (?P<error>.)\
         "
    )
    .trust();
}

#[derive(Debug)]
enum Context {
    Normal,
    InAttribute,
    InStave,
}

pub fn lex<'a>(
    source: &'a str,
    filename: Option<&str>,
) -> Result<(Vec<MetaToken<'a>>, SourceMap), LexingError> {
    use self::Token::*;

    let source_map = SourceInfo::new(source, filename);

    let mut tokens = Vec::new();

    const CAPTURE_PRIORITIES: &[&str] = &[
        "keyword",
        "key",
        "ident",
        "string",
        "number",
        "delim",
        "staveline",
        "blank",
        "newline",
        "whitespace",
        "comment",
        "error",
    ];

    const STAVE_CAPTURE_PRIORITIES: &[&str] = &[
        "note",
        "part",
        "barline",
        "symbol",
        "number",
        "whitespace",
        "comment",
        "error",
    ];

    let mut context = Context::Normal;

    for capture in STRUCTURE_REGEX.captures_iter(source) {
        let mut group = None;

        for group_name in CAPTURE_PRIORITIES {
            if let Some(m) = capture.name(group_name) {
                group = group.or_else(|| Some((group_name, m)));
            }
        }

        let (&group_name, m) = group.trust();
        let text = m.as_str();
        let text_len = text.len();
        let span = Span(m.start(), text);
        let (line, col) = line_col_at(source, m.start());
        let loc = SourceLoc {
            line,
            col,
            info: source_map.clone(),
            width: text_len,
        };

        match group_name {
            "keyword" => {
                let token = match text {
                    "piece" => Piece,
                    "voice" => Voice,
                    "part" => Part,
                    "section" => Section,
                    "play" => Play,
                    _ => unreachable!(),
                };

                tokens.push(MetaToken { token, span, loc });
            }
            "key" => {
                tokens.push(MetaToken {
                    token: Key(text[..(text_len - 1)].trim()),
                    span,
                    loc,
                });

                context = Context::InAttribute;
            }
            "ident" => {
                tokens.push(MetaToken {
                    token: Ident(text.trim()),
                    span,
                    loc,
                });
            }
            "string" => tokens.push(MetaToken {
                token: Str(&text[1..(text_len - 1)]),
                span,
                loc,
            }),
            "number" => {
                let number = text.parse().trust();
                tokens.push(MetaToken {
                    token: Num(number),
                    span,
                    loc,
                });
            }
            "delim" => {
                let token = match text {
                    "{" => LeftBrace,
                    "}" => RightBrace,
                    "," => Comma,
                    _ => unreachable!(),
                };
                tokens.push(MetaToken { token, span, loc });

                context = Context::Normal;
            }
            "staveline" => {
                context = Context::InStave;
                let start = span.0;

                for capture in MUSIC_REGEX.captures_iter(text) {
                    let mut group = None;

                    for group_name in STAVE_CAPTURE_PRIORITIES {
                        if let Some(m) = capture.name(group_name) {
                            group = group.or_else(|| Some((group_name, m)));
                        }
                    }

                    let (&group_name, m) = group.trust();
                    let text = m.as_str();
                    let start = start + m.start();
                    let span = Span(start, text);
                    let (line, col) = line_col_at(source, start);
                    let loc = SourceLoc {
                        line,
                        col,
                        info: source_map.clone(),
                        width: text.len(),
                    };

                    match group_name {
                        "note" => tokens.push(MetaToken {
                            token: Note(text),
                            span,
                            loc,
                        }),
                        "part" => tokens.push(MetaToken {
                            token: PlayPart(&text[1..]),
                            span,
                            loc,
                        }),
                        "barline" => tokens.push(MetaToken {
                            token: Barline,
                            span,
                            loc,
                        }),
                        "symbol" => {
                            let token = match text {
                                "-" => Rest,
                                "x" => Hit,
                                "\"" => Ditto,
                                "%" => RepeatBar,
                                "." => ExtendNote,
                                _ => unreachable!(),
                            };
                            tokens.push(MetaToken { token, span, loc });
                        }
                        "number" => {
                            let number = text.parse::<i64>().trust();
                            tokens.push(MetaToken {
                                token: Num(number),
                                span,
                                loc,
                            });
                        }
                        "whitespace" | "comment" => (),
                        "error" => {
                            return Err(LexingError {
                                loc,
                                error: ErrorType::UnexpectedCharacter {
                                    text: text.to_owned(),
                                    context: "stave",
                                },
                            })
                        }
                        _ => unreachable!(),
                    }
                }
            }
            "blank" => {
                match context {
                    Context::InAttribute => tokens.push(MetaToken {
                        token: Comma,
                        span,
                        loc: loc.clone(),
                    }),
                    Context::InStave => tokens.push(MetaToken {
                        token: Barline,
                        span,
                        loc: loc.clone(),
                    }),
                    _ => (),
                }

                tokens.push(MetaToken {
                    token: BlankLine,
                    span,
                    loc,
                });

                context = Context::Normal;
            }
            "newline" => {
                match context {
                    Context::InAttribute => tokens.push(MetaToken {
                        token: Comma,
                        span,
                        loc,
                    }),
                    Context::InStave => tokens.push(MetaToken {
                        token: Barline,
                        span,
                        loc,
                    }),
                    _ => (),
                }

                context = Context::Normal;
            }
            "whitespace" | "comment" => (),
            "error" => {
                return Err(LexingError {
                    loc,
                    error: ErrorType::UnexpectedCharacter {
                        text: text.to_owned(),
                        context: "file",
                    },
                })
            }
            _ => unreachable!(),
        }
    }

    let (line, col) = line_col_at(source, source.len());
    tokens.push(MetaToken {
        token: EOF,
        span: Span(source.len(), ""),
        loc: SourceLoc {
            line,
            col,
            info: source_map.clone(),
            width: 1,
        },
    });

    Ok((tokens, source_map))
}

#[cfg(test)]
mod tests {
    use super::Token::*;
    use super::*;

    fn lextest(source: &str, mut result: Vec<Token>) {
        result.push(EOF);
        assert_eq!(
            lex(source, None)
                .unwrap()
                .0
                .into_iter()
                .map(|meta| meta.token)
                .collect::<Vec<_>>(),
            result
        );
    }

    #[test]
    fn empty_file() {
        lextest("", vec![]);
    }

    #[test]
    fn invalid_tokens() {
        assert_eq!(
            lex("@", None).unwrap_err().error,
            ErrorType::UnexpectedCharacter {
                text: "@".to_owned(),
                context: "file",
            },
        );
    }

    #[test]
    fn invalid_tokens_in_stave() {
        assert_eq!(
            lex("   :|{}|", None).unwrap_err().error,
            ErrorType::UnexpectedCharacter {
                text: "{".to_owned(),
                context: "stave",
            },
        );
    }

    #[test]
    fn lex_piece() {
        lextest("piece{}", vec![Piece, LeftBrace, RightBrace])
    }

    #[test]
    fn lex_section() {
        lextest("section {}", vec![Section, LeftBrace, RightBrace])
    }

    #[test]
    fn lex_voice() {
        lextest("voice {}", vec![Voice, LeftBrace, RightBrace])
    }

    #[test]
    fn lex_part() {
        lextest("part {}", vec![Part, LeftBrace, RightBrace])
    }

    #[test]
    fn lex_play() {
        lextest("play {}", vec![Play, LeftBrace, RightBrace])
    }

    #[test]
    fn comments_ignored() {
        lextest(
            "// This is a piece\npiece{}",
            vec![Piece, LeftBrace, RightBrace],
        )
    }

    #[test]
    fn insignificant_whitespace_ignored() {
        lextest("piece {\n}", vec![Piece, LeftBrace, RightBrace])
    }

    #[test]
    fn lex_name() {
        lextest(
            "piece Heroine {}",
            vec![Piece, Ident("Heroine"), LeftBrace, RightBrace],
        );
    }

    #[test]
    fn lex_quoted_name() {
        lextest(
            r#"piece "Lust for Life" {}"#,
            vec![Piece, Str("Lust for Life"), LeftBrace, RightBrace],
        );
    }

    #[test]
    fn lex_name_with_spaces() {
        lextest(
            r#"piece Lust for Life {}"#,
            vec![Piece, Ident("Lust for Life"), LeftBrace, RightBrace],
        );
    }

    #[test]
    fn lex_quoted_name_with_quotes_in_it() {
        lextest(
            r#"piece "\"Lust\" for \"Life\"" {}"#,
            vec![
                Piece,
                Str(r#"\"Lust\" for \"Life\""#),
                LeftBrace,
                RightBrace,
            ],
        );
    }

    #[test]
    fn lex_empty_key() {
        lextest("{ : A }", vec![LeftBrace, Key(""), Ident("A"), RightBrace]);
    }

    #[test]
    fn lex_all_staves_key() {
        lextest(":: | -", vec![Key(":"), Barline, Rest]);
    }

    #[test]
    fn lex_field_in_block() {
        lextest(
            r#"piece LFL { title: "Party Girl" }"#,
            vec![
                Piece,
                Ident("LFL"),
                LeftBrace,
                Key("title"),
                Str("Party Girl"),
                RightBrace,
            ],
        );
    }

    #[test]
    fn lex_ridiculous_field_name() {
        lextest(
            r#"piece LFL { F#_=,,''   : "Party Girl" }"#,
            vec![
                Piece,
                Ident("LFL"),
                LeftBrace,
                Key("F#_=,,''"),
                Str("Party Girl"),
                RightBrace,
            ],
        );
    }

    #[test]
    fn lex_multiple_fields() {
        lextest(
            "{ drums, name: drum_voice }",
            vec![
                LeftBrace,
                Ident("drums"),
                Comma,
                Key("name"),
                Ident("drum_voice"),
                RightBrace,
            ],
        );
    }

    #[test]
    fn lex_numbers() {
        lextest(
            "{ channel: 0, octave: -1 }",
            vec![
                LeftBrace,
                Key("channel"),
                Num(0),
                Comma,
                Key("octave"),
                Num(-1),
                RightBrace,
            ],
        );
    }

    #[test]
    fn lex_blank_lines() {
        lextest(
            "{ a: 0,\n\nb: 1 }",
            vec![
                LeftBrace,
                Key("a"),
                Num(0),
                Comma,
                BlankLine,
                Key("b"),
                Num(1),
                RightBrace,
            ],
        );
    }

    #[test]
    fn lex_note() {
        lextest(": | A", vec![Key(""), Barline, Note("A")]);
    }

    #[test]
    fn lex_complex_notes() {
        lextest(
            ": | B#,,c_''d=",
            vec![Key(""), Barline, Note("B#,,"), Note("c_''"), Note("d=")],
        );
    }

    #[test]
    fn lex_note_length() {
        lextest(
            ": | A... B4 | C.",
            vec![
                Key(""),
                Barline,
                Note("A"),
                ExtendNote,
                ExtendNote,
                ExtendNote,
                Note("B"),
                Num(4),
                Barline,
                Note("C"),
                ExtendNote,
            ],
        );
    }

    #[test]
    fn lex_symbols() {
        lextest(
            "C : | x - x-| % | \" |",
            vec![
                Key("C"),
                Barline,
                Hit,
                Rest,
                Hit,
                Rest,
                Barline,
                RepeatBar,
                Barline,
                Ditto,
                Barline,
            ],
        );
    }

    #[test]
    fn lex_play_part() {
        lextest(":| *Theme", vec![Key(""), Barline, PlayPart("Theme")]);
    }

    #[test]
    fn semicolon_can_break_stave_within_one_line() {
        lextest(
            "A:|x;B:|x;",
            vec![Key("A"), Barline, Hit, Key("B"), Barline, Hit],
        );
    }

    #[test]
    fn right_brace_can_break_stave_within_one_line() {
        lextest(
            "{ A:|x }",
            vec![LeftBrace, Key("A"), Barline, Hit, RightBrace],
        );
    }
}
