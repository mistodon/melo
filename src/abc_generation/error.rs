use std::fmt::{Display, Error, Formatter};

use error;


#[derive(Debug, Fail, PartialEq, Eq)]
pub struct AbcGenerationError
{
    pub error: ErrorType,
}


#[derive(Debug, PartialEq, Eq)]
pub enum ErrorType
{
    FormattingError
    {
        error: Error
    },

    UnsupportedTuplet
    {
        tuplet: u32
    },
}

impl From<Error> for AbcGenerationError
{
    fn from(error: Error) -> Self
    {
        AbcGenerationError {
            error: ErrorType::FormattingError { error },
        }
    }
}


impl Display for AbcGenerationError
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>
    {
        use self::ErrorType::*;

        let error_message = match self.error
        {
            FormattingError { error } =>
                format!("Internal formatting error: {}", error),
            UnsupportedTuplet { tuplet } =>
                format!("Piece requires a tuplet of {} notes, but only tuplets of 3 to 9 notes are currently supported.", tuplet),
        };

        error::fmt_simple_error(
            f,
            &error_message,
            "<file>",
        )
    }
}

