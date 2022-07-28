mod error;
mod lexing;
mod midi_generation;
pub mod notes;
mod parsing;
mod parsing_old;
mod sequencing;
mod trust;

#[cfg(test)]
mod test_helpers;

pub use color_eyre::eyre::{eyre, Result};

pub use crate::{
    error::colors,
    midi_generation::data::MidiGenerationOptions,
};

pub fn compile_to_midi(
    input: &str,
    filename: Option<&str>,
    options: &MidiGenerationOptions,
) -> Result<Vec<u8>> {
    // let (tokens, source_map) = lexing::lex(input, filename)?;
    // let parse_tree = parsing_old::parse(&tokens, &source_map)?;
    let (parse_tree, source_map) = parsing::parse(input, filename)?;
    let pieces = sequencing::sequence_pieces(&parse_tree, &source_map)?;
    let midi = midi_generation::generate_midi(
        pieces.get(0).ok_or_else(|| eyre!("No pieces found"))?,
        &source_map,
        options,
    )
    .ok_or_else(|| eyre!("Compilation to MIDI failed!"))?;

    Ok(midi)
}
