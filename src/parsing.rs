use color_eyre::eyre::{eyre, Result};
use sashimi::{DefaultRules, ExactRules, LineBasedRules, Parser};

use crate::{
    error::{SourceInfo, SourceMap},
    notes::Midi,
    parsing_old::data::ParseTree,
};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Decimal<'a>(&'a [u8]);

#[derive(Debug, PartialEq, Eq)]
pub struct MeloFile<'a>(pub Section<'a>);

#[derive(Debug, Default, PartialEq, Eq)]
pub struct MetaAttr<'a> {
    span: &'a [u8],
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Section<'a> {
    meta: Vec<MetaAttr<'a>>,

    id: Option<&'a [u8]>,

    title: Option<&'a [u8]>,
    composer: Option<&'a [u8]>,
    beats: Option<usize>,
    tempo: Option<Decimal<'a>>,

    octave: Option<i8>,
    transpose: Option<i8>,
    volume: Option<Decimal<'a>>,

    voices: Vec<Voice<'a>>,
    sections: Vec<Section<'a>>,
    parts: Vec<Part<'a>>,
    plays: Vec<Play<'a>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Voice<'a> {
    meta: Vec<MetaAttr<'a>>,

    id: Option<&'a [u8]>,

    program: Option<usize>,
    channel: Option<usize>,

    octave: Option<i8>,
    transpose: Option<i8>,
    volume: Option<Decimal<'a>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Part<'a> {
    meta: Vec<MetaAttr<'a>>,

    id: Option<&'a [u8]>,

    octave: Option<i8>,
    transpose: Option<i8>,
    volume: Option<Decimal<'a>>,

    parts: Vec<Part<'a>>,

    grand_staves: Vec<GrandStave<'a>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Play<'a> {
    meta: Vec<MetaAttr<'a>>,

    voice_id: Option<&'a [u8]>,

    program: Option<usize>,
    channel: Option<usize>,

    octave: Option<i8>,
    transpose: Option<i8>,
    volume: Option<Decimal<'a>>,

    sections: Vec<Section<'a>>,
    parts: Vec<Part<'a>>,

    grand_staves: Vec<GrandStave<'a>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct GrandStave<'a> {
    staves: Vec<Stave<'a>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Stave<'a> {
    index: usize,
    key: &'a [u8],
    bars: Vec<Bar<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BarElement<'a> {
    Repeat {
        bars_to_repeat: usize,
        times_to_repeat: usize,
    },
    Rest(usize),
    Hit(usize),
    Extension(usize),
    Note {
        note: &'a [u8],
        duration: usize,
    },
    Label(&'a [u8]),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Bar<'a> {
    elements: Vec<BarElement<'a>>,
}

pub fn parse<'a>(input: &'a str, filename: Option<&str>) -> Result<(ParseTree<'a>, SourceMap)> {
    let source_map = SourceInfo::new(input, filename);

    Ok((translate(new_parse(input, filename)?), source_map))
}

enum AttrOrStaveKey<'a> {
    AttrKey(&'a [u8]),
    StaveKey(&'a [u8]),
}

fn translate<'a>(new_style: MeloFile<'a>) -> ParseTree<'a> {
    ParseTree {
        pieces: Default::default(),
    }
}

fn new_parse<'a>(input: &'a str, filename: Option<&str>) -> Result<MeloFile<'a>> {
    let mut parser = Parser::new(input);
    parser.skip_whitespace();

    let result = MeloFile(parse_section_content(&mut parser, None, vec![])?);
    assert!(parser.finished());
    Ok(result)
}

fn check_note<'a>(parser: &Parser<'a>) -> Option<&'a [u8]> {
    let mut start = parser.cursor();
    let mut parser = parser.clone_with_rules::<ExactRules>();
    let base_notes = &[
        b"a", b"b", b"c", b"d", b"e", b"f", b"g", b"A", b"B", b"C", b"D", b"E", b"F", b"G",
    ];

    base_notes
        .iter()
        .find(|&&note| parser.skip(note).is_some())?;

    parser.skip_matching(|ch| b"=_#".contains(&ch));
    parser.skip_matching(|ch| b",'".contains(&ch));
    let bytes = &parser.bytes()[start..parser.cursor()];

    match parser.check(b":").is_some() {
        true => Some(bytes),
        false => None,
    }
}

