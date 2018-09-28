#[derive(Debug)]
pub struct MidiGenerationOptions {
    pub ticks_per_beat: i16,
}

impl Default for MidiGenerationOptions {
    fn default() -> Self {
        MidiGenerationOptions {
            ticks_per_beat: 480,
        }
    }
}
