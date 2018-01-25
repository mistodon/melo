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
        let mut scaled_last_end_position = 0;
        let mut last_tuplet_marker = None;

        let mut chord = Vec::new();
        let mut chord_position = 0;

        let mut notes = bar.notes.iter().peekable();

        // Way smarter idea than this:
        //  1.  Choose note/rest and length
        //  2.  If tuplet, shorten length and choose number of repetitions
        //  3.  Push those notes as strings into a Vec (with '-' for ties where required)
        //  4.  iterate over chunks of size n(tuple) and write (n<notes...>
        loop
        {
            let note = notes.next();

            if let Some(note) = note
            {
                chord.push(note);
                chord_position = note.position;
            }

            let scaled_chord_position = chord_position as u64 * scale;

            // Required rest
            {
                let rest_end_position = match note
                {
                    Some(_) => scaled_chord_position,
                    None => notes_per_bar
                };
                let rest_length = rest_end_position - scaled_last_end_position;

                if tuplet > 1
                {
                    let individual_rest_length = notes_per_bar / (tuplet * 2);
                    let mut rest_position = scaled_last_end_position;
                    let rest_end = rest_position + rest_length;

                    while rest_position < rest_end
                    {
                        if rest_position % tuplet == 0 && Some(rest_position) != last_tuplet_marker
                        {
                            last_tuplet_marker = Some(rest_position);
                            write!(buffer, "({}", tuplet)?;
                        }
                        write!(buffer, "z{}", individual_rest_length)?;
                        rest_position += individual_rest_length;
                    }

                    if rest_end_position % tuplet == 0 && Some(rest_end_position) != last_tuplet_marker && rest_end_position != notes_per_bar
                    {
                        last_tuplet_marker = Some(rest_end_position);
                        write!(buffer, "({}", tuplet)?;
                    }
                }
                else
                {
                    match rest_length
                    {
                        0 => (),
                        1 => write!(buffer, "z")?,
                        n => write!(buffer, "z{}", n)?,
                    }
                }
            }

            let next_note = notes.peek();

            match note
            {
                Some(note) =>
                {
                    if Some(note.position) != next_note.map(|note| note.position)
                    {

                        let abc = {
                            match chord.len()
                            {
                                0 => unreachable!(),
                                1 => notes::midi_to_abc(chord[0].midi).trust().to_owned(),
                                _ => {
                                    let chord_abc = chord.iter()
                                        .map(|note| notes::midi_to_abc(note.midi).trust())
                                        .collect::<Vec<&str>>()
                                        .join("");
                                    format!("[{}]", chord_abc)
                                }
                            }
                        };

                        let min_chord_length = chord.iter().map(|note| note.length).min().trust();
                        let scaled_chord_length = (min_chord_length as u64) * scale;

                        match scaled_chord_length
                        {
                            1 => write!(buffer, "{}", abc)?,
                            n => write!(buffer, "{}{}", abc, n)?,
                        }

                        chord.clear();
                        scaled_last_end_position = scaled_chord_position + scaled_chord_length;
                    }
                }
                None => break
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
        write_bars_test(source, "L:1/8\n(3C4C4C4|\n", 4);
    }

    #[test]
    fn test_quintuplet_in_4_4_time()
    {
        let source = "voice A {} play A { :| C C C C C |}";
        write_bars_test(source, "L:1/8\n(5C4C4C4C4C4|\n", 4);
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
        write_bars_test(source, "L:1/8\n(3C2z2z2(3z2C2C2|\n", 4);
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
        let source = "voice A {} play A { :| C C C ; :| E E E ; :| G G G }";
        write_bars_test(source, "L:1/8\n(3[CEG]4[CEG]4[CEG]4|\n", 4);
    }

    #[test]
    #[ignore] // TODO(claire): Make this work
    fn long_notes_in_triplets()
    {
        let source = "voice A {} play A { :| C | CC | CCC CCC | }";
        write_bars_test(source, "L:1/8\n(3C2-C2-C2-(3C2-C2-C2|\n(3C2-C2-C2(3C2-C2-C2|\n(3C2C2C2(3C2C2C2|\n", 4);
    }

    #[test]
    // TODO(claire): Remove this test if it is fixed by fixing `rest_before_chord_not_duplicated`
    fn broken_drums()
    {
        let source = "
            voice A { drums }
            play A {
                d^: | ---- x--- ---- x--- |
                F^: | x--- x--- x--- x--- |
                D:  | --xx ---- --xx --xx |
                C:  | x--- ---- x--- ---- |
            }";
        write_bars_test(source, "L:1/16\n[^F,,C,,]zD,,D,,[^D^F,,]z3[^F,,C,,]zD,,D,,[^D,^F,,]zD,,D,,|\n", 4);
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
