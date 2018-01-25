#[cfg(test)]
#[macro_use] extern crate pretty_assertions;

#[macro_use] extern crate failure;
#[macro_use] extern crate lazy_static;

extern crate regex;

pub mod notes;
mod abc_generation;
pub mod lexing;     // TODO(***realname***): These should be private
pub mod parsing;    // TODO(***realname***): These should be private
mod sequencing;
mod trust;

#[cfg(test)]
mod test_helpers;


use failure::Error;


pub fn compile_to_abc(input: &str) -> Result<String, Error>
{
    let tokens = lexing::lex(input)?;
    let parse_tree = parsing::parse(&tokens)?;
    let pieces = sequencing::sequence_pieces(&parse_tree.pieces)?;
    let abc = abc_generation::generate_abc(&pieces)?;

    Ok(abc)
}

