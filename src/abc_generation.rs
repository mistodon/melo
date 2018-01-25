use std::fmt::Error;

use sequencing::data::*;
use notes;
use trust::Trust;


#[derive(Debug, Fail, PartialEq, Eq)]
pub enum AbcGenerationError
{
    #[fail(display = "error: Error in formatting: {}", error)]
    FormattingError
    {
        #[cause]
        error: Error,
    },

    #[fail(display = "error: Piece requires a {}-tuplet, but only tuplets of 3-9 notes are currently supported.", tuplet)]
    UnsupportedTuplet
    {
        tuplet: u64
    }
}

impl From<Error> for AbcGenerationError
{
    fn from(error: Error) -> Self { AbcGenerationError::FormattingError { error } }
}


fn div_tuplet(notes_per_beat: u64) -> (u64, u64)
{
    let mut tuplet = notes_per_beat;
    let mut division = 1;

    while tuplet > 0 && tuplet % 2 == 0
    {
        tuplet /= 2;
        division *= 2;
    }

    let division = if tuplet < 3 { division * 4 } else { division * 8 };

    (division, tuplet)
}


pub fn generate_abc(pieces: &[Piece]) -> Result<String, AbcGenerationError>
{
    use std::fmt::Write;

    let mut buffer = String::new();

    for (index, piece) in pieces.iter().enumerate()
    {
        writeln!(buffer, "X:{}", index + 1)?;

        if let Some(title) = piece.title
        {
            writeln!(buffer, "T:{}", title)?;
        }
        if let Some(composer) = piece.composer
        {
            writeln!(buffer, "C:{}", composer)?;
        }

        writeln!(buffer, "M:{}/4", piece.beats)?;
        writeln!(buffer, "Q:1/4={}", piece.tempo)?;
        writeln!(buffer, "K:C")?;

        for voice in &piece.voices
        {
            writeln!(buffer, "V:{}", voice.name)?;
            writeln!(buffer, "%%MIDI channel {}", voice.channel)?;
            writeln!(buffer, "%%MIDI program {}", voice.program)?;

            if !voice.bars.is_empty()
            {
                let stave_text = write_bars(&voice.bars, piece.beats)?;

                write!(buffer, "{}", stave_text)?;
            }
        }
    }

    writeln!(buffer)?;

    Ok(buffer)
}


fn write_bars(bars: &[Bar], beats_per_bar: u64) -> Result<String, AbcGenerationError>
{
    use std::fmt::Write;
    use notes::lcm;

    let mut buffer = String::new();

    let max_divisions = bars.iter().map(|bar| bar.divisions).max().trust();
    let notes_per_bar = lcm(max_divisions, beats_per_bar);
    let notes_per_beat = notes_per_bar / beats_per_bar;
    let (beat_division, tuplet) = div_tuplet(notes_per_beat);

    if tuplet > 9
    {
        return Err(AbcGenerationError::UnsupportedTuplet { tuplet })
    }

    writeln!(buffer, "L:1/{}", beat_division)?;

    for bar in bars
    {
        let scale = notes_per_bar / bar.divisions;

        let mut notes = bar.notes.iter();
        let mut cursor: u64 = 0;
        let mut abc_notes = vec![];

        loop
        {
            let note = notes.next();
            let position = note.map(|note| note.position as u64 * scale).unwrap_or(notes_per_bar);

            if position < cursor
            {
                continue
            }

            let rest_length = position - cursor;

            if rest_length > 0
            {
                let individual_rest_length =  if tuplet == 1 { rest_length } else { 1 };
                let number_of_rests = rest_length / individual_rest_length;
                let rest_string = match individual_rest_length
                {
                    1 => "z".to_owned(),
                    n => format!("z{}", n),
                };

                for _ in 0..number_of_rests
                {
                    abc_notes.push(rest_string.clone());
                }

                cursor = position;
            }


            match note
            {
                Some(note) => {
                    let chord: Vec<Note> = bar.notes.iter()
                        .filter(|other| other.position == note.position)
                        .map(|&note| note)
                        .collect();

                    let min_chord_length = chord.iter().map(|note| note.length as u64 * scale).min().trust();
                    let individual_chord_length = if tuplet == 1 { min_chord_length } else { 1 };
                    let number_of_chords = min_chord_length / individual_chord_length;

                    let chord_notes_string = chord.iter()
                        .map(|note| notes::midi_to_abc(note.midi).trust())
                        .collect::<Vec<&str>>()
                        .join("");

                    let chord_string = match chord.len()
                    {
                        1 => chord_notes_string,
                        _ => format!("[{}]", chord_notes_string)
                    };

                    let chord_string = match individual_chord_length
                    {
                        1 => format!("{}", chord_string),
                        n => format!("{}{}", chord_string, n)
                    };

                    let tied_chord_string = format!("{}-", chord_string);

                    for _ in 0..(number_of_chords - 1)
                    {
                        abc_notes.push(tied_chord_string.clone())
                    }
                    abc_notes.push(chord_string);

                    cursor += min_chord_length;
                }
                None => break
            }
        }

        assert!(cursor == notes_per_bar);
        assert!(abc_notes.len() % tuplet as usize == 0);

        match tuplet
        {
            1 => {
                write!(buffer, "{}", abc_notes.join(""))?;
            }
            n => {
                for chunk in abc_notes.chunks(n as usize)
                {
                    write!(buffer, "({}", n)?;
                    for note in chunk
                    {
                        write!(buffer, "{}", note)?;
                    }
                }
            }
        }

        writeln!(buffer, "|")?;
    }

    Ok(buffer)
}


