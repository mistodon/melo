use error::{self, SourceLoc, SourceMap};
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
        tuplet: u32,
        cause_location: Option<SourceLoc>,
        divisions: Option<u32>,
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

        let filename = match self.info
        {
            Some(ref info) => info.filename(),
            None => "<file>",
        };

        match self.error
        {
            FormattingError { error } => error::fmt_simple_error(
                f,
                &format!("Internal formatting error: {}", error),
                filename,
            ),

            UnsupportedTuplet {
                tuplet,
                ref cause_location,
                divisions,
            } =>
            {
                let main_message = format!("Piece requires a tuplet of {} notes, but only tuplets of 3 to 9 notes are currently supported.", tuplet);

                match *cause_location
                {
                    Some(ref loc) => error::fmt_error(
                        f,
                        &format!(
                            "{}\n       This tuplet is required because of the following bar, which contains {} divisions.",
                            main_message,
                            divisions.unwrap_or(0),
                        ),
                        loc.info.filename(),
                        loc.cause_line(),
                        loc.line,
                        loc.col,
                        loc.width,
                    ),

                    None => error::fmt_simple_error(f, &main_message, filename),
                }
            }
        }
    }
}
