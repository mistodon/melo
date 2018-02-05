use error::{self, SourceLoc};
use lexing::data::MetaToken;
use std::fmt::{Display, Error, Formatter};


#[derive(Debug, Fail, PartialEq, Eq)]
pub struct ParsingError
{
    pub loc: SourceLoc,
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

    InvalidOctave
    {
        octave: i64
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

    ExcessNotesInRepeatBar
    {
        placement: &'static str
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

        if let MultipleParsingErrors { ref errors } = self.error
        {
            for error in errors
            {
                write!(f, "{}", error)?;
            }

            if errors.len() > 1
            {
                error::fmt_simple_error(
                    f,
                    &format!("Aborting due to {} previous errors.", errors.len()),
                    self.loc.info.filename(),
                )?;
            }

            Ok(())
        }
        else
        {
            use notes::{MAX_SHARP, MIN_SHARP};

            let error_message = match self.error
            {
                UnexpectedToken {
                    ref token,
                    context,
                    ref expected,
                } =>
                {
                    format!("Unexpected token `{}` {}. Expected {}.",
                            token,
                            context,
                            expected)
                }

                UnexpectedEOF {
                    context,
                    ref expected,
                } =>
                {
                    format!("Unexpected end of input {}. Expected {}.",
                            context,
                            expected)
                }

                InvalidNote { ref note } =>
                {
                    format!("Note `{}` is out of range. Must be between `{}` and `{}`.",
                            note,
                            MIN_SHARP,
                            MAX_SHARP)
                }

                InvalidHit { ref stave_prefix } =>
                {
                    format!("Hit markers (`x`) cannot be used in `{}:` staves. They are only valid in single-note staves.",
                            stave_prefix)
                }

                // TODO(***realname***): which ones are valid?
                InvalidAttribute {
                    ref attribute,
                    structure,
                } => format!("Invalid attribute `{}` for `{}`.", attribute, structure),

                InvalidOctave { octave } =>
                {
                    format!("Invalid octave `{}` would throw every note out of range. Must be in the range [-10, 10].",
                            octave)
                }

                UndeclaredStave { ref stave_prefix } =>
                {
                    format!("The `{}:` stave wasn't declared at the start of the `play` block. All staves must be declared before the first blank line.",
                            stave_prefix)
                }

                InvalidLength { ref length } =>
                {
                    format!("Invalid note length `{}`. Lengths must be between 0 and 255.",
                            length)
                }

                UnexpectedLength { ref length } =>
                {
                    format!("Unexpected note length `{}`. Lengths must follow a note or rest.",
                            length)
                }

                ExcessNotesInRepeatBar { placement } =>
                {
                    format!("Unexpected notes {} repeat sign `%`. Bars with repeat signs should contain nothing else.", placement)
                }

                _ => unreachable!(),
            };

            error::fmt_error(
                f,
                &error_message,
                self.loc.info.filename(),
                self.loc.cause_line(),
                self.loc.line,
                self.loc.col,
                self.loc.width,
            )
        }
    }
}

impl ParsingError
{
    pub fn eof(eof_token: &MetaToken, context: &'static str, expected: String)
        -> ParsingError
    {
        ParsingError {
            loc: eof_token.loc.clone(),
            error: ErrorType::UnexpectedEOF { context, expected },
        }
    }

    pub fn unexpected(
        token: &MetaToken,
        context: &'static str,
        expected: String,
    ) -> ParsingError
    {
        let text = token.span.1.to_owned();
        let text = if text.trim().is_empty()
        {
            token.token.readable_type().into()
        }
        else
        {
            text
        };

        ParsingError {
            loc: token.loc.clone(),
            error: ErrorType::UnexpectedToken {
                token: text,
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
            loc: errors[0].loc.clone(),
            error: ErrorType::MultipleParsingErrors { errors },
        }
    }
}
