use std::fmt::{Display, Result, Formatter, Write};

use error::{self, SourceLoc};
use formatting::{MultiFormat, StyleType};


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


impl<'a> MultiFormat<StyleType<'a>> for LexingError
{
    fn multi_fmt<W: Write>(&self, f: &mut W, style_type: &StyleType) -> Result
    {
        use self::ErrorType::*;

        let error_message = match self.error
        {
            UnexpectedCharacter { ref text, context } =>
            {
                format!("Unexpected character `{}` in {}.", text, context)
            }
        };

        error::fmt_error_multi(
            f,
            style_type,
            &error_message,
            self.loc.info.filename(),
            self.loc.cause_line(),
            self.loc.line,
            self.loc.col,
            self.loc.width,
        )
    }
}

impl Display for LexingError
{
    fn fmt(&self, f: &mut Formatter) -> Result
    {
        self.multi_fmt(f, &StyleType::Normal)
    }
}
