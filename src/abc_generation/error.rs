use error::{self, SourceMap};
use std::fmt::{Display, Error, Formatter};


#[derive(Debug, Fail)]
pub struct AbcGenerationError
{
    pub info: Option<SourceMap>,
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


pub fn fmt_err(error: Error, info: Option<SourceMap>) -> AbcGenerationError
{
    AbcGenerationError {
        info,
        error: ErrorType::FormattingError { error },
    }
}


impl Display for AbcGenerationError
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>
    {
        use self::ErrorType::*;

        let error_message = match self.error
        {
            FormattingError { error } => format!("Internal formatting error: {}", error),
            UnsupportedTuplet { tuplet } =>
            {
                format!("Piece requires a tuplet of {} notes, but only tuplets of 3 to 9 notes are currently supported.",
                        tuplet)
            }
        };

        let filename = match self.info
        {
            Some(ref info) => info.filename(),
            None => "<file>",
        };

        error::fmt_simple_error(f, &error_message, filename)
    }
}