#[cfg(test)]
mod tests
{
    use super::*;

    fn write_bars_test(source: &str, expected: &str, notes_per_bar: u64)
    {
        use lexing;
        use parsing;
        use sequencing;

        let tokens = lexing::lex(source).expect("ERROR IN LEXER");
        let parse_tree = parsing::parse(&tokens).expect("ERROR IN PARSER");
        let pieces = sequencing::sequence_pieces(&parse_tree.pieces).expect("ERROR IN SEQUENCER");
        let bars = &pieces[0].voices[0].bars;

        assert_eq!(write_bars(bars, notes_per_bar).unwrap(), expected);
    }

    fn write_bars_fail(source: &str, notes_per_bar: u64)
    {
        use lexing;
        use parsing;
        use sequencing;

        let tokens = lexing::lex(source).expect("ERROR IN LEXER");
        let parse_tree = parsing::parse(&tokens).expect("ERROR IN PARSER");
        let pieces = sequencing::sequence_pieces(&parse_tree.pieces).expect("ERROR IN SEQUENCER");
        let bars = &pieces[0].voices[0].bars;

        assert!(write_bars(bars, notes_per_bar).is_err());
    }

    #[test]
    fn test_div_tuplet()
    {
        assert_eq!(div_tuplet(0), (4, 0));
        assert_eq!(div_tuplet(1), (4, 1));
        assert_eq!(div_tuplet(2), (8, 1));
        assert_eq!(div_tuplet(3), (8, 3));
        assert_eq!(div_tuplet(4), (16, 1));
        assert_eq!(div_tuplet(5), (8, 5));
        assert_eq!(div_tuplet(6), (16, 3));
        assert_eq!(div_tuplet(7), (8, 7));
        assert_eq!(div_tuplet(8), (32, 1));
        assert_eq!(div_tuplet(9), (8, 9));
        assert_eq!(div_tuplet(10), (16, 5));
        assert_eq!(div_tuplet(10), (16, 5));
    }

    #[test]
    fn test_single_note()
    {
        let source = "voice A {} play A { :| C | }";
        write_bars_test(source, "L:1/4\nC4|\n", 4);
    }

    #[test]
    fn test_two_notes_in_sequence()
    {
        let source = "voice A {} play A { :| C C | }";
        write_bars_test(source, "L:1/4\nC2C2|\n", 4);
    }

    #[test]
    fn test_four_notes_in_sequence()
    {
        let source = "voice A {} play A { :| C C C C | }";
        write_bars_test(source, "L:1/4\nCCCC|\n", 4);
    }

    #[test]
    fn test_eight_notes_in_sequence()
    {
        let source = "voice A {} play A { :| C C C C C C C C | }";
        write_bars_test(source, "L:1/8\nCCCCCCCC|\n", 4);
    }

