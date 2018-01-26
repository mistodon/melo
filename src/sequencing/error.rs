use std::fmt::{ Display, Formatter, Error };


#[derive(Debug, Fail, PartialEq, Eq)]
pub struct SequencingError
{
    pub line: usize,
    pub col: usize,
    pub error: ErrorType,
}


#[derive(Debug, PartialEq, Eq)]
pub enum ErrorType
{
    InvalidNote
    {
        midi: i8,
        octave_offset: i8,
    },

    UndeclaredVoice
    {
        voice_name: String,
    },

    VoicelessPlayBlock,
}


impl Display for SequencingError
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>
    {
        use ansi_term::Color;
        use self::ErrorType::*;

        let error_message = match self.error
        {
            InvalidNote { midi, octave_offset } => {
                use notes::{ self, MIN_SHARP, MAX_SHARP };
                use trust::Trust;

                let note = notes::midi_to_sharp(midi).trust();
                let dir = if octave_offset <= 0 { "down" } else { "up" };
                let oct = octave_offset.abs();

                format!("Note `{}` is invalid after being shifted {} {} octaves. Notes must lie between `{}` and `{}`.", note, dir, oct, MIN_SHARP, MAX_SHARP)
            }
            UndeclaredVoice { ref voice_name } =>
                format!("No voice named `{}` was declared.", voice_name),
            VoicelessPlayBlock => "Voiceless `play` blocks are not yet supported.".to_owned(),
        };

        writeln!(f, "{}: {}",
            Color::Fixed(9).paint(format!("error:{}:{}", self.line, self.col)),
            Color::Fixed(15).paint(error_message))
    }
}

