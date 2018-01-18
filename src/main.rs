extern crate regex;
extern crate structopt;

#[macro_use]
extern crate structopt_derive;


use structopt::StructOpt;


#[derive(Debug, StructOpt)]
struct Command
{
    #[structopt(help = "Input file, or stdin")]
    input: Option<String>,

    #[structopt(short = "o", long = "output", help = "Output file, or stdout")]
    output: Option<String>,
}


fn main()
{
    use std::fs::File;

    let command = Command::from_args();

    let input_text = {
        use std::io::Read;

        let mut content = String::new();

        match command.input
        {
            Some(filename) => {

                File::open(&filename).unwrap().read_to_string(&mut content).unwrap();
            },
            None => {
                std::io::stdin().read_to_string(&mut content).unwrap();
            }
        }

        content
    };

    let processed = process(&input_text);

    {
        use std::io::Write;

        if let Some(filename) = command.output
        {
            File::create(&filename).unwrap().write_all(processed.as_bytes()).unwrap();
        }
        else
        {
            std::io::stdout().write_all(processed.as_bytes()).unwrap();
        }
    }
}


fn process(input: &str) -> String
{
    use regex::{ Regex, Captures };

    let pattern = Regex::new(r"(?m)%\s*MIDSCRIPT\[\n([^%]*)%\s*\]MIDSCRIPT\n").expect("Failed to compile regex");

    let result = pattern.replace_all(input, |captures: &Captures| compile(&captures[1]));

    result.into_owned()
}


fn compile(input: &str) -> String
{
    use std::collections::BTreeMap;

    type Beat = bool;
    type DrumBar = Vec<Beat>;
    type Stave = Vec<DrumBar>;

    let staves: BTreeMap<String, Stave> = {
        let mut staves = BTreeMap::new();

        for line in input.lines()
        {
            let divide = line.find(':').expect("Expected stave to begin with \"<note>:\"");
            let note = line[0..divide].to_owned();
            let rest = &line[(divide+1) ..];

            let stave: &mut Stave = staves.entry(note).or_insert(Vec::new());

            for bar_chars in rest.split(';')
            {
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
                if note.len() == 1
                {
                    buffer.push_str(&note[0]);
                }
                else
                {
                    buffer.push_str(&format!("[{}]", note.join(" ")));
                }
                buffer.push_str(" ");
            }
            buffer.push_str("|\n");
        }
    }

    buffer
}

