use crate::error::{self, SourceLoc};
use std::fmt::{Display, Error, Formatter};

use failure::Fail;

#[derive(Debug, Fail, PartialEq, Eq)]
pub struct LexingError {
    pub loc: SourceLoc,
    pub error: ErrorType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorType {
    UnexpectedCharacter { text: String, context: &'static str },
}

impl Display for LexingError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        use self::ErrorType::*;

        let error_message = match self.error {
            UnexpectedCharacter { ref text, context } => {
                format!("Unexpected character `{}` in {}.", text, context)
            }
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