    #[test]
    fn test_sixteen_notes_in_sequence()
    {
        let source = "voice A {} play A { :| C C C C C C C C C C C C C C C C | }";
        write_bars_test(source, "L:1/16\nCCCCCCCCCCCCCCCC|\n", 4);
    }

    #[test]
    fn test_four_notes_then_sixteen()
    {
        let source = "voice A {} play A { :| C C C C | C C C C C C C C C C C C C C C C | }";
        write_bars_test(source, "L:1/16\nC4C4C4C4|\nCCCCCCCCCCCCCCCC|\n", 4);
    }

    #[test]
    fn test_three_note_bar_in_3_4_time()
    {
        let source = "voice A {} play A { :| C C C | }";
        write_bars_test(source, "L:1/4\nCCC|\n", 3);
    }

    #[test]
    fn test_triplet_in_4_4_time()
    {
        let source = "voice A {} play A { :| C C C | }";
        write_bars_test(source, "L:1/8\n(3C-C-C-(3CC-C-(3C-CC-(3C-C-C|\n", 4);
    }

    #[test]
    fn test_fast_triplets()
    {
        let source = "voice A {} play A { :| ccc ccc ccc ccc }";
        write_bars_test(source, "L:1/8\n(3ccc(3ccc(3ccc(3ccc|\n", 4);
    }

    #[test]
    fn test_even_faster_triplets()
    {
        let source = "voice A {} play A { :| cccccc cccccc cccccc cccccc }";
        write_bars_test(source, "L:1/16\n(3ccc(3ccc(3ccc(3ccc(3ccc(3ccc(3ccc(3ccc|\n", 4);
    }

    #[test]
    fn test_quintuplet_in_4_4_time()
    {
        let source = "voice A {} play A { :| C C C C C |}";
        write_bars_test(source, "L:1/8\n(5C-C-C-CC-(5C-C-CC-C-(5C-CC-C-C-(5CC-C-C-C|\n", 4);
    }

    #[test]
    fn test_notes_with_rests()
    {
        let source = "voice A {} play A { :| C - C - |}";
        write_bars_test(source, "L:1/4\nCzCz|\n", 4);
    }

    #[test]
    fn test_triplets_with_rests()
    {
        let source = "voice A {} play A { :| C - - - C C | }";
        write_bars_test(source, "L:1/8\n(3C-Cz(3zzz(3zzC-(3CC-C|\n", 4);
    }

    #[test]
    fn test_two_notes_at_once()
    {
        let source = "voice A {} play A { :| C ; :| G }";
        write_bars_test(source, "L:1/4\n[CG]4|\n", 4);
    }

    #[test]
    fn three_notes_at_once()
    {
        let source = "voice A {} play A { :| C - - - ; :| E - - - ; :| G - - - }";
        write_bars_test(source, "L:1/4\n[CEG]z3|\n", 4);
    }

    #[test]
    fn triplet_chords()
    {
        // TODO(***realname***): This and similar tests should be able to be expressed as a single triplet of 1/2 notes
        let source = "voice A {} play A { :| C C C ; :| E E E ; :| G G G }";
        write_bars_test(source, "L:1/8\n(3[CEG]-[CEG]-[CEG]-(3[CEG][CEG]-[CEG]-(3[CEG]-[CEG][CEG]-(3[CEG]-[CEG]-[CEG]|\n", 4);
    }

    #[test]
    fn long_notes_in_triplets()
    {
        let source = "voice A {} play A { :| C | CC | CCC CCC | }";
        write_bars_test(source, "L:1/8\n(3C-C-C-(3C-C-C-(3C-C-C-(3C-C-C|\n(3C-C-C-(3C-C-C(3C-C-C-(3C-C-C|\n(3C-CC-(3CC-C(3C-CC-(3CC-C|\n", 4);
    }

    #[test]
    fn rest_before_chord_not_duplicated()
    {
        let source = " voice A {} play A { :| - C | ; :| - G | }";
        write_bars_test(source, "L:1/4\nz2[CG]2|\n", 4);
    }

    #[test]
    fn large_tuplets_fail()
    {
        let source = "voice A {} play A { :| abcdabcdabc | }";
        write_bars_fail(source, 4);
    }
}
