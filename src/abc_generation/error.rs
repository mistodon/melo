use std::fmt::{ Display, Formatter, Error };


#[derive(Debug, Fail, PartialEq, Eq)]
pub struct AbcGenerationError
{
    pub line: usize,
    pub col: usize,
    pub error: ErrorType,
}


#[derive(Debug, PartialEq, Eq)]
pub enum ErrorType
{
    FormattingError
    {
        error: Error,
    },

    UnsupportedTuplet
    {
        tuplet: u64
    }
}

impl From<Error> for AbcGenerationError
{
    fn from(error: Error) -> Self { AbcGenerationError { line: 0, col: 0, error: ErrorType::FormattingError { error } } }
}


impl Display for AbcGenerationError
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>
    {
        use ansi_term::Color;
        use self::ErrorType::*;

        let error_message = match self.error
        {
            FormattingError { error } =>
                format!("Internal error in formatting: {}", error),
            UnsupportedTuplet { tuplet } =>
                format!("Piece requires a tuplet of {} notes, but only tuplets of 3 to 9 notes are currently supported.", tuplet),
        };

        // TODO(***realname***): Don't show line/col for formatting errors
        writeln!(f, "{}: {}",
            Color::Fixed(9).paint(format!("error:{}:{}", self.line, self.col)),
            Color::Fixed(15).paint(error_message))
    }
}

