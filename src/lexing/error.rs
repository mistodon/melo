use std::fmt::{Display, Error, Formatter};

use error::SourceLoc;


#[derive(Debug, Fail, PartialEq, Eq)]
pub struct LexingError
{
    pub loc: SourceLoc,
    pub error: ErrorType,
}


#[derive(Debug, PartialEq, Eq)]
pub enum ErrorType
{
    UnexpectedCharacter
    {
        text: String, context: &'static str
    },
}


impl Display for LexingError
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>
    {
        use self::ErrorType::*;
        use ansi_term::Color;

        let error_message = match self.error
        {
            UnexpectedCharacter { ref text, context } =>
            {
                format!("Unexpected character `{}` in {}.", text, context)
            }
        };

        let error_label = Color::Fixed(9).paint("error");
        let error_message = Color::Fixed(15).paint(error_message);
        let arrow = Color::Fixed(12).paint("-->");
        let filename = self.loc.filename_or("<stdin>");
        let (line, col) = self.loc.line_col();
        let location = self.loc.cause_line();
        let underline = Color::Fixed(12).paint(self.loc.under_line());
        let line_prefix = format!("{} |    ", line);
        let underline_indent = " ".repeat(line_prefix.len());
        let line_prefix = Color::Fixed(12).paint(line_prefix);

        writeln!(
            f,
            "{}: {}\n   {} {}:{}:{}\n\n{}{}\n{}{}",
            error_label,
            error_message,
            arrow,
            filename,
            line,
            col,
            line_prefix,
            location,
            underline_indent,
            underline
        )
    }
}
