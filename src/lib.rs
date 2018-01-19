extern crate regex;


struct CompileDrumsOptions
{
    pub beats: usize,
}

impl Default for CompileDrumsOptions
{
    fn default() -> Self
    {
        CompileDrumsOptions
        {
            beats: 4,
        }
    }
}


pub fn compile_to_abc(input: &str) -> String
{
    use regex::{ Regex, Captures };

    let drumscript_pattern = Regex::new(r"(?m)drumscript\s*\(([a-zA-Z0-9= ]*)\)\s*\{([^{}]*)\}\n?").expect("Failed to compile drumscript regex");

    let blank_line_pattern = Regex::new(r"\n\s*\n").expect("Failed to compile blank line regex");

    let result = drumscript_pattern.replace_all(input,
        |captures: &Captures|
        {
            let params = {
                use std::collections::BTreeMap;

                let mut params = BTreeMap::new();
                for param in captures[1].split(',').map(str::trim).filter(|arg| !arg.is_empty())
                {
                    let divide = param.find('=').unwrap();
                    let key = param[0..divide].trim();
                    let value = param[(divide+1)..].trim();
                    params.insert(key, value);
                }

                let mut options = CompileDrumsOptions::default();

                options.beats = params.get("beats").map(|value| value.parse::<usize>().unwrap()).unwrap_or(options.beats);

                options
            };
            let content = &captures[2];
            compile_drums_to_abc(content, &params)
        });
    let result = blank_line_pattern.replace_all(&result, "\n%\n");

    result.into_owned()
}


#[derive(Debug, Default, Clone)]
struct Stave<'a>
{
    pub bars: Vec<Bar<Note<'a>>>,
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
enum Note<'a>
{
    Rest,
    Note(&'a str),
}

impl<'a> Default for Note<'a> { fn default() -> Self { Note::Rest } }

impl<'a> Note<'a>
{
    pub fn as_abc(&self) -> &'a str
    {
        const REST: &str = "z";

        match *self
        {
            Note::Rest => REST,
            Note::Note(note) => note,
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Chord<'a>
{
    notes: Vec<Note<'a>>,
}


fn compile_drums_to_abc(input: &str, options: &CompileDrumsOptions) -> String
{
    let staves: Vec<Stave> = {
        use std::collections::BTreeMap;

        let mut stave_map = BTreeMap::new();

        for line in input.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
        {
            let divide = line.find(':').expect("Expected stave to begin with \"<note>:\"");
            let stave_note = line[0..divide].trim();
            let stave_bars = &line[(divide+1) ..];

            let stave: &mut Stave = stave_map.entry(stave_note).or_insert_with(Stave::default);

            let mut bars: Vec<Bar<Note>> = stave_bars.split(';')
                .filter(|bar| !bar.trim().is_empty())
                .map(
                    |bar|
                    {
                        let notes = bar.chars()
                            .filter(|ch| !ch.is_whitespace())
                            .map(
                                |ch| match ch
                                {
                                    'x' => Note::Note(stave_note),
                                    '-' => Note::Rest,
                                    c => panic!("Invalid char {} in drum line", c)
                                })
                            .collect::<Vec<Note>>();
                        Bar(notes)
                    })
                .collect();

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
    let min_bar_len = std::cmp::max(max_bar_len, options.beats);
    let track = track.iter().map(|bar| bar.stretched(min_bar_len));

    assert!(min_bar_len % options.beats == 0, "All bars must be aligned with the time signature");

    let notes_per_beat = max_bar_len / options.beats;
    let (beat_division, tuplet) = match notes_per_beat
    {
        n if n % 7 == 0 => ((n*8) / 7, Some(7)),
        n if n % 5 == 0 => ((n*8) / 5, Some(5)),
        n if n % 3 == 0 => ((n*8) / 3, Some(3)),
        n if n % 2 == 0 => ((n*8) / 2, None),
        _ => unimplemented!("Unsupported tuplet")
    };

    let mut buffer = String::new();

    {
        use std::fmt::Write;

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