fn check_stave_key<'a>(parser: &Parser<'a>) -> Option<&'a [u8]> {
    let src = parser.source();
    let cur = parser.cursor();

    // Simple staves
    let simple_keys: &[&[u8]] = &[b"::", b"@:", b"p:", b":"];
    if let Some(simple_key) = simple_keys
        .iter()
        .find(|expected| parser.check(expected).is_some())
        .map(|found| src[cur..(cur + found.len() - 1)].as_bytes())
    {
        return Some(simple_key);
    }

    // Notes
    check_note(parser)
}

fn check_attr_or_stave_key<'a>(parser: &Parser<'a>) -> Option<AttrOrStaveKey<'a>> {
    if let Some(x) = check_stave_key(parser) {
        return Some(AttrOrStaveKey::StaveKey(x));
    }

    // Identifiers (attribute names)
    let mut parser = parser.clone();
    let ident = parser.skip_ident();
    if let (Some(ident), true) = (ident, parser.check(b":").is_some()) {
        return Some(AttrOrStaveKey::AttrKey(ident));
    }

    None
}

fn parse_usize<'a>(parser: &mut Parser<'a>) -> Result<usize> {
    let parser = parser.with_rules_mut::<LineBasedRules>();
    let value = parser.skip_matching(|ch| b"0123456789".contains(&ch));
    if value.is_empty() {
        return Err(eyre!("no integer value found"))?;
    }

    let as_str = std::str::from_utf8(&value)?;
    let as_usize: usize = as_str.parse()?;

    Ok(as_usize)
}

fn parse_section_content<'a>(
    parser: &mut Parser<'a>,
    id: Option<&'a [u8]>,
    outer_meta: Vec<MetaAttr<'a>>,
) -> Result<Section<'a>> {
    let mut section = Section {
        id,
        meta: outer_meta,
        ..Default::default()
    };

    let mut inner_meta = parse_inner_meta(parser)?;
    section.meta.append(&mut inner_meta);

    while !parser.finished() {
        if parser.check(b"}").is_some() {
            break;
        }

        // TODO: Parse metas for upcoming blocks

        // Parse a block
        let valid_blocks: &[&[u8]] = &[b"voice", b"piece", b"section", b"part", b"play"];
        let kind = valid_blocks
            .iter()
            .find(|kind| parser.skip_keyword(kind).is_some());

        if let Some(kind) = kind {
            let id = parser.skip_ident();

            // TODO: Check for ; to allow empty blocks?

            parser.expect(b"{")?;

            match kind {
                &b"voice" => section
                    .voices
                    .push(parse_voice_content(parser, id, vec![])?),
                &b"piece" => section
                    .sections
                    .push(parse_section_content(parser, id, vec![])?),
                &b"section" => section
                    .sections
                    .push(parse_section_content(parser, id, vec![])?),
                &b"part" => section.parts.push(parse_part_content(parser, id, vec![])?),
                &b"play" => section.plays.push(parse_play_content(parser, id, vec![])?),
                _ => unreachable!(),
            }

            parser.expect(b"}")?;
            continue;
        }

        if let Some(key) = check_attr_or_stave_key(parser) {
            match key {
                AttrOrStaveKey::AttrKey(key) => {
                    parser.expect(key)?;
                    parser.expect(b":")?;

                    match key {
                        b"title" => section.title = Some(parse_string(parser)?),
                        b"composer" => section.composer = Some(parse_string(parser)?),
                        b"beats" => section.beats = Some(parse_usize(parser)?),
                        b"tempo" => section.tempo = Some(parse_decimal(parser)?),
                        b"octave" => section.octave = Some(parse_i8(parser)?),
                        b"transpose" => section.transpose = Some(parse_i8(parser)?),
                        b"volume" => section.volume = Some(parse_decimal(parser)?),
                        _ => return Err(eyre!("unrecognized attribute"))?,
                    }

                    parser.expect(b"\n").or(parser.expect(b";"))?;
                }
                AttrOrStaveKey::StaveKey(_) => {
                    section
                        .plays
                        .push(parse_play_content(parser, None, vec![])?);
                }
            }
            continue;
        }

        return Err(eyre!(
            "expected end of section or block or attribute or stave"
        ))?;
    }

    Ok(section)
}

