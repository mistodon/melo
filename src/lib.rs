#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate lazy_static;

mod abc_generation;
mod error;
mod lexing;
mod midi_generation;
pub mod notes;
pub mod parse;
mod parsing;
mod sequencing;
mod trust;

#[cfg(test)]
mod test_helpers;

pub use crate::error::colors;
pub use crate::midi_generation::data::MidiGenerationOptions;
pub use failure::Error;

#[deprecated]
pub fn compile_to_abc(input: &str, filename: Option<&str>) -> Result<String, Error> {
    let (tokens, source_map) = lexing::lex(input, filename)?;
    let parse_tree = parsing::parse(&tokens, &source_map)?;
    let pieces = sequencing::sequence_pieces(&parse_tree, &source_map)?;
    let abc = abc_generation::generate_abc(&pieces, &source_map)?;

    Ok(abc)
}

pub fn compile_to_midi(
    input: &str,
    filename: Option<&str>,
    options: &MidiGenerationOptions,
) -> Result<Vec<u8>, Error> {
    let (tokens, source_map) = lexing::lex(input, filename)?;
    let parse_tree = parsing::parse(&tokens, &source_map)?;

    let test_parse_tree = parse::parse(input, filename)?;
    println!("Parse tree:\n{:#?}", test_parse_tree);

    let pieces = sequencing::sequence_pieces(&parse_tree, &source_map)?;
    let midi = midi_generation::generate_midi(
        pieces
            .get(0)
            .ok_or_else(|| failure::err_msg("No pieces found"))?,
        &source_map,
        options,
    )
    .ok_or_else(|| failure::err_msg("Compilation to MIDI failed!"))?;

    Ok(midi)
}
