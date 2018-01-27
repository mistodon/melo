extern crate midscript;
extern crate structopt;

#[macro_use]
extern crate structopt_derive;

use std::path::Path;

use structopt::StructOpt;


#[derive(Debug, StructOpt)]
enum Command
{
    #[structopt(name = "abc", about = "Compile midscript to abc notation.")]
    Abc
    {
        #[structopt(help = "Input file, or stdin if not specified.")]
        input: Option<String>,

        #[structopt(short = "o", long = "output",
                    help = "Output file, or stdout if not specified.")]
        output: Option<String>,
    },

    #[structopt(name = "mid",
                about = "Compile midscript to a MIDI file. (Currently requires abc2midi.)")]
    Mid
    {
        #[structopt(help = "Input file, or stdin if not specified.")]
        input: Option<String>,

        #[structopt(short = "o", long = "output", help = "Output file.")]
        output: String,
    },

    #[structopt(name = "play",
                about = "Compile and play midscript as MIDI. (Currently requires timidity.)")]
    Play
    {
        #[structopt(help = "Input file, or stdin if not specified.")]
        input: Option<String>,
    },

    #[structopt(name = "ref", about = "View useful information for composing in miscript.")]
    Ref
    {
        #[structopt(subcommand)]
        subcommand: RefCommand,
    },
}

#[derive(Debug, StructOpt)]
enum RefCommand
{
    #[structopt(name = "notes", about = "View information about valid notes.")] Notes,

    #[structopt(name = "instruments", about = "View the program numbers for GM instruments.")]
    Instruments,
}


fn main()
{
    let command = Command::from_args();

    match command
    {
        Command::Abc { input, output } =>
        {
            let input_text = read_input(input.as_ref());

            let processed = match midscript::compile_to_abc(&input_text)
            {
                Err(err) =>
                {
                    eprintln!("Compilation failed:\n{}", err);
                    std::process::exit(1)
                }
                Ok(p) => p,
            };

            write_output(&processed, output);
        }

        Command::Mid { input, output } =>
        {
            use std::process::Command;

            let input_text = read_input(input.as_ref());

            let processed = match midscript::compile_to_abc(&input_text)
            {
                Err(err) =>
                {
                    eprintln!("Compilation failed:\n{}", err);
                    std::process::exit(1)
                }
                Ok(p) => p,
            };

            let mut intermediate = output.clone();
            intermediate.push_str(".abc");

            write_output(&processed, Some(&intermediate));

            let output = Command::new("abc2midi")
                .arg(&intermediate)
                .arg("-o")
                .arg(&output)
                .output();

            println!("{:?}", output);
        }

        Command::Play { input } =>
        {
            use std::process::Command;

            let input_text = read_input(input.as_ref());

            let processed = match midscript::compile_to_abc(&input_text)
            {
                Err(err) =>
                {
                    eprintln!("Compilation failed:\n{}", err);
                    std::process::exit(1)
                }
                Ok(p) => p,
            };

            let intermediate = "anonymous.abc";

            write_output(&processed, Some(intermediate));

            let output = Command::new("abc2midi")
                .arg(&intermediate)
                .arg("-o")
                .arg("anonymous.mid")
                .output();

            println!("{:?}", output);

            let output = Command::new("timidity").arg("anonymous.mid").output();

            println!("{:?}", output);
        }

        Command::Ref { subcommand } => match subcommand
        {
            RefCommand::Notes => println!(
                "{}",
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/reference/notes.txt"))
            ),

            RefCommand::Instruments => println!(
                "{}",
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/reference/instruments.txt"
                ))
            ),
        },
    }
}

fn read_input<P>(input: Option<P>) -> String
where
    P: AsRef<Path>,
{
    use std::fs::File;
    use std::io::Read;

    let mut content = String::new();

    match input
    {
        Some(filename) =>
        {
            File::open(filename.as_ref())
                .unwrap()
                .read_to_string(&mut content)
                .unwrap();
        }
        None =>
        {
            std::io::stdin().read_to_string(&mut content).unwrap();
        }
    }

    content
}

fn write_output<P>(content: &str, output: Option<P>)
where
    P: AsRef<Path>,
{
    use std::fs::File;
    use std::io::Write;

    if let Some(filename) = output
    {
        File::create(filename.as_ref())
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
    }
    else
    {
        std::io::stdout().write_all(content.as_bytes()).unwrap();
    }
}