fn parse_voice_content<'a>(
    parser: &mut Parser<'a>,
    id: Option<&'a [u8]>,
    outer_meta: Vec<MetaAttr<'a>>,
) -> Result<Voice<'a>> {
    let mut voice = Voice {
        id,
        meta: outer_meta,
        ..Default::default()
    };

    let mut inner_meta = parse_inner_meta(parser)?;
    voice.meta.append(&mut inner_meta);

    while !parser.finished() {
        if parser.check(b"}").is_some() {
            break;
        }

        if let Some(key) = check_attr_or_stave_key(parser) {
            match key {
                AttrOrStaveKey::AttrKey(key) => {
                    parser.expect(key)?;
                    parser.expect(b":")?;

                    match key {
                        b"program" => voice.program = Some(parse_usize(parser)?),
                        b"channel" => voice.channel = Some(parse_usize(parser)?),
                        b"octave" => voice.octave = Some(parse_i8(parser)?),
                        b"transpose" => voice.transpose = Some(parse_i8(parser)?),
                        b"volume" => voice.volume = Some(parse_decimal(parser)?),
                        _ => return Err(eyre!("Unrecognized attribute"))?,
                    }

                    parser.expect(b"\n").or(parser.expect(b";"))?;
                }
                AttrOrStaveKey::StaveKey(_) => {
                    return Err(eyre!("cannot parse staves in voice block"))?;
                }
            }
            continue;
        }

        return Err(eyre!("expected end of voice or attribute"))?;
    }

    Ok(voice)
}

fn parse_duration<'a>(parser: &mut Parser<'a, LineBasedRules>) -> Option<usize> {
    let parser = parser.with_rules_mut::<LineBasedRules>();
    let value = parser.skip_matching(|ch| b"0123456789".contains(&ch));
    if value.is_empty() {
        return None;
    }

    let as_str = std::str::from_utf8(&value).ok()?;
    let as_usize: usize = as_str.parse().ok()?;

    Some(as_usize)
}

fn parse_element<'a>(parser: &mut Parser<'a, LineBasedRules>, requires_asterisk_for_labels: bool) -> Result<BarElement<'a>> {
    if parser.skip(b"x").is_some() {
        let duration = parse_duration(parser).unwrap_or(1);
        return Ok(BarElement::Hit(duration));
    }
    if parser.skip(b".").is_some() {
        let duration = parse_duration(parser).unwrap_or(1);
        return Ok(BarElement::Extension(duration));
    }
    if parser.skip(b"-").is_some() {
        let duration = parse_duration(parser).unwrap_or(1);
        return Ok(BarElement::Rest(duration));
    }
    if parser.skip(b"%").is_some() {
        let bars: usize;
        if parser.skip(b"(").is_some() {
            bars = parse_duration(parser).unwrap_or(1);
            parser.expect(b")")?;
        } else {
            bars = 1;
        }
        let times = parse_duration(parser).unwrap_or(1);
        return Ok(BarElement::Repeat {
            bars_to_repeat: bars,
            times_to_repeat: times,
        });
    }
    if !requires_asterisk_for_labels || parser.skip(b"*").is_some() {
        dbg!(requires_asterisk_for_labels);
        let label = parser.expect_ident()?;
        return Ok(BarElement::Label(label));
    }

    // TODO: Parse a note!

    Err(eyre!("not implemented"))
}

