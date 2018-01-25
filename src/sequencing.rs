use parsing::data::*;
use trust::Trust;

use self::data::*;


pub mod data
{
    #[derive(Debug, PartialEq, Eq)]
    pub struct Piece<'a>
    {
        pub title: Option<&'a str>,
        pub composer: Option<&'a str>,
        pub tempo: u64,
        pub beats: u64,

        pub voices: Vec<Voice<'a>>,
    }

    impl<'a> Default for Piece<'a>
    {
        fn default() -> Self
        {
            Piece
            {
                title: None,
                composer: None,
                tempo: 120,
                beats: 4,
                voices: Vec::new(),
            }
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct Voice<'a>
    {
        pub name: &'a str,
        pub channel: u8,
        pub program: u8,
        pub octave: i8,
        pub bars: Vec<Bar>,
    }

    impl<'a> Default for Voice<'a>
    {
        fn default() -> Self
        {
            Voice
            {
                name: "error",
                channel: 1,
                program: 0,
                octave: 0,
                bars: Vec::new(),
            }
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct Bar
    {
        pub divisions: u64,
        pub notes: Vec<Note>,
    }

    impl Default for Bar
    {
        fn default() -> Self
        {
            Bar { divisions: 1, notes: Vec::new() }
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Note
    {
        pub midi: i8,
        pub position: u8,
        pub length: u8,
    }
}


#[derive(Debug, Fail, PartialEq, Eq)]
pub enum SequencingError
{
    #[fail(display = "error: {}", message)]
    InvalidNote
    {
        message: String,
    },

    #[fail(display = "error: No voice named {} was declared.", voice_name)]
    UndeclaredVoice
    {
        voice_name: String,
    },

    #[fail(display = "error: Voiceless `play` blocks are not yet supported.")]
    VoicelessPlayBlock,
}


pub fn sequence_pieces<'a>(
    piece_nodes: &[PieceNode<'a>]) -> Result<Vec<Piece<'a>>, SequencingError>
{
    use notes::lcm;

    let mut pieces = Vec::new();

    for piece_node in piece_nodes
    {
        // validation
        {
            for play in &piece_node.plays
            {
                let matched = piece_node.voices.iter().any(|voice| Some(voice.name) == play.voice);
                if !matched
                {
                    let error = match play.voice
                    {
                        Some(voice_name) => SequencingError::UndeclaredVoice { voice_name: voice_name.to_owned() },
                        None => SequencingError::VoicelessPlayBlock
                    };

                    return Err(error)
                }
            }
        }

        let mut piece = Piece::default();

        piece.title = piece_node.title.or(piece.title);
        piece.composer = piece_node.composer.or(piece.composer);
        piece.tempo = piece_node.tempo.unwrap_or(piece.tempo);
        piece.beats = piece_node.beats.unwrap_or(piece.beats);

        for voice_node in &piece_node.voices
        {
            let mut voice = Voice { name: voice_node.name, .. Default::default() };
            voice.channel = voice_node.channel.unwrap_or(voice.channel);
            voice.program = voice_node.program.unwrap_or(voice.program);
            voice.octave = voice_node.octave.unwrap_or(voice.octave);

            for play_node in &piece_node.plays
            {
                if play_node.voice != Some(voice.name)
                {
                    continue
                }

                for stave_node in &play_node.staves
                {
                    for (index, bar_node) in stave_node.bars.iter().enumerate()
                    {
                        if index >= voice.bars.len()
                        {
                            voice.bars.push(Bar::default())
                        }

                        let bar = &mut voice.bars[index];
                        let bar_node_length = bar_node.notes.len() as u64;

                        let lcm = lcm(bar.divisions, bar_node_length);
                        let bar_scale = (lcm / bar.divisions) as u64;
                        let note_scale = (lcm / bar_node_length) as u8;

                        bar.divisions *= bar_scale;
                        for note in &mut bar.notes
                        {
                            note.length *= bar_scale as u8;
                            note.position *= bar_scale as u8;
                        }

                        for (note_index, note_node) in bar_node.notes.iter().enumerate()
                        {
                            match *note_node
                            {
                                NoteNode::Rest => continue,
                                NoteNode::Note(midi) => {
                                    let octave = voice.octave;

                                    let midi = midi.checked_add(octave * 12)
                                        .ok_or_else(
                                            ||
                                            {
                                                use notes;

                                                let sharp = notes::midi_to_sharp(midi).trust();
                                                let flat = notes::midi_to_flat(midi).trust();
                                                let (direction, offset) = match octave
                                                {
                                                    o if o > 0 => ("up", o),
                                                    o => ("down", -o)
                                                };
                                                let message = format!(
                                                    "Note ({} / {}) is invalid after shifting {} {} octaves. Notes must lie between {} and {}.",
                                                    flat, sharp, direction, offset,
                                                    notes::MIN_SHARP, notes::MAX_SHARP);

                                                SequencingError::InvalidNote
                                                {
                                                    message
                                                }
                                            })?;

                                    let length = 1 * note_scale;
                                    let position = note_index as u8 * note_scale;
                                    let note = Note { midi, length, position };

                                    bar.notes.push(note);
                                }
                            }
                        }
                    }
                }

                for bar in &mut voice.bars
                {
                    bar.notes.sort_by_key(|note| note.position);
                }
            }

            piece.voices.push(voice);
        }

        pieces.push(piece);
    }

    Ok(pieces)
}


#[cfg(test)]
mod tests
{
    use super::*;
    use lexing;
    use parsing;


    fn sequence_test(source: &str, expected: Piece)
    {
        let tokens = &lexing::lex(source).expect("ERROR IN LEXER");
        let parse_tree = parsing::parse(tokens).expect("ERROR IN PARSER");
        let piece = &sequence_pieces(&parse_tree.pieces).unwrap()[0];
        assert_eq!(piece, &expected);
    }

    fn sequence_test_fail(source: &str)
    {
        let tokens = &lexing::lex(source).expect("ERROR IN LEXER");
        let parse_tree = parsing::parse(tokens).expect("ERROR IN PARSER");
        assert!(sequence_pieces(&parse_tree.pieces).is_err());
    }

    fn voice_test(source: &str, expected_bars: Vec<Bar>)
    {
        let tokens = &lexing::lex(source).expect("ERROR IN LEXER");
        let parse_tree = parsing::parse(tokens).expect("ERROR IN PARSER");
        let piece = &sequence_pieces(&parse_tree.pieces).unwrap()[0];
        assert_eq!(piece.voices[0].bars, expected_bars);
    }

    #[test]
    fn sequence_empty_piece()
    {
        sequence_test("", Piece::default());
    }

    #[test]
    fn piece_with_attributes()
    {
        sequence_test("piece { title: One, composer: Two, tempo: 3, beats: 4 }",
            Piece
            {
                title: Some("One"),
                composer: Some("Two"),
                tempo: 3,
                beats: 4,
                .. Default::default()
            });
    }

    #[test]
    fn piece_with_empty_voice()
    {
        sequence_test("voice Empty { }",
            Piece
            {
                voices: vec![Voice { name: "Empty", .. Default::default() }],
                .. Default::default()
            });
    }

    #[test]
    fn voice_with_mismatched_play()
    {
        sequence_test_fail("voice OneNote { } play Different { :| C }");
    }

    #[test]
    fn voice_with_single_note()
    {
        voice_test("voice OneNote { } play OneNote { :| C }",
           vec![
               Bar
               {
                   divisions: 1,
                   notes: vec![Note { midi: 60, length: 1, position: 0 }]
               }
           ]);
    }

    #[test]
    fn voice_with_two_notes()
    {
        voice_test("voice TwoNote { } play TwoNote { :| C G }",
           vec![
               Bar
               {
                   divisions: 2,
                   notes: vec![
                       Note { midi: 60, length: 1, position: 0 },
                       Note { midi: 67, length: 1, position: 1 },
                   ]
               }
           ]);
    }

    #[test]
    fn voice_with_two_staves()
    {
        voice_test("voice Diad { } play Diad { :| C ; :| G }",
           vec![
               Bar
               {
                   divisions: 1,
                   notes: vec![
                       Note { midi: 60, length: 1, position: 0 },
                       Note { midi: 67, length: 1, position: 0 },
                   ]
               }
           ]);
    }

    #[test]
    fn threes_against_twos()
    {
        voice_test("voice Diad { } play Diad { :| C E G ; :| c g }",
           vec![
               Bar
               {
                   divisions: 6,
                   notes: vec![
                       Note { midi: 60, length: 2, position: 0 },
                       Note { midi: 72, length: 3, position: 0 },
                       Note { midi: 64, length: 2, position: 2 },
                       Note { midi: 79, length: 3, position: 3 },
                       Note { midi: 67, length: 2, position: 4 },
                   ]
               }
           ]);
    }

    #[test]
    fn fail_when_notes_moved_out_of_range()
    {
        sequence_test_fail("voice V { octave: 1} play V { :| g^'''}");
    }
}
