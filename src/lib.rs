#[cfg(test)]
#[macro_use] extern crate pretty_assertions;

#[macro_use] extern crate failure;
#[macro_use] extern crate lazy_static;

extern crate regex;

pub mod notes;
mod abc_generation;
pub mod lexing;     // TODO(***realname***): These should be private
pub mod parsing;    // TODO(***realname***): These should be private
mod validation;
mod trust;

#[cfg(test)]
mod test_helpers;


use failure::Error;


struct CompileDrumsOptions
{
    pub beats: usize,
    pub channel: u8,
    pub program: u8,
    pub octave: i8,
}

impl Default for CompileDrumsOptions
{
    fn default() -> Self
    {
        CompileDrumsOptions
        {
            beats: 4,
            channel: 0,
            program: 0,
            octave: 0,
        }
    }
}


pub fn compile_to_abc_new(input: &str) -> Result<String, Error>
{
    let tokens = lexing::lex(input)?;
    let mut parse_tree = parsing::parse(&tokens)?;
    let valid = validation::adjust_and_validate(&mut parse_tree);

    if !valid
    {
        eprintln!("Validation failed!");
        panic!("Validation failed and not correctly handling errors yet!")
    }
    else
    {
        Ok(abc_generation::generate_abc(&parse_tree).expect("We aren't handling generation errors yet!"))
    }
}

pub fn compile_to_abc(input: &str) -> String
{
    use regex::{ Regex, Captures };

    let voice_pattern = Regex::new(r"(?m)voice\s+([a-zA-Z0-9_\- ]+)\s*\(([a-zA-Z0-9=+\-, ]*)\)\s*\{([^{}]*)\}\n?").expect("Failed to compile voice regex");

    let blank_line_pattern = Regex::new(r"\n\s*\n").expect("Failed to compile blank line regex");

    let result = voice_pattern.replace_all(input,
        |captures: &Captures|
        {
            let params = {
                use std::collections::BTreeMap;

                let mut params = BTreeMap::new();
                for param in captures[2].split(',').map(str::trim).filter(|arg| !arg.is_empty())
                {
                    let divide = param.find('=');
                    match divide
                    {
                        Some(divide) => {
                            let key = param[0..divide].trim();
                            let value = param[(divide+1)..].trim();
                            params.insert(key, value);
                        }
                        None => match param
                        {
                            "drums" => {
                                params.insert("channel", "10");
                            }
                            s => panic!("Unrecognized option '{}'", s)
                        }
                    }
                }

                let mut options = CompileDrumsOptions::default();

                options.beats = params.get("beats").map(|value| value.parse::<usize>().unwrap()).unwrap_or(options.beats);
                options.channel = params.get("channel").map(|value| value.parse::<u8>().unwrap()).unwrap_or(options.channel);
                options.program = params.get("program").map(|value| value.parse::<u8>().unwrap()).unwrap_or(options.program);
                options.octave = params.get("octave").map(|value| value.parse::<i8>().unwrap()).unwrap_or(options.octave);

                options
            };
            let voice_name = &captures[1];
            let content = &captures[3];
            compile_midscript_to_abc(voice_name, content, &params)
        });
    let result = blank_line_pattern.replace_all(&result, "\n%\n");

    result.into_owned()
}


#[derive(Debug, Default, Clone)]
struct Stave
{
    pub bars: Vec<Bar<Note>>,
}

#[derive(Debug, Default, Clone)]
struct Bar<T>(pub Vec<T>);

