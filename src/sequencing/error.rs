use error::{self, SourceLoc};
use std::fmt::{Display, Error, Formatter};


#[derive(Debug, Fail, PartialEq, Eq)]
pub struct SequencingError
{
    pub loc: SourceLoc,
    pub error: ErrorType,
}


#[derive(Debug, PartialEq, Eq)]
pub enum ErrorType
{
    InvalidNote
    {
        octave_offset: i8,
    },

    UndeclaredVoice
    {
        voice_name: String,
    },

    VoicelessPlayBlock,

    NothingToRepeat,
}


impl Display for SequencingError
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>
    {
        use self::ErrorType::*;

        let error_message = match self.error
        {
            InvalidNote { octave_offset } =>
            {
                use notes::{MAX_SHARP, MIN_SHARP};

                let dir = if octave_offset <= 0 { "down" } else { "up" };
                let oct = octave_offset.abs();

                format!("Note `{}` is invalid after being shifted {} {} octaves. Notes must lie between `{}` and `{}`.",
                        self.loc.text(),
                        dir,
                        oct,
                        MIN_SHARP,
                        MAX_SHARP)
            }

            UndeclaredVoice { ref voice_name } =>
            {
                format!("No voice named `{}` was declared.", voice_name)
            }

            VoicelessPlayBlock => "Voiceless `play` blocks are not yet supported.".to_owned(),

            NothingToRepeat => "There is no previous bar to repeat.".to_owned(),
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
