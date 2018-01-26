use std::fmt::{ Display, Formatter, Error };


#[derive(Debug, Fail, PartialEq, Eq)]
pub struct LexingError
{
    pub line: usize,
    pub col: usize,
    pub error: ErrorType,
}


#[derive(Debug, PartialEq, Eq)]
pub enum ErrorType
{
    UnexpectedCharacter
    {
        text: String,
        context: &'static str,
    },
}


impl Display for LexingError
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>
    {
        use ansi_term::Color;
        use self::ErrorType::*;

        let error_message = match self.error
        {
            UnexpectedCharacter { ref text, ref context } => {
                format!("Unexpected character `{}` in {}.", text, context)
            }
        };

        writeln!(f, "{}: {}",
            Color::Fixed(9).paint(format!("error:{}:{}", self.line, self.col)),
            Color::Fixed(15).paint(error_message))
    }
}

