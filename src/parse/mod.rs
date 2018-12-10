use std::borrow::Cow;

use failure::{self, Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseTree<'a> {
    pub pieces: Vec<Piece<'a>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Piece<'a> {
    pub title: Option<&'a str>,
    pub composer: Option<&'a str>,
    pub tempo: Option<u64>,
    pub beats: Option<u64>,

    pub voices: Vec<Voice<'a>>,
    pub plays: Vec<Play<'a>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Voice<'a> {
    pub name: Option<&'a str>,
    pub program: Option<u8>,
    pub channel: Option<u8>,
    pub transpose: Option<i8>,
    pub volume: Option<u8>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Play<'a> {
    pub voice: Option<&'a str>,
    pub staves: Vec<Stave<'a>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Stave<'a> {
    pub prefix: Cow<'a, str>,
    //     pub bars: Vec<BarTypeNode>,
}

struct Parser<'a> {
    pub source: &'a [u8],
    pub cursor: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Parser {
            source: source.as_bytes(),
            cursor: 0,
        }
    }

    #[inline(always)]
    pub fn finished(&self) -> bool {
        self.cursor == self.source.len()
    }

    #[inline(always)]
    pub fn check(&self, next: &[u8]) -> bool {
        let end = self.cursor + next.len();
        end <= self.source.len() && &self.source[self.cursor..end] == next
    }

    pub fn skip(&mut self, next: &[u8]) -> bool {
        let skipped = self.check(next);
        if skipped {
            self.cursor += next.len();
            self.skip_whitespace();
        }
        skipped
    }

    pub fn expect(&mut self, next: &[u8]) -> Result<(), Error> {
        if self.finished() {
            return Err(failure::err_msg(format!(
                "Expected {} but reached the end of the file.",
                ::std::str::from_utf8(next).unwrap()
            )));
        }

        let next_byte = self.source[self.cursor];

        if !self.skip(next) {
            Err(failure::err_msg(format!(
                "Expected {} but saw {}",
                ::std::str::from_utf8(next).unwrap(),
                next_byte
            )))
        } else {
            Ok(())
        }
    }

    pub fn check_keyword(&mut self, keyword: &[u8]) -> bool {
        fn is_ident_char(ch: u8) -> bool {
            ch == b'_'
                || (b'a' <= ch && ch <= b'z')
                || (b'A' <= ch && ch <= b'Z')
                || (b'0' <= ch && ch <= b'9')
        }

        let end = self.cursor + keyword.len();
        self.check(keyword) && (end == self.source.len() || !is_ident_char(self.source[end]))
    }

    pub fn skip_keyword(&mut self, keyword: &[u8]) -> bool {
        let success = self.check_keyword(keyword);
        if success {
            self.cursor += keyword.len();
            self.skip_whitespace();
        }
        success
    }

    pub fn skip_whitespace(&mut self) {
        fn is_whitespace(ch: u8) -> bool {
            match ch {
                b' ' | b'\t' | b'\r' | b'\n' => true,
                _ => false,
            }
        }

        while !self.finished() && is_whitespace(self.source[self.cursor]) {
            self.cursor += 1;
        }
    }

    pub fn check_attr(&mut self) -> Option<&'a [u8]> {
        fn is_attr_char(ch: u8) -> bool {
            ch == b'_'
                || (b'a' <= ch && ch <= b'z')
                || (b'A' <= ch && ch <= b'Z')
                || (b'0' <= ch && ch <= b'9')
        }

        let end = self.cursor + keyword.len();
        self.check(keyword) && (end == self.source.len() || !is_ident_char(self.source[end]))
    }

//     pub fn check_property(&mut self, keyword: &[u8]) -> bool {
// 
//         fn is_ident_char(ch: u8) -> bool {
//             ch == b'_'
//                 || (b'a' <= ch && ch <= b'z')
//                 || (b'A' <= ch && ch <= b'Z')
//                 || (b'0' <= ch && ch <= b'9')
//         }
// 
//         let end = self.cursor + keyword.len();
//         self.check(keyword) && (end == self.source.len() || !is_ident_char(self.source[end]))
//     }
}

pub fn parse<'a>(input: &'a str, filename: Option<&'a str>) -> Result<ParseTree<'a>, Error> {
    let parser = &mut Parser::new(input);

    let mut pieces = Vec::new();

    parser.skip_whitespace();

    loop {
        pieces.push(parse_piece(parser)?);

        if parser.finished() {
            break;
        }
    }

    Ok(ParseTree { pieces })
}

