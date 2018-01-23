extern crate midscript;
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

    let processed = match midscript::compile_to_abc_new(&input_text)
    {
        Err(err) => {
            eprintln!("Compilation failed:\nerror: {}", err);
            std::process::exit(1)
        },
        Ok(p) => p
    };

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


