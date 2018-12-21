use failure::{self, Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseTree<'a> {
    pub pieces: Vec<Piece<'a>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Piece<'a> {
    pub title: Option<&'a [u8]>,
    pub composer: Option<&'a [u8]>,
    pub tempo: Option<u64>,
    pub beats: Option<u64>,

    pub voices: Vec<Voice<'a>>,
    pub plays: Vec<Play<'a>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Voice<'a> {
    pub name: Option<&'a [u8]>,
    pub program: Option<u8>,
    pub channel: Option<u8>,
    pub transpose: Option<i8>,
    pub volume: Option<u8>,
    pub drums: Option<bool>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Play<'a> {
    pub name: Option<&'a [u8]>,
    pub grand_staves: Vec<GrandStave<'a>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GrandStave<'a> {
    pub staves: Vec<Stave<'a>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Stave<'a> {
    pub prefix: Option<&'a [u8]>,
    pub bars: Vec<Bar<'a>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Bar<'a> {
    pub contents: Vec<StaveEvent<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaveEvent<'a> {
    pub event: StaveEventType<'a>,
    pub duration: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaveEventType<'a> {
    Rest,
    Hit,
    Prolong,
    Note(Note),
    RepeatBars(usize),
    PlayPart(&'a [u8]),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub note: NoteSymbol,
    pub acc: Accidental,
    pub octave: i8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NoteSymbol {
    LowA,
    LowB,
    LowC,
    LowD,
    LowE,
    LowF,
    LowG,
    HighA,
    HighB,
    HighC,
    HighD,
    HighE,
    HighF,
    HighG,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Accidental {
    Flat,
    Natural,
    Sharp,
}

fn is_whitespace(ch: u8) -> bool {
    match ch {
        b' ' | b'\t' | b'\r' => true,
        _ => false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

    #[allow(dead_code)]
    fn debug_position(&self) {
        let before = self.cursor - std::cmp::min(self.cursor, 20);
        let end = std::cmp::min(self.cursor + 100, self.source.len());
        eprintln!(
            "{}«{}»",
            std::str::from_utf8(&self.source[before..self.cursor]).unwrap(),
            std::str::from_utf8(&self.source[self.cursor..end]).unwrap()
        );
    }

    #[allow(dead_code)]
    #[allow(unused_variables)]
    #[inline(always)]
    fn log(&self, message: &str) {
        #[cfg(feature = "verbose")]
        {
            eprint!("{}: ", message);
            self.debug_position();
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

    pub fn skip_only(&mut self, next: &[u8]) -> bool {
        let skipped = self.check(next);
        if skipped {
            self.cursor += next.len();
        }
        skipped
    }

    pub fn expect(&mut self, next: &[u8]) -> Result<(), Error> {
        if self.finished() {
            return Err(failure::err_msg(format!(
                "Expected `{}` but reached the end of the file.",
                ::std::str::from_utf8(next).unwrap()
            )));
        }

        let next_byte = self.source[self.cursor];

        if !self.skip(next) {
            Err(failure::err_msg(format!(
                "Expected `{}` but saw `{}`",
                ::std::str::from_utf8(next).unwrap(),
                ::std::str::from_utf8(&[next_byte]).unwrap(),
            )))
        } else {
            Ok(())
        }
    }

    // TODO: Lots of unpleasant duplication
    pub fn expect_only(&mut self, next: &[u8]) -> Result<(), Error> {
        if self.finished() {
            return Err(failure::err_msg(format!(
                "Expected `{}` but reached the end of the file.",
                ::std::str::from_utf8(next).unwrap()
            )));
        }

        let next_byte = self.source[self.cursor];

        if !self.skip_only(next) {
            Err(failure::err_msg(format!(
                "Expected `{}` but saw `{}`",
                ::std::str::from_utf8(next).unwrap(),
                ::std::str::from_utf8(&[next_byte]).unwrap(),
            )))
        } else {
            Ok(())
        }
    }

    pub fn check_range_only(&mut self, min: u8, max: u8) -> Option<u8> {
        if self.cursor >= self.source.len() {
            return None;
        }

        let ch = self.source[self.cursor];
        if ch >= min && ch <= max {
            Some(ch)
        } else {
            None
        }
    }

    pub fn skip_range_only(&mut self, min: u8, max: u8) -> Option<u8> {
        match self.check_range_only(min, max) {
            result @ Some(_) => {
                self.cursor += 1;
                result
            }
            None => None,
        }
    }

    pub fn check_set_only(&mut self, options: &[u8]) -> Option<u8> {
        if self.cursor >= self.source.len() {
            return None;
        }

        let ch = self.source[self.cursor];
        options.iter().find(|&&opt| ch == opt).cloned()
    }

    pub fn skip_set_only(&mut self, options: &[u8]) -> Option<u8> {
        match self.check_set_only(options) {
            result @ Some(_) => {
                self.cursor += 1;
                result
            }
            None => None,
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
        let mut in_comment = false;
        loop {
            if self.skip_only(b"//") {
                in_comment = true;
            } else if self.skip_only(b"\n") {
                in_comment = false;
            } else {
                if self.finished() || !(in_comment || is_whitespace(self.source[self.cursor])) {
                    break;
                }

                self.cursor += 1;
            }
        }
    }

    pub fn skip_whitespace_in_line(&mut self) {
        let mut in_comment = false;
        loop {
            if self.skip_only(b"//") {
                in_comment = true;
            } else {
                if self.finished()
                    || self.check(b"\n")
                    || !(in_comment || is_whitespace(self.source[self.cursor]))
                {
                    break;
                }

                self.cursor += 1;
            }
        }
    }

    pub fn check_attr(&mut self) -> Option<&'a [u8]> {
        fn is_attr_char(ch: u8) -> bool {
            ch == b'_'
                || ch == b','
                || ch == b'\''
                || ch == b'#'
                || (b'a' <= ch && ch <= b'z')
                || (b'A' <= ch && ch <= b'Z')
                || (b'0' <= ch && ch <= b'9')
        }

        let mut end = self.cursor;
        while end < self.source.len() {
            if is_attr_char(self.source[end]) {
                end += 1;
            } else {
                break;
            }
        }

        if end == self.cursor {
            None
        } else {
            Some(&self.source[self.cursor..end])
        }
    }

    pub fn parse_attr(&mut self) -> Option<&'a [u8]> {
        let attr = self.check_attr();
        if let Some(attr) = attr {
            self.cursor += attr.len();
            self.skip_whitespace();
        }
        attr
    }

    pub fn parse_number_only<T: std::str::FromStr>(&mut self) -> Result<T, Error> {
        fn is_digit(ch: u8) -> bool {
            ch >= b'0' && ch <= b'9'
        }

        let mut end = self.cursor;
        while end < self.source.len() {
            if is_digit(self.source[end]) || (end == self.cursor && self.source[end] == b'-') {
                end += 1;
            } else {
                break;
            }
        }

        let result: &str = std::str::from_utf8(&self.source[self.cursor..end])
            .map_err(|_| failure::err_msg("Invalid attribute value - must be utf8"))?;
        let result: T = result
            .parse()
            .map_err(|_| failure::err_msg("Could not parse number"))?;

        self.cursor = end;
        Ok(result)
    }

    pub fn parse_string_only(&mut self) -> Result<&'a [u8], Error> {
        // We only accept UTF-8 so this should be safe.
        let source_str = unsafe { std::str::from_utf8_unchecked(&self.source[self.cursor..]) };

        let mut started = false;
        let mut escaping = false;
        for (i, ch) in source_str.char_indices() {
            if started {
                match ch {
                    '\\' if !escaping => escaping = true,
                    '"' if !escaping => {
                        self.cursor += i + 1;
                        return Ok(&source_str[1..i].as_bytes());
                    }
                    _ => escaping = false,
                }
            } else {
                if ch != '"' {
                    return Err(failure::err_msg("String must open with `\"`"));
                }
                started = true;
            }
        }

        Err(failure::err_msg("Unclosed string!"))
    }

    pub fn parse_bool_only(&mut self) -> Result<bool, Error> {
        if self.skip_keyword(b"true") {
            Ok(true)
        } else if self.skip_keyword(b"false") {
            Ok(false)
        } else {
            Err(failure::err_msg("Failed to parse bool."))
        }
    }

    pub fn skip_end_of_stave(&mut self) -> bool {
        self.finished() || self.skip_only(b"\n") || self.skip_only(b";") || self.check(b"}")
    }
}

pub fn parse<'a>(input: &'a str, _filename: Option<&'a str>) -> Result<ParseTree<'a>, Error> {
    let parser = &mut Parser::new(input);

    let mut pieces = Vec::new();

    parser.skip_whitespace();

    loop {
        parser.log("parse loop");

        pieces.push(parse_piece(parser)?);

        if parser.finished() {
            break;
        }
    }

    Ok(ParseTree { pieces })
}

fn parse_piece<'a>(parser: &mut Parser<'a>) -> Result<Piece<'a>, Error> {
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

fn parse_piece_contents<'a>(parser: &mut Parser<'a>) -> Result<Piece<'a>, Error> {
    enum BlockType<'a> {
        Play(Option<&'a [u8]>),
        Voice(Option<&'a [u8]>),
    }

    let mut piece = Piece::default();

    loop {
        parser.log("parse_piece_contents loop");

        let block_type = {
            if parser.skip_keyword(b"play") {
                BlockType::Play(parser.parse_attr())
            } else if parser.skip_keyword(b"voice") {
                BlockType::Voice(parser.parse_attr())
            } else if let Some(attr_name) = parser.parse_attr() {
                parser.expect(b":")?;

                // TODO: more ugly duplication...
                match attr_name {
                    b"tempo" => piece.tempo = Some(parser.parse_number_only()?),
                    b"beats" => piece.beats = Some(parser.parse_number_only()?),
                    b"title" => piece.title = Some(parser.parse_string_only()?),
                    b"composer" => piece.composer = Some(parser.parse_string_only()?),
                    _ => return Err(failure::err_msg("Invalid attribute name")),
                }

                parser.skip_whitespace_in_line();
                let attribute_ended = parser.finished()
                    || parser.skip(b",")
                    || parser.skip(b"\n")
                    || parser.skip(b";")
                    || parser.check(b"}");

                if !attribute_ended {
                    return Err(failure::err_msg(
                        "Attributes must end with a newline, comma, or semi-colon.",
                    ));
                }

                continue;
            } else {
                parser.skip_whitespace();

                let done = parser.finished() || parser.check(b"}");
                if !done {
                    // Top-level contents are considered a play block
                    piece.plays.push(parse_play_contents(parser, None)?);
                    parser.skip_whitespace();
                }

                break;
            }
        };

        parser.expect(b"{")?;
        match block_type {
            BlockType::Play(name) => {
                piece.plays.push(parse_play_contents(parser, name)?);
            }
            BlockType::Voice(name) => {
                piece.voices.push(parse_voice_contents(parser, name)?);
            }
        }
        parser.expect(b"}")?;
    }

    Ok(piece)
}

fn parse_voice_contents<'a>(
    parser: &mut Parser<'a>,
    name: Option<&'a [u8]>,
) -> Result<Voice<'a>, Error> {
    let mut voice = Voice {
        name,
        ..Voice::default()
    };

    while let Some(attr_name) = parser.parse_attr() {
        parser.expect(b":")?;

        match attr_name {
            b"program" => voice.program = Some(parser.parse_number_only()?),
            b"channel" => voice.channel = Some(parser.parse_number_only()?),
            b"octave" => voice.transpose = Some(parser.parse_number_only::<i8>()? * 12),
            b"volume" => voice.volume = Some(parser.parse_number_only()?),
            b"drums" => voice.drums = Some(parser.parse_bool_only()?),
            _ => return Err(failure::err_msg("Invalid attribute name")),
        }

        parser.skip_whitespace_in_line();
        if !(parser.skip(b",") || parser.skip(b"\n") || parser.skip(b";")) {
            break;
        }
    }

    Ok(voice)
}

fn parse_play_contents<'a>(
    parser: &mut Parser<'a>,
    name: Option<&'a [u8]>,
) -> Result<Play<'a>, Error> {
    let mut play = Play {
        name,
        ..Play::default()
    };

    let mut working_grand_stave = GrandStave::default();

    loop {
        parser.log("parse_play_contents loop");

        let attr_name = parser.parse_attr();

        if parser.skip(b":") {
            parser.log("Starting a play attribute");
            if parser.skip_only(b"|") {
                // Parse a stave
                parser.skip_whitespace_in_line();
                working_grand_stave
                    .staves
                    .push(parse_stave_contents(parser, attr_name)?);

                if parser.skip_end_of_stave() {
                    parser.skip_whitespace();

                    play.grand_staves.push(std::mem::replace(
                        &mut working_grand_stave,
                        GrandStave::default(),
                    ));
                }
            } else {
                // Parse an attribute value
                return Err(failure::err_msg(
                    "Attributes in play blocks not currently supported. Use `|` to start a stave.",
                ));
            }
        } else {
            if let Some(attr_name) = attr_name {
                return Err(failure::err_msg(format!(
                    "Attribute `{}` is missing a value.",
                    std::str::from_utf8(attr_name).unwrap()
                )));
            }

            parser.skip_whitespace();
            break;
        }
    }

    Ok(play)
}

fn parse_stave_contents<'a>(
    parser: &mut Parser<'a>,
    stave_prefix: Option<&'a [u8]>,
) -> Result<Stave<'a>, Error> {
    let mut done = false;
    let mut working_event = None;
    let mut working_bar = vec![];
    let mut bars = vec![];

    while !done {
        parser.log("parse_stave_contents loop");

        let mut bar_ended = false;

        // TODO: [chords] (tuplets)3
        if parser.skip_only(b"x") {
            working_event = Some(StaveEventType::Hit);
        } else if parser.skip_only(b"-") {
            working_event = Some(StaveEventType::Rest);
        } else if parser.skip_only(b".") {
            working_event = Some(StaveEventType::Prolong);
        } else if let Some(note) = parse_note(parser) {
            working_event = Some(StaveEventType::Note(note));
        } else if parser.skip_only(b"%") {
            // TODO: Lot going on here
            if parser.skip_only(b"(") {
                parser.skip_whitespace_in_line();
                if let Some(bar_count) = parser.parse_number_only::<usize>().ok() {
                    working_event = Some(StaveEventType::RepeatBars(bar_count));
                    parser.skip_whitespace_in_line();
                } else if let Some(part_name) = parser.parse_attr() {
                    working_event = Some(StaveEventType::PlayPart(part_name));
                } else {
                    return Err(failure::err_msg("Expected bar count or part name"));
                }
                parser.expect_only(b")")?;
            } else {
                working_event = Some(StaveEventType::RepeatBars(1));
            }
        } else if parser.skip_only(b"|") {
            bar_ended = true;
        } else if parser.skip_end_of_stave() {
            bar_ended = true;
            parser.skip_whitespace_in_line();
            if !parser.skip_only(b"|") {
                done = true;
            }
        } else {
            parser.log("Unexpected character in stave!");
            return Err(failure::err_msg("Unexpected character in stave!"));
        }

        if let Some(event_type) = working_event.take() {
            parser.skip_whitespace_in_line();
            let duration = parser.parse_number_only::<usize>().unwrap_or(1);

            working_bar.push(StaveEvent {
                event: event_type,
                duration,
            });
        }

        if bar_ended && !working_bar.is_empty() {
            let bar = Bar {
                contents: std::mem::replace(&mut working_bar, vec![]),
            };
            bars.push(bar);
        }

        parser.skip_whitespace_in_line();
    }

    Ok(Stave {
        prefix: stave_prefix,
        bars,
        ..Default::default()
    })
}

fn parse_note(parser: &mut Parser) -> Option<Note> {
    let note_symbol = parser
        .skip_range_only(b'a', b'g')
        .or_else(|| parser.skip_range_only(b'A', b'G'))?;
    let note_symbol = match note_symbol {
        b'A' => NoteSymbol::LowA,
        b'B' => NoteSymbol::LowB,
        b'C' => NoteSymbol::LowC,
        b'D' => NoteSymbol::LowD,
        b'E' => NoteSymbol::LowE,
        b'F' => NoteSymbol::LowF,
        b'G' => NoteSymbol::LowG,
        b'a' => NoteSymbol::HighA,
        b'b' => NoteSymbol::HighB,
        b'c' => NoteSymbol::HighC,
        b'd' => NoteSymbol::HighD,
        b'e' => NoteSymbol::HighE,
        b'f' => NoteSymbol::HighF,
        b'g' => NoteSymbol::HighG,
        _ => return None,
    };

    let acc = parser
        .skip_set_only(b"_=#")
        .map(|acc| match acc {
            b'_' => Accidental::Flat,
            b'=' => Accidental::Natural,
            b'#' => Accidental::Sharp,
            _ => unreachable!(),
        })
        .unwrap_or(Accidental::Natural);

    let mut octave = 0_i8;
    while let Some(octave_mod) = parser.skip_set_only(b",'") {
        match octave_mod {
            b',' => octave -= 1,
            b'\'' => octave += 1,
            _ => unreachable!(),
        }
    }

    Some(Note {
        note: note_symbol,
        acc,
        octave,
    })
}

#[cfg(test)]
mod tests {
    // TODO: more tests covering parse failure

    use super::*;

    fn parse_succeeds(source: &str, result: ParseTree) {
        assert_eq!(parse(source, None).unwrap(), result);
    }

    fn parse_equivalent(variants: &[&str], result: ParseTree) {
        for variant in variants {
            parse_succeeds(variant, result.clone());
        }
    }

    fn parse_fails(source: &str) {
        assert!(parse(source, None).is_err());
    }

    fn plays_tree(plays: &[Play<'static>]) -> ParseTree<'static> {
        ParseTree {
            pieces: vec![Piece {
                plays: plays.to_owned(),
                ..Piece::default()
            }],
        }
    }

    fn grand_stave(events: &[&[&[StaveEvent<'static>]]]) -> GrandStave<'static> {
        GrandStave {
            staves: events
                .iter()
                .map(|bars| Stave {
                    prefix: None,
                    bars: bars
                        .iter()
                        .map(|&events| Bar {
                            contents: events.to_owned(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    const EMPTY_STAVE: &[&[StaveEvent<'static>]] = &[];

    #[test]
    fn parse_empty_piece() {
        parse_equivalent(
            &["", "  piece {}", "piece {}", "piece {\t   \n}"],
            ParseTree {
                pieces: vec![Piece::default()],
            },
        );
    }

    #[test]
    fn parse_empty_pieces() {
        parse_equivalent(
            &["piece{}piece{}", "piece {\n}piece\t{ }"],
            ParseTree {
                pieces: vec![Piece::default(), Piece::default()],
            },
        );
    }

    #[test]
    fn parse_piece_with_attributes() {
        parse_equivalent(
            &[
                "piece { tempo: 120, beats: 4 }",
                "piece {
                    tempo: 120,
                    beats: 4,
                 }",
                "tempo: 120
                 beats: 4
                ",
            ],
            ParseTree {
                pieces: vec![Piece {
                    tempo: Some(120),
                    beats: Some(4),
                    ..Piece::default()
                }],
            },
        );
    }

    #[test]
    fn parse_toplevel_piece_attributes() {
        parse_succeeds(
            r#"title: "Title", composer: "Composer""#,
            ParseTree {
                pieces: vec![Piece {
                    title: Some(b"Title"),
                    composer: Some(b"Composer"),
                    ..Piece::default()
                }],
            },
        );
    }

    #[test]
    fn parse_piece_with_all_attributes() {
        parse_succeeds(
            r#"piece {
                title: "Title",
                composer: "Composer",
                tempo: 100,
                beats: 3,
             }"#,
            ParseTree {
                pieces: vec![Piece {
                    title: Some(b"Title"),
                    composer: Some(b"Composer"),
                    tempo: Some(100),
                    beats: Some(3),
                    plays: vec![],
                    voices: vec![],
                }],
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
        );
    }

    #[test]
    fn parse_piece_with_anon_empty_play() {
        parse_succeeds("piece { play { } }", plays_tree(&[Play::default()]));
    }

    #[test]
    fn parse_piece_with_anon_empty_voice_and_play() {
        parse_equivalent(
            &[
                "piece { play { } voice { } }",
                "piece { voice { } play { } }",
            ],
            ParseTree {
                pieces: vec![Piece {
                    plays: vec![Play::default()],
                    voices: vec![Voice::default()],
                    ..Piece::default()
                }],
            },
        );
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
        );
    }

    #[test]
    fn parse_solo_anon_empty_play() {
        parse_succeeds("play { }", plays_tree(&[Play::default()]));
    }

    #[test]
    fn parse_solo_named_play() {
        parse_equivalent(
            &[
                "play Named {}",
                "play Named{}",
                "play Named
                 {
                 }",
            ],
            plays_tree(&[Play {
                name: Some(b"Named"),
                ..Play::default()
            }]),
        );
    }

    #[test]
    fn parse_solo_named_voice() {
        parse_equivalent(
            &[
                "voice Named {}",
                "voice Named{}",
                "voice Named
                 {
                 }",
            ],
            ParseTree {
                pieces: vec![Piece {
                    voices: vec![Voice {
                        name: Some(b"Named"),
                        ..Voice::default()
                    }],
                    ..Piece::default()
                }],
            },
        );
    }

    #[test]
    fn parse_solo_anon_empty_voice_and_play() {
        parse_equivalent(
            &["play { } voice { }", "voice { } play { }"],
            ParseTree {
                pieces: vec![Piece {
                    plays: vec![Play::default()],
                    voices: vec![Voice::default()],
                    ..Piece::default()
                }],
            },
        );
    }

    #[test]
    fn parse_voice_with_single_attribute() {
        parse_equivalent(
            &[
                "voice { program:10 }",
                "voice { program: 10 }",
                "voice { program: 10, }",
                "voice {
                    program: 10
                }",
                "voice {
                    program: 10,
                }",
            ],
            ParseTree {
                pieces: vec![Piece {
                    voices: vec![Voice {
                        program: Some(10),
                        ..Voice::default()
                    }],
                    ..Piece::default()
                }],
            },
        );
    }

    #[test]
    fn parse_voice_with_multiple_attributes() {
        parse_equivalent(
            &[
                "voice { program: 30, channel: 2 }",
                "voice { program: 30, channel: 2, }",
                "voice { program: 30; channel: 2; }",
                "voice { program: 30
                    channel: 2, }",
                "voice {
                    program: 30
                    channel: 2
                }",
                "voice {
                    program: 30,
                    channel: 2,
                }",
            ],
            ParseTree {
                pieces: vec![Piece {
                    voices: vec![Voice {
                        program: Some(30),
                        channel: Some(2),
                        ..Voice::default()
                    }],
                    ..Piece::default()
                }],
            },
        );
    }

    #[test]
    fn parse_voice_with_all_attributes() {
        parse_succeeds(
            "voice {
                octave: -2,
                channel: 3,
                program: 5,
                volume: 8,
                drums: true,
            }",
            ParseTree {
                pieces: vec![Piece {
                    voices: vec![Voice {
                        transpose: Some(-24),
                        channel: Some(3),
                        program: Some(5),
                        volume: Some(8),
                        drums: Some(true),
                        name: None,
                    }],
                    ..Piece::default()
                }],
            },
        );
    }

    #[test]
    fn parse_play_with_one_grand_stave_and_one_basic_stave() {
        parse_equivalent(
            &[
                "play { :| }",
                "play { : | }",
                "play { :
                    |
                }",
                "play {
                    :|
                }",
            ],
            plays_tree(&[Play {
                grand_staves: vec![grand_stave(&[EMPTY_STAVE])],
                ..Play::default()
            }]),
        );
    }

    #[test]
    fn parse_play_with_one_grand_stave_and_two_basic_staves() {
        parse_equivalent(
            &[
                "play {
                    :| ; :|
                }",
                "play {
                    :|
                    :|
                }",
            ],
            plays_tree(&[Play {
                grand_staves: vec![grand_stave(&[EMPTY_STAVE, EMPTY_STAVE])],
                ..Play::default()
            }]),
        );
    }

    #[test]
    fn parse_play_with_two_grand_staves() {
        parse_equivalent(
            &[
                "play { :| ;; :| }",
                "play { :| ; ; :| }",
                //                 "play { :| ;;; :| }", // TODO: This fails because a line starts with `; How should that be handled?
                "play {
                    :| ;
                    :|
                }",
                "play {
                    :|

                    :|
                }",
                "play {
                    :|




                    :|
                }",
            ],
            plays_tree(&[Play {
                grand_staves: vec![grand_stave(&[EMPTY_STAVE]), grand_stave(&[EMPTY_STAVE])],
                ..Play::default()
            }]),
        );
    }

    #[test]
    fn parse_solo_stave_as_play_block() {
        parse_succeeds(
            ":|",
            plays_tree(&[Play {
                grand_staves: vec![grand_stave(&[EMPTY_STAVE])],
                ..Play::default()
            }]),
        );
    }

    #[test]
    fn parse_solo_two_staves() {
        parse_equivalent(
            &[
                ":|
                 :|",
                ":| ; :|",
                ":|
                  |
                 :|",
            ],
            plays_tree(&[Play {
                grand_staves: vec![grand_stave(&[EMPTY_STAVE, EMPTY_STAVE])],
                ..Play::default()
            }]),
        );
    }

    #[test]
    fn parse_toplevel_piece_attributes_and_toplevel_staves() {
        parse_succeeds(
            r#"tempo: 160
               beats: 6

               :|
            "#,
            ParseTree {
                pieces: vec![Piece {
                    tempo: Some(160),
                    beats: Some(6),
                    plays: vec![Play {
                        grand_staves: vec![grand_stave(&[EMPTY_STAVE])],
                        ..Play::default()
                    }],
                    ..Piece::default()
                }],
            },
        );
    }

    #[test]
    fn comments_are_whitespace() {
        parse_equivalent(
            &[
                "play PlayName { :| ;; :| ; :| } // Comment at end",
                "play PlayName { // Comments
                    :|           // in
                                 // every
                    :| ; :|      // line
                }",
                "play // Comments on
                 PlayName // some of the
                 {
                    :|

                    :|
                    :|
                 } // lines
                ",
            ],
            plays_tree(&[Play {
                name: Some(b"PlayName"),
                grand_staves: vec![
                    grand_stave(&[EMPTY_STAVE]),
                    grand_stave(&[EMPTY_STAVE, EMPTY_STAVE]),
                ],
                ..Play::default()
            }]),
        );
    }

    fn hit(duration: usize) -> StaveEvent<'static> {
        StaveEvent {
            event: StaveEventType::Hit,
            duration,
        }
    }

    fn prolong(duration: usize) -> StaveEvent<'static> {
        StaveEvent {
            event: StaveEventType::Prolong,
            duration,
        }
    }

    fn rest(duration: usize) -> StaveEvent<'static> {
        StaveEvent {
            event: StaveEventType::Rest,
            duration,
        }
    }

    fn note(note: NoteSymbol, duration: usize) -> StaveEvent<'static> {
        StaveEvent {
            event: StaveEventType::Note(Note {
                note,
                acc: Accidental::Natural,
                octave: 0,
            }),
            duration,
        }
    }

    #[test]
    fn parse_hits() {
        parse_equivalent(
            &[":|x|xx|xxx|", ":| x | x x | xx x |"],
            plays_tree(&[Play {
                grand_staves: vec![grand_stave(&[&[
                    &[hit(1)],
                    &[hit(1), hit(1)],
                    &[hit(1), hit(1), hit(1)],
                ]])],
                ..Play::default()
            }]),
        );
    }

    #[test]
    fn parse_prolong() {
        parse_equivalent(
            &[":|..|x.|", ":| . . | x . |"],
            plays_tree(&[Play {
                grand_staves: vec![grand_stave(&[&[
                    &[prolong(1), prolong(1)],
                    &[hit(1), prolong(1)],
                ]])],
                ..Play::default()
            }]),
        );
    }

    #[test]
    fn parse_rests() {
        parse_equivalent(
            &[":|-|-.|x--|", ":| - | - . | x - - |"],
            plays_tree(&[Play {
                grand_staves: vec![grand_stave(&[&[
                    &[rest(1)],
                    &[rest(1), prolong(1)],
                    &[hit(1), rest(1), rest(1)],
                ]])],
                ..Play::default()
            }]),
        );
    }

    #[test]
    fn parse_single_repeat() {
        parse_equivalent(
            &[":|%", ":| % |"],
            plays_tree(&[Play {
                grand_staves: vec![grand_stave(&[&[&[StaveEvent {
                    event: StaveEventType::RepeatBars(1),
                    duration: 1,
                }]]])],
                ..Play::default()
            }]),
        );
    }

    #[test]
    fn parse_double_repeat() {
        parse_equivalent(
            &[":|%%", ":| % % |"],
            plays_tree(&[Play {
                grand_staves: vec![grand_stave(&[&[&[
                    StaveEvent {
                        event: StaveEventType::RepeatBars(1),
                        duration: 1,
                    },
                    StaveEvent {
                        event: StaveEventType::RepeatBars(1),
                        duration: 1,
                    },
                ]]])],
                ..Play::default()
            }]),
        );
    }

    #[test]
    fn parse_multi_bar_repeat() {
        parse_equivalent(
            &[":|%(3)", ":| %(3) |", ":| %( 3 ) |"],
            plays_tree(&[Play {
                grand_staves: vec![grand_stave(&[&[&[StaveEvent {
                    event: StaveEventType::RepeatBars(3),
                    duration: 1,
                }]]])],
                ..Play::default()
            }]),
        );
    }

    #[test]
    fn parse_play_part() {
        parse_equivalent(
            &[
                ":|%(part_name_2)",
                ":| %(part_name_2 ) |",
                ":| %( part_name_2 ) |",
            ],
            plays_tree(&[Play {
                grand_staves: vec![grand_stave(&[&[&[StaveEvent {
                    event: StaveEventType::PlayPart(b"part_name_2"),
                    duration: 1,
                }]]])],
                ..Play::default()
            }]),
        );
    }

    #[test]
    fn parse_play_notes() {
        parse_equivalent(
            &[":|c|cd|cde|", ":| c | c d | cd e |"],
            plays_tree(&[Play {
                grand_staves: vec![grand_stave(&[&[
                    &[note(NoteSymbol::HighC, 1)],
                    &[note(NoteSymbol::HighC, 1), note(NoteSymbol::HighD, 1)],
                    &[
                        note(NoteSymbol::HighC, 1),
                        note(NoteSymbol::HighD, 1),
                        note(NoteSymbol::HighE, 1),
                    ],
                ]])],
                ..Play::default()
            }]),
        );
    }

    #[test]
    fn parse_durations() {
        parse_equivalent(
            &[":|-|-1|-3"],
            plays_tree(&[Play {
                grand_staves: vec![grand_stave(&[&[&[rest(1)], &[rest(1)], &[rest(3)]]])],
                ..Play::default()
            }]),
        );
    }

    // TODO: More sophisticated testing for this
    #[test]
    fn parse_notes() {
        fn single_note_tree(note: Note) -> ParseTree<'static> {
            plays_tree(&[Play {
                grand_staves: vec![grand_stave(&[&[&[StaveEvent {
                    event: StaveEventType::Note(note),
                    duration: 1,
                }]]])],
                ..Play::default()
            }])
        }

        parse_succeeds(
            ":| A",
            single_note_tree(Note {
                note: NoteSymbol::LowA,
                acc: Accidental::Natural,
                octave: 0,
            }),
        );
        parse_succeeds(
            ":| G",
            single_note_tree(Note {
                note: NoteSymbol::LowG,
                acc: Accidental::Natural,
                octave: 0,
            }),
        );
        parse_succeeds(
            ":| a",
            single_note_tree(Note {
                note: NoteSymbol::HighA,
                acc: Accidental::Natural,
                octave: 0,
            }),
        );
        parse_succeeds(
            ":| g",
            single_note_tree(Note {
                note: NoteSymbol::HighG,
                acc: Accidental::Natural,
                octave: 0,
            }),
        );
        parse_succeeds(
            ":| c#",
            single_note_tree(Note {
                note: NoteSymbol::HighC,
                acc: Accidental::Sharp,
                octave: 0,
            }),
        );
        parse_succeeds(
            ":| c_",
            single_note_tree(Note {
                note: NoteSymbol::HighC,
                acc: Accidental::Flat,
                octave: 0,
            }),
        );
        parse_succeeds(
            ":| c",
            single_note_tree(Note {
                note: NoteSymbol::HighC,
                acc: Accidental::Natural,
                octave: 0,
            }),
        );
        parse_succeeds(
            ":| c#,",
            single_note_tree(Note {
                note: NoteSymbol::HighC,
                acc: Accidental::Sharp,
                octave: -1,
            }),
        );
        parse_succeeds(
            ":| c_,",
            single_note_tree(Note {
                note: NoteSymbol::HighC,
                acc: Accidental::Flat,
                octave: -1,
            }),
        );
        parse_succeeds(
            ":| c,",
            single_note_tree(Note {
                note: NoteSymbol::HighC,
                acc: Accidental::Natural,
                octave: -1,
            }),
        );
        parse_succeeds(
            ":| c#'",
            single_note_tree(Note {
                note: NoteSymbol::HighC,
                acc: Accidental::Sharp,
                octave: 1,
            }),
        );
        parse_succeeds(
            ":| c_'",
            single_note_tree(Note {
                note: NoteSymbol::HighC,
                acc: Accidental::Flat,
                octave: 1,
            }),
        );
        parse_succeeds(
            ":| c'",
            single_note_tree(Note {
                note: NoteSymbol::HighC,
                acc: Accidental::Natural,
                octave: 1,
            }),
        );
        parse_succeeds(
            ":| c'''",
            single_note_tree(Note {
                note: NoteSymbol::HighC,
                acc: Accidental::Natural,
                octave: 3,
            }),
        );
        parse_succeeds(
            ":| c,,,",
            single_note_tree(Note {
                note: NoteSymbol::HighC,
                acc: Accidental::Natural,
                octave: -3,
            }),
        );
    }
}