fn parse_piece<'a>(parser: &mut Parser) -> Result<Piece<'a>, Error> {
    if parser.skip_keyword(b"piece") {
        parser.expect(b"{")?;
        let piece = parse_piece_contents(parser)?;
        parser.expect(b"}")?;
        Ok(piece)
    } else {
        let piece = parse_piece_contents(parser)?;
        Ok(piece)
    }
}

fn parse_piece_contents<'a>(parser: &mut Parser) -> Result<Piece<'a>, Error> {
    enum BlockType {
        Play,
        Voice,
    }

    let mut piece = Piece::default();

    loop {
        let block_type = {
            if parser.skip_keyword(b"play") {
                BlockType::Play
            } else if parser.skip_keyword(b"voice") {
                BlockType::Voice
            } else {
                parser.skip_whitespace();
                break
            }
        };

        parser.expect(b"{")?;
        match block_type {
            BlockType::Play => {
                piece.plays.push(parse_play_contents(parser)?);
            }
            BlockType::Voice => {
                piece.voices.push(parse_voice_contents(parser)?);
            }
        }
        parser.expect(b"}")?;
    }

    Ok(piece)
}

fn parse_play_contents<'a>(parser: &mut Parser) -> Result<Play<'a>, Error> {
    Ok(Play::default())
}

fn parse_voice_contents<'a>(parser: &mut Parser) -> Result<Voice<'a>, Error> {
    Ok(Voice::default())
}

#[cfg(test)]
mod tests {

    use super::*;

    fn parse_succeeds(source: &str, result: ParseTree) {
        assert_eq!(parse(source, None).unwrap(), result);
    }

    fn parse_fails(source: &str) {
        assert!(parse(source, None).is_err());
    }

    #[test]
    fn parse_empty_file() {
        parse_succeeds(
            "",
            ParseTree {
                pieces: vec![Piece::default()],
            },
        );
    }

    #[test]
    fn parse_empty_piece() {
        parse_succeeds(
            "piece {}",
            ParseTree {
                pieces: vec![Piece::default()],
            },
        );
    }

    #[test]
    fn parse_empty_piece_with_whitespace() {
        parse_succeeds(
            "piece {\t   \n}",
            ParseTree {
                pieces: vec![Piece::default()],
            },
        );
    }

    #[test]
    fn parse_empty_pieces() {
        parse_succeeds(
            "piece{}piece{}",
            ParseTree {
                pieces: vec![Piece::default(), Piece::default()],
            },
        );
    }

    #[test]
    fn parse_empty_pieces_with_whitespace() {
        parse_succeeds(
            "piece {\n}piece\t{ }",
            ParseTree {
                pieces: vec![Piece::default(), Piece::default()],
            },
        );
    }

    #[test]
    fn fail_unopened_piece() {
        parse_fails("piece");
    }

    #[test]
    fn fail_unclosed_piece() {
        parse_fails("piece {");
    }

    #[test]
    fn fail_unexpected_token_piece() {
        parse_fails("piece @");
    }

    #[test]
    fn parse_piece_with_anon_empty_voice() {
        parse_succeeds(
            "piece { voice { } }",
            ParseTree {
                pieces: vec![Piece {
                    voices: vec![Voice::default()],
                    ..Piece::default()
                }],
            },
        )
    }

    #[test]
    fn parse_piece_with_anon_empty_play() {
        parse_succeeds(
            "piece { play { } }",
            ParseTree {
                pieces: vec![Piece {
                    plays: vec![Play::default()],
                    ..Piece::default()
                }],
            },
        )
    }

    #[test]
    fn parse_piece_with_anon_empty_voice_and_play() {
        parse_succeeds(
            "piece { play { } voice { } }",
            ParseTree {
                pieces: vec![Piece {
                    plays: vec![Play::default()],
                    voices: vec![Voice::default()],
                    ..Piece::default()
                }],
            },
        )
    }

    #[test]
    fn parse_solo_anon_empty_voice() {
        parse_succeeds(
            "voice { }",
            ParseTree {
                pieces: vec![Piece {
                    voices: vec![Voice::default()],
                    ..Piece::default()
                }],
            },
        )
    }

    #[test]
    fn parse_solo_anon_empty_play() {
        parse_succeeds(
            "play { }",
            ParseTree {
                pieces: vec![Piece {
                    plays: vec![Play::default()],
                    ..Piece::default()
                }],
            },
        )
    }

    #[test]
    fn parse_solo_anon_empty_voice_and_play() {
        parse_succeeds(
            "play { } voice { }",
            ParseTree {
                pieces: vec![Piece {
                    plays: vec![Play::default()],
                    voices: vec![Voice::default()],
                    ..Piece::default()
                }],
            },
        )
    }
}
