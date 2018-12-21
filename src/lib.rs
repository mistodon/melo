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

pub fn compile_to_midi(
    input: &str,
    filename: Option<&str>,
    options: &MidiGenerationOptions,
) -> Result<Vec<u8>, Error> {
    let (_, source_map) = lexing::lex(input, filename)?;
    let parse_tree = parse::parse(input, filename)?;

    let pieces = sequencing::sequence_pieces(&parse_tree)?;
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
