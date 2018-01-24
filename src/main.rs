extern crate midscript;
extern crate structopt;

#[macro_use]
extern crate structopt_derive;


use structopt::StructOpt;


#[derive(Debug, StructOpt)]
enum Command
{
    #[structopt(name = "abc", about = "Compile midscript to abc notation.")]
    Abc
    {
        #[structopt(help = "Input file, or stdin if not specified.")]
        input: Option<String>,

        #[structopt(short = "o", long = "output", help = "Output file, or stdout if not specified.")]
        output: Option<String>,
    },

    #[structopt(name = "ref", about = "View useful information for composing in miscript.")]
    Ref
    {
        #[structopt(subcommand)]
        subcommand: RefCommand,
    }
}

#[derive(Debug, StructOpt)]
enum RefCommand
{
    #[structopt(name = "notes", about = "View information about valid notes.")]
    Notes,

    #[structopt(name = "instruments", about = "View the program numbers for GM instruments.")]
    Instruments,
}


fn main()
{
    use std::fs::File;

    let command = Command::from_args();

    match command
    {
        Command::Abc { input, output } =>
        {
            let input_text = {
                use std::io::Read;

                let mut content = String::new();

                match input
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

            let processed = match midscript::compile_to_abc(&input_text)
            {
                Err(err) => {
                    eprintln!("Compilation failed:\n{}", err);
                    std::process::exit(1)
                },
                Ok(p) => p
            };

            {
                use std::io::Write;

                if let Some(filename) = output
                {
                    File::create(&filename).unwrap().write_all(processed.as_bytes()).unwrap();
                }
                else
                {
                    std::io::stdout().write_all(processed.as_bytes()).unwrap();
                }
            }
        }

        Command::Ref { subcommand } => match subcommand
        {
            RefCommand::Notes => println!(
                "{}",
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/reference/notes.txt"))),

            RefCommand::Instruments => println!(
                "{}",
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/reference/instruments.txt"))),
        }
    }
}