impl<T: Clone + Default> Bar<T>
{
    pub fn stretched(&self, beats: usize) -> Self
    {
        let prev_beats = self.0.len();
        let can_stretch = beats % prev_beats == 0;
        assert!(can_stretch, "Cannot stretch a bar from {} to {}", prev_beats, beats);

        let stride = beats / prev_beats;

        let result = (0..beats).into_iter()
            .map(|beat| if beat % stride == 0 { self.0[beat / stride].clone() } else { T::default() })
            .collect::<Vec<T>>();

        Bar(result)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Note
{
    Rest,
    Note(i8),
}

impl Default for Note { fn default() -> Self { Note::Rest } }

impl Note
{
    pub fn as_abc(&self) -> &'static str
    {
        const REST: &str = "z";

        match *self
        {
            Note::Rest => REST,
            Note::Note(note) => notes::midi_to_abc(note).unwrap(),
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Chord
{
    notes: Vec<Note>,
}


fn compile_midscript_to_abc(voice_name: &str, input: &str, options: &CompileDrumsOptions) -> String
{
    let staves: Vec<Stave> = {
        use std::collections::BTreeMap;

        let mut stave_map = BTreeMap::new();
        let mut previous_stave = None;
        let mut allow_new_staves = true;

        const ANONYMOUS_STAVE_NAMES: &[&str] = &[
            "V0", "V1", "V2", "V3", "V4", "V5", "V6", "V7",
        ];
        let mut anonymous_stave_count = 0;

        for line in input.lines()
            .map(str::trim)
        {
            if line.is_empty()
            {
                if !stave_map.is_empty()
                {
                    allow_new_staves = false;
                }
                anonymous_stave_count = 0;
                continue
            }

            let divide = line.find(':');

            let (stave_prefix, stave_bars) = match divide
            {
                Some(divide) => {
                    let stave_prefix = line[0..divide].trim();
                    let stave_bars = &line[(divide+1) ..];
                    (stave_prefix, stave_bars)
                }
                None => {
                    let stave_prefix = previous_stave.expect("Expected stave to begin with \"[prefix]:\"");
                    let stave_bars = line;
                    (stave_prefix, stave_bars)
                }
            };

            let stave_prefix = match stave_prefix
            {
                s if s.is_empty() => {
                    let prefix = ANONYMOUS_STAVE_NAMES[anonymous_stave_count];
                    anonymous_stave_count += 1;
                    prefix
                }
                s => s
            };

            if !(allow_new_staves || stave_map.contains_key(stave_prefix))
            {
                panic!("All staves must be declared before first blank line");
            }

            previous_stave = Some(stave_prefix);

            let stave: &mut Stave = stave_map.entry(stave_prefix).or_insert_with(Stave::default);

            fn parse_bars<F>(stave_bars: &str, parsing: F) -> Vec<Bar<Note>>
            where
                F: Fn(&str) -> Bar<Note>
            {
                stave_bars.split([';', '|'].as_ref())
                    .filter(|bar| !bar.trim().is_empty())
                    .map(parsing)
                    .collect()
            }

            let mut bars = {
                if stave_prefix.starts_with('V')
                {
                    parse_bars(stave_bars,
                        |bar|
                        {
                            let notes = {
                                let oct = options.octave * 12;
                                let mut notes = Vec::new();
                                let mut token_start = 0;
                                let mut in_note = false;

                                for (index, ch) in bar.char_indices()
                                {
                                    let is_whitespace = ch.is_whitespace();
                                    let is_rest = ch == '-';
                                    let is_note_start = ('a' <= ch && ch <= 'g') || ('A' <= ch && ch <= 'G');
                                    let ends_current_note = is_whitespace || is_rest || is_note_start;

                                    if in_note && ends_current_note
                                    {
                                        let note = notes::note_to_midi(&bar[token_start..index]).unwrap() + oct;
                                        notes.push(Note::Note(note));
                                        in_note = false;
                                    }

                                    if is_rest
                                    {
                                        notes.push(Note::Rest);
                                    }

                                    if is_note_start
                                    {
                                        token_start = index;
                                        in_note = true;
                                    }
                                }

                                if in_note
                                {
                                    let note = notes::note_to_midi(&bar[token_start..]).unwrap() + oct;
                                    notes.push(Note::Note(note));
                                }

                                notes
                            };

                            Bar(notes)
                        })
                }
                else
                {
                    parse_bars(stave_bars,
                        |bar|
                        {
                            let notes = bar.chars()
                                .filter(|ch| !ch.is_whitespace())
                                .map(
                                    |ch| match ch
                                    {
                                        'x' => Note::Note(notes::note_to_midi(stave_prefix).unwrap() + (options.octave * 12)),
                                        '-' => Note::Rest,
                                        c => panic!("Invalid char {} in drum line", c)
                                    })
                                .collect::<Vec<Note>>();

                            Bar(notes)
                        })
                }
            };

            stave.bars.append(&mut bars);
        }

        stave_map.into_iter().map(|(_, value)| value).collect()
    };

    let stave_length = staves.get(0).unwrap().bars.len();
    assert!(staves.iter().all(|stave| stave.bars.len() == stave_length), "All staves must be the same length");

    let track = (0..stave_length).into_iter()
        .map(
            |bar_index|
            {
                let bar_length = staves.iter().map(|stave| stave.bars[bar_index].0.len()).max().unwrap();
                let bars = staves.iter().map(|stave| stave.bars[bar_index].stretched(bar_length));

                let mut chords: Bar<Chord> = Bar(vec![Chord::default(); bar_length]);
                for bar in bars
                {
                    for (index, &note) in bar.0.iter().enumerate().filter(|&(_, &note)| note != Note::Rest)
                    {
                        chords.0[index].notes.push(note);
                    }
                }

                chords
            })
        .collect::<Vec<Bar<Chord>>>();

    let max_bar_len = track.iter().map(|bar| bar.0.len()).max().unwrap();

    let min_bar_len = {
        // TODO(***realname***): This should really be the LCM of max_bar_len and options.beats
        let min_bar_len = std::cmp::max(max_bar_len, options.beats);
        let min_bar_len = if min_bar_len % options.beats == 0 { min_bar_len } else { min_bar_len * options.beats };
        assert!(min_bar_len % options.beats == 0, "All bars must be aligned with the time signature");
        min_bar_len
    };

    let track = track.iter().map(|bar| bar.stretched(min_bar_len));

    let notes_per_beat = min_bar_len / options.beats;
    let (beat_division, tuplet) = match notes_per_beat
    {
        1 => (options.beats, None),
        n if n % 7 == 0 => ((n*8) / 7, Some(7)),
        n if n % 5 == 0 => ((n*8) / 5, Some(5)),
        n if n % 3 == 0 => ((n*8) / 3, Some(3)),
        n if n % 2 == 0 => ((n*8) / 2, None),
        _ => unimplemented!("Unsupported tuplet")
    };

    let mut buffer = String::new();

    {
        use std::fmt::Write;

        writeln!(buffer, "V:{}", voice_name).unwrap();
        writeln!(buffer, "%%MIDI channel {}", options.channel).unwrap();
        writeln!(buffer, "%%MIDI program {}", options.program).unwrap();
        writeln!(buffer, "L:1/{}", beat_division).unwrap();

        for bar in track
        {
            for (index, chord) in bar.0.iter().enumerate()
            {
                if let Some(tuplet) = tuplet
                {
                    if index % tuplet == 0
                    {
                        write!(buffer, "({}", tuplet).unwrap();
                    }
                }

                match chord.notes.len()
                {
                    0 => buffer += Note::Rest.as_abc(),
                    1 => buffer += chord.notes[0].as_abc(),
                    _ => {
                        buffer += "[";
                        for (index, &note) in chord.notes.iter().enumerate()
                        {
                            if index != 0
                            {
                                buffer += " ";
                            }
                            buffer += note.as_abc();
                        }
                        buffer += "]";
                    }
                }
            }
            buffer += "|\n";
        }
    }

    buffer
}