fn parse_stave<'a>(parser: &mut Parser<'a, LineBasedRules>, index: usize) -> Result<Stave<'a>> {
    let key = check_stave_key(parser.with_rules::<DefaultRules>()).ok_or_else(|| eyre!("No stave key found"))?;
    parser.expect(key)?;
    parser.expect(b":")?;
    parser.expect(b"|")?;

    let mut bars = vec![];
    let mut elements = vec![];

    let requires_asterisk_for_labels = !matches!(key, b":" | b"@" | b"p");

    while parser.check(b"\n").is_none() {
        if parser.skip(b"|").is_some() {
            let e = std::mem::replace(&mut elements, vec![]);
            bars.push(Bar { elements: e });
            continue;
        }

        elements.push(parse_element(parser, requires_asterisk_for_labels)?);
    }

    if !elements.is_empty() {
        bars.push(Bar { elements });
    }

    Ok(Stave {
        index,
        key,
        bars,
    })
}

fn parse_grand_stave<'a>(parser: &mut Parser<'a>) -> Result<GrandStave<'a>> {
    let mut staves = vec![];

    let parser = parser.with_rules_mut::<LineBasedRules>();
    let mut index = 0;

    while !parser.finished() {
        staves.push(parse_stave(parser, index)?);
        index += 1;

        let break_grand_stave: bool;
        if parser.skip(b";;").or(parser.check(b"}")).is_some() {
            break_grand_stave = true;
        } else {
            parser.expect(b"\n").or(parser.expect(b";"))?;
            parser.skip_whitespace();
            break_grand_stave = parser
                .skip(b"\n")
                .or(parser.skip(b";"))
                .or(parser.check(b"}"))
                .is_some();
        }

        if break_grand_stave {
            let parser = parser.with_rules_mut::<DefaultRules>();
            parser.skip_whitespace();
            break;
        }
    }

    Ok(GrandStave { staves })
}

fn parse_part_content<'a>(
    parser: &mut Parser<'a>,
    id: Option<&'a [u8]>,
    outer_meta: Vec<MetaAttr<'a>>,
) -> Result<Part<'a>> {
    let mut part = Part {
        id,
        meta: outer_meta,
        ..Default::default()
    };

    let mut inner_meta = parse_inner_meta(parser)?;
    part.meta.append(&mut inner_meta);

    while !parser.finished() {
        if parser.check(b"}").is_some() {
            break;
        }

        // TODO: Parse metas for upcoming blocks

        // Parse a block
        let valid_blocks: &[&[u8]] = &[b"voice", b"section", b"part", b"play"];
        let kind = valid_blocks
            .iter()
            .find(|kind| parser.skip_keyword(kind).is_some());

        if let Some(kind) = kind {
            let id = parser.skip_ident();

            // TODO: Check for ; to allow empty blocks?

            parser.expect(b"{")?;

            match kind {
                &b"part" => part.parts.push(parse_part_content(parser, id, vec![])?),
                _ => unreachable!(),
            }

            parser.expect(b"}")?;
            continue;
        }

        if let Some(key) = check_attr_or_stave_key(parser) {
            match key {
                AttrOrStaveKey::AttrKey(key) => {
                    parser.expect(key)?;
                    parser.expect(b":")?;

                    match key {
                        b"octave" => part.octave = Some(parse_i8(parser)?),
                        b"transpose" => part.transpose = Some(parse_i8(parser)?),
                        b"volume" => part.volume = Some(parse_decimal(parser)?),
                        _ => return Err(eyre!("unrecognized attribute")),
                    }

                    parser.expect(b"\n").or(parser.expect(b";"))?;
                }
                AttrOrStaveKey::StaveKey(_) => {
                    part.grand_staves.push(parse_grand_stave(parser)?);
                }
            }
            continue;
        }

        return Err(eyre!("expected end of part or block or attribute or stave"))?;
    }

    Ok(part)
}

