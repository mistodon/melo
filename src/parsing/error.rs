use std::fmt::{Display, Error, Formatter};

use lexing::data::MetaToken;


#[derive(Debug, Fail, PartialEq, Eq)]
pub struct ParsingError
{
    pub line: usize,
    pub col: usize,
    pub error: ErrorType,
}


#[derive(Debug, PartialEq, Eq)]
pub enum ErrorType
{
    UnexpectedToken
    {
        token: String,
        context: &'static str,
        expected: String,
    },

    UnexpectedEOF
    {
        context: &'static str,
        expected: String,
    },

    InvalidNote
    {
        note: String
    },

    InvalidHit
    {
        stave_prefix: String
    },

    InvalidAttribute
    {
        attribute: String,
        structure: &'static str,
    },

    UndeclaredStave
    {
        stave_prefix: String
    },

    InvalidLength
    {
        length: i64
    },

    UnexpectedLength
    {
        length: i64
    },

    MultipleParsingErrors
    {
        errors: Vec<ParsingError>
    },
}


impl Display for ParsingError
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>
    {
        use self::ErrorType::*;
        use ansi_term::Color;

        if let MultipleParsingErrors { ref errors } = self.error
        {
            for error in errors
            {
                write!(f, "{}", error)?;
            }

            if errors.len() > 1
            {
                writeln!(
                    f,
                    "{}: {}",
                    Color::Fixed(9).paint("error"),
                    Color::Fixed(15)
                        .paint(format!("Aborting due to {} previous errors.", errors.len()))
                )?;
            }

            Ok(())
        }
        else
        {
            use notes::{MAX_SHARP, MIN_SHARP};

            let error_message = match self.error
            {
                UnexpectedToken { ref token, context, ref expected } =>
                    format!("Unexpected token `{}` {}. Expected {}.", token, context, expected),

                UnexpectedEOF { context, ref expected } =>
                    format!("Unexpected end of input {}. Expected {}.", context, expected),

                InvalidNote { ref note } =>
                    format!("Note `{}` is out of range. Must be between `{}` and `{}`.",
                            note, MIN_SHARP, MAX_SHARP),

                InvalidHit { ref stave_prefix } =>
                    format!("Hit markers (`x`) cannot be used in `{}:` staves. They are only valid in single-note staves.", stave_prefix),

                // TODO(***realname***): which ones are valid?
                InvalidAttribute { ref attribute, structure } =>
                    format!("Invalid attribute `{}` for `{}`.",
                        attribute, structure),

                UndeclaredStave { ref stave_prefix } =>
                    format!("The `{}:` stave wasn't declared at the start of the `play` block. All staves must be declared before the first blank line.", stave_prefix),

                InvalidLength { ref length } =>
                    format!("Invalid note length `{}`. Lengths must be between 0 and 255.", length),

                UnexpectedLength { ref length } =>
                    format!("Unexpected note length `{}`. Lengths must follow a note or rest.", length),

                _ => unreachable!()
            };

            writeln!(
                f,
                "{}: {}",
                Color::Fixed(9).paint(format!("error:{}:{}", self.line, self.col)),
                Color::Fixed(15).paint(error_message)
            )
        }
    }
}

impl ParsingError
{
    pub fn eof(eof_token: &MetaToken, context: &'static str, expected: String)
        -> ParsingError
    {
        let (line, col) = (eof_token.line, eof_token.col);
        ParsingError {
            line,
            col,
            error: ErrorType::UnexpectedEOF { context, expected },
        }
    }

    pub fn unexpected(
        token: &MetaToken,
        context: &'static str,
        expected: String,
    ) -> ParsingError
    {
        let (line, col) = (token.line, token.col);
        ParsingError {
            line,
            col,
            error: ErrorType::UnexpectedToken {
                token: token.span.1.to_owned(),
                context,
                expected,
            },
        }
    }
}

impl From<Vec<ParsingError>> for ParsingError
{
    fn from(errors: Vec<ParsingError>) -> Self
    {
        ParsingError {
            line: 0,
            col: 0,
            error: ErrorType::MultipleParsingErrors { errors },
        }
    }
}
