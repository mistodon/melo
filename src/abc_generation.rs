use parsing::{ ParseTree, NoteNode };
use notes;


pub fn generate_abc(parse_tree: &ParseTree) -> Option<String>
{
    use std::fmt::Write;

    let mut buffer = String::new();

    for (index, piece) in parse_tree.pieces.iter().enumerate()
    {
        writeln!(buffer, "X:{}", index + 1).ok()?;

        if let Some(title) = piece.title
        {
            writeln!(buffer, "T:{}", title).ok()?;
        }
        if let Some(composer) = piece.composer
        {
            writeln!(buffer, "C:{}", composer).ok()?;
        }

        let beats = piece.beats.unwrap_or(4) as usize;

        writeln!(buffer, "M:{}/4", beats).ok()?;
        writeln!(buffer, "Q:1/4={}", piece.tempo.unwrap_or(120)).ok()?;
        writeln!(buffer, "K:C").ok()?;

        for voice in &piece.voices
        {
            writeln!(buffer, "V:{}", voice.name).ok()?;
            writeln!(buffer, "%%MIDI channel {}", voice.channel.unwrap_or(1)).ok()?;
            writeln!(buffer, "%%MIDI program {}", voice.program.unwrap_or(0)).ok()?;

            for play in piece.plays.iter().filter(|play| play.voice == Some(voice.name))
            {
                let first_stave_bars = &play.staves.get(0)?.bars;
                let bar_count = first_stave_bars.len();
                let bar_length = first_stave_bars.get(0)?.notes.len();

                let notes_per_beat = bar_length / beats;

                let (beat_division, tuplet) = match notes_per_beat
                {
                    1 => (beats, None),
                    n if n % 7 == 0 => ((n*8) / 7, Some(7)),
                    n if n % 5 == 0 => ((n*8) / 5, Some(5)),
                    n if n % 3 == 0 => ((n*8) / 3, Some(3)),
                    n if n % 2 == 0 => ((n*8) / 2, None),
                    _ => unimplemented!("Unsupported tuplet")
                };

                writeln!(buffer, "L:1/{}", beat_division).ok()?;

                for bar_index in 0..bar_count
                {
                    for beat_index in 0..bar_length
                    {
                        if let Some(tuplet) = tuplet
                        {
                            if beat_index % tuplet == 0
                            {
                                write!(buffer, "({}", tuplet).ok()?
                            }
                        }

                        let mut chord = Vec::new();
                        for stave in &play.staves
                        {
                            let note = stave.bars[bar_index].notes[beat_index];
                            if let NoteNode::Note(midi) = note
                            {
                                chord.push(midi);
                            }
                        }

                        match chord.len()
                        {
                            0 => buffer += "z",
                            1 => buffer += notes::midi_to_abc(chord[0])?,
                            _ => {
                                buffer += "[";
                                for (index, &midi) in chord.iter().enumerate()
                                {
                                    if index != 0
                                    {
                                        buffer += " ";
                                    }
                                    buffer += notes::midi_to_abc(midi)?
                                }
                                buffer += "]";
                            }
                        }
                    }

                    buffer += "|\n";
                }
            }
        }
    }

    Some(buffer)
}

