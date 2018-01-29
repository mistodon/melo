#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;

extern crate ansi_term;
extern crate regex;


pub mod notes;
mod abc_generation;
mod error;
mod lexing;
mod parsing;
mod sequencing;
mod trust;

#[cfg(test)]
mod test_helpers;


use failure::Error;


pub fn compile_to_abc(input: &str, filename: Option<&str>) -> Result<String, Error>
{
    let (tokens, source_map) = lexing::lex(input, filename)?;
    let parse_tree = parsing::parse(&tokens, &source_map)?;
    let pieces = sequencing::sequence_pieces(&parse_tree, &source_map)?;
    let abc = abc_generation::generate_abc(&pieces, &source_map)?;

    Ok(abc)
}
