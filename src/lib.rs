mod error;
mod lexing;
mod midi_generation;
pub mod notes;
mod parsing;
mod sequencing;
mod trust;

#[cfg(test)]
mod test_helpers;

pub use eyre::{eyre, Result};

pub use crate::error::colors;
pub use crate::midi_generation::data::MidiGenerationOptions;

pub fn compile_to_midi(
    input: &str,
    filename: Option<&str>,
    options: &MidiGenerationOptions,
) -> Result<Vec<u8>> {
    let (tokens, source_map) = lexing::lex(input, filename)?;
    let parse_tree = parsing::parse(&tokens, &source_map)?;
    let pieces = sequencing::sequence_pieces(&parse_tree, &source_map)?;
    let midi = midi_generation::generate_midi(
        pieces.get(0).ok_or_else(|| eyre!("No pieces found"))?,
        &source_map,
        options,
    )
    .ok_or_else(|| eyre!("Compilation to MIDI failed!"))?;

    Ok(midi)
}
