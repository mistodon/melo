extern crate regex;


pub fn compile_to_abc(input: &str) -> String
{
    use regex::{ Regex, Captures };

    let drumscript_pattern = Regex::new(r"(?m)drumscript\s*\{([^{}]*)\}\n?").expect("Failed to compile drumscript regex");

    let blank_line_pattern = Regex::new(r"\n\s*\n").expect("Failed to compile blank line regex");

    let result = drumscript_pattern.replace_all(input, |captures: &Captures| compile_drums_to_abc(&captures[1]));
    let result = blank_line_pattern.replace_all(&result, "\n%\n");

    result.into_owned()
}


fn compile_drums_to_abc(input: &str) -> String
{
    use std::collections::BTreeMap;

    type Beat = bool;
    type DrumBar = Vec<Beat>;
    type Stave = Vec<DrumBar>;

    let staves: BTreeMap<String, Stave> = {
        let mut staves = BTreeMap::new();

        for line in input.lines()
        {
            let line = line.trim();
            if line.is_empty()
            {
                continue
            }

            let divide = line.find(':').expect("Expected stave to begin with \"<note>:\"");
            let note = line[0..divide].to_owned();
            let rest = &line[(divide+1) ..];

            let stave: &mut Stave = staves.entry(note).or_insert(Vec::new());

            for bar_chars in rest.split(';')
            {
                if bar_chars.trim().is_empty()
                {
                    continue
                }

                let mut bar = Vec::new();
                for ch in bar_chars.chars()
                {
                    match ch
                    {
                        'x' => bar.push(true),
                        '.' => bar.push(false),
                        _ => ()
                    }
                }
                stave.push(bar);
            }
        }

        staves
    };

    type Note = Vec<String>;
    type Bar = Vec<Note>;

    let bars: Vec<Bar> = {
        let note_names: Vec<String> = staves.keys().map(String::to_owned).collect();
        let staves: Vec<Stave> = staves.values().map(Vec::to_owned).collect();
        let bar_count = staves[0].len();

        let mut bars = Vec::with_capacity(bar_count);

        for bar_index in 0..bar_count
        {
            let mut bar = Vec::new();

            for beat_num in 0..8
            {
                let mut note = Vec::new();

                for note_index in 0..note_names.len()
                {
                    let is_beat = staves[note_index][bar_index][beat_num];

                    if is_beat
                    {
                        note.push(note_names[note_index].clone());
                    }
                }

                bar.push(note);
            }

            bars.push(bar);
        }

        bars
    };

    let mut buffer = String::new();

    {
        for bar in &bars
        {
            for note in bar
            {
                match note.len()
                {
                    0 => buffer.push_str("z"),
                    1 => buffer.push_str(&note[0]),
                    _ => buffer.push_str(&format!("[{}]", note.join(" "))),
                }
            }
            buffer.push_str("|\n");
        }
    }

    buffer
}