fn parse_play_content<'a>(
    parser: &mut Parser<'a>,
    voice_id: Option<&'a [u8]>,
    outer_meta: Vec<MetaAttr<'a>>,
) -> Result<Play<'a>> {
    let mut play = Play {
        voice_id,
        meta: outer_meta,
        ..Default::default()
    };

    let mut inner_meta = parse_inner_meta(parser)?;
    play.meta.append(&mut inner_meta);

    while !parser.finished() {
        if parser.check(b"}").is_some() {
            break;
        }

        // TODO: Parse metas for upcoming blocks

        // Parse a block
        let valid_blocks: &[&[u8]] = &[b"section", b"part"];
        let kind = valid_blocks
            .iter()
            .find(|kind| parser.skip_keyword(kind).is_some());

        if let Some(kind) = kind {
            let id = parser.skip_ident();

            // TODO: Check for ; to allow empty blocks?

            parser.expect(b"{")?;

            match kind {
                &b"section" => play
                    .sections
                    .push(parse_section_content(parser, id, vec![])?),
                &b"part" => play.parts.push(parse_part_content(parser, id, vec![])?),
                _ => unreachable!(),
            }

            parser.expect(b"}")?;
            continue;
        }

        if let Some(key) = check_attr_or_stave_key(parser) {
            match key {
                AttrOrStaveKey::AttrKey(key) => {
                    parser.expect(key)?;
                    parser.expect(b":")?;

                    match key {
                        b"program" => play.program = Some(parse_usize(parser)?),
                        b"channel" => play.channel = Some(parse_usize(parser)?),
                        b"octave" => play.octave = Some(parse_i8(parser)?),
                        b"transpose" => play.transpose = Some(parse_i8(parser)?),
                        b"volume" => play.volume = Some(parse_decimal(parser)?),
                        _ => return Err(eyre!("Unrecognized attribute"))?,
                    }

                    parser.expect(b"\n").or(parser.expect(b";"))?;
                }
                AttrOrStaveKey::StaveKey(_) => {
                    play.grand_staves.push(parse_grand_stave(parser)?);
                }
            }
            continue;
        }

        return Err(eyre!("expected end of play or block or attribute or stave"));
    }

    Ok(play)
}

fn parse_string<'a>(parser: &mut Parser<'a>) -> Result<&'a [u8]> {
    let parser = parser.with_rules_mut::<LineBasedRules>();
    if parser.skip(b"\"").is_some() {
        parser.expect(b"\"")?;
        let value = parser.skip_inside(b'"')?;
        parser.expect(b"\"")?;
        Ok(value)
    } else if parser.skip(b"'").is_some() {
        parser.expect(b"'")?;
        let value = parser.skip_inside(b'\'')?;
        parser.expect(b"'")?;
        Ok(value)
    } else {
        Err(eyre!("not a valid string"))?
    }
}

fn parse_decimal<'a>(parser: &mut Parser<'a>) -> Result<Decimal<'a>> {
    let parser = parser.with_rules_mut::<LineBasedRules>();
    let value = parser.skip_matching(|ch| b"0123456789.".contains(&ch));
    if value.is_empty() {
        return Err(eyre!("no decimal value found"))?;
    }

    let as_str = std::str::from_utf8(&value)?;
    let _as_float: f64 = as_str.parse()?;

    Ok(Decimal(value))
}

fn parse_i8<'a>(parser: &mut Parser<'a>) -> Result<i8> {
    let parser = parser.with_rules_mut::<LineBasedRules>();
    let value = parser.skip_matching(|ch| b"0123456789-".contains(&ch));
    if value.is_empty() {
        return Err(eyre!("no integer value found"))?;
    }

    let as_str = std::str::from_utf8(&value)?;
    let as_i8: i8 = as_str.parse()?;

    Ok(as_i8)
}

fn parse_inner_meta<'a>(parser: &mut Parser<'a>) -> Result<Vec<MetaAttr<'a>>> {
    let mut metas = vec![];

    while parser.skip(b"#![").is_some() {
        let content = parser.skip_inside(b'[')?;
        parser.expect(b"]")?;

        metas.push(MetaAttr { span: content });
    }

    Ok(metas)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick() {
        let input = r##"
play {
    :: | Part1 | Part2 |
}
"##;

        panic!("{:#?}\nITS ACTAULLY FINE", new_parse(input, None).unwrap());
    }
}
