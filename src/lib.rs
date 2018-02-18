#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;

extern crate ansi_term;
extern crate regex;
extern crate rimd;


pub mod notes;
mod abc_generation;
mod error;
mod lexing;
mod midi_generation;
mod parsing;
mod sequencing;
mod trust;

#[cfg(test)]
mod test_helpers;

pub use midi_generation::data::MidiGenerationOptions;

use failure::Error;
use error::SourceMap;


#[deprecated]
pub fn compile_to_abc(input: &str, filename: Option<&str>) -> Result<String, Error>
{
    let source_map = &SourceMap::new(filename.unwrap_or("<file>").to_owned(), input);
    let (tokens, source_info) = lexing::lex(input, filename, source_map)?;
    let parse_tree = parsing::parse(&tokens, &source_info)?;
    let pieces = sequencing::sequence_pieces(&parse_tree, &source_info)?;
    let abc = abc_generation::generate_abc(&pieces, &source_info)?;

    Ok(abc)
}


pub fn compile_to_midi(
    input: &str,
    filename: Option<&str>,
    options: &MidiGenerationOptions,
) -> Result<Vec<u8>, Error>
{
    let source_map = &SourceMap::new(filename.unwrap_or("<file>").to_owned(), input);
    let (tokens, source_info) = lexing::lex(input, filename, source_map)?;
    let parse_tree = parsing::parse(&tokens, &source_info)?;
    let pieces = sequencing::sequence_pieces(&parse_tree, &source_info)?;
    let midi = midi_generation::generate_midi(
        pieces.get(0).ok_or(failure::err_msg("No pieces found"))?,
        &source_info,
        options,
    ).ok_or(failure::err_msg("Compilation to MIDI failed!"))?;

    Ok(midi)
}
