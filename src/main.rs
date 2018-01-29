extern crate failure;
extern crate midscript;
extern crate mktemp;
extern crate structopt;

#[macro_use]
extern crate structopt_derive;


use std::path::Path;

use failure::Error;


#[derive(Debug, StructOpt)]
enum MidscriptCommand
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

        #[structopt(short = "o", long = "output",
                    help = "Output file, or stdout if not specified.")]
        output: Option<String>,
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
    use structopt::StructOpt;

    let command = MidscriptCommand::from_args();

    if let Err(err) = run_command(command)
    {
        eprintln!("{}\n    Command failed.", err);
    }
}


fn run_command(command: MidscriptCommand) -> Result<(), Error>
{
    use mktemp::Temp;
    use std::process::Command;

    match command
    {
        MidscriptCommand::Abc { input, output } =>
        {
            let abc = compile_to_abc(&input)?;
            write_output(&abc, output)
        }

        MidscriptCommand::Mid { input, output } => compile_to_midi(&input, &output),

        MidscriptCommand::Play { input } =>
        {
            let mid_out = Temp::new_file()?;
            compile_to_midi(&input, &Some(&mid_out))?;

            eprintln!("    Playing...");

            let midi_player_command =
                std::env::var_os("MIDSCRIPT_MIDI_PLAYER").unwrap_or_else(|| "timidity".into());
            let command_output = Command::new(midi_player_command)
                .arg(mid_out.as_ref().as_os_str())
                .output()?;

            if !command_output.status.success()
            {
                use std::io::Write;

                std::io::stderr().write_all(&command_output.stderr)?;
                return Err(failure::err_msg("    Compile and play failed."))
            }

            Ok(())
        }

        MidscriptCommand::Ref { subcommand } =>
        {
            match subcommand
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
            }

            Ok(())
        }
    }
}


fn read_input<P>(input: Option<P>) -> Result<String, Error>
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
            File::open(filename.as_ref())?.read_to_string(&mut content)?;
        }
        None =>
        {
            std::io::stdin().read_to_string(&mut content)?;
        }
    }

    Ok(content)
}


fn write_output<P>(content: &str, output: Option<P>) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    use std::fs::File;
    use std::io::Write;

    if let Some(filename) = output
    {
        File::create(filename.as_ref())?.write_all(content.as_bytes())?;
    }
    else
    {
        std::io::stdout().write_all(content.as_bytes())?;
    }

    Ok(())
}


fn read_binary<P>(input: Option<P>) -> Result<Vec<u8>, Error>
where
    P: AsRef<Path>,
{
    use std::fs::File;
    use std::io::Read;

    let mut content = Vec::new();

    match input
    {
        Some(filename) =>
        {
            File::open(filename.as_ref())?.read_to_end(&mut content)?;
        }
        None =>
        {
            std::io::stdin().read_to_end(&mut content)?;
        }
    }

    Ok(content)
}


fn write_binary<P>(content: &[u8], output: Option<P>) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    use std::fs::File;
    use std::io::Write;

    if let Some(filename) = output
    {
        File::create(filename.as_ref())?.write_all(content)?;
    }
    else
    {
        std::io::stdout().write_all(content)?;
    }

    Ok(())
}


fn compile_to_abc<P>(input: &Option<P>) -> Result<String, Error>
where
    P: AsRef<Path>,
{
    eprintln!("    Compiling to abc...");
    let source = read_input(input.as_ref())?;
    let filename = input.as_ref().map(|s| s.as_ref());

    let result = midscript::compile_to_abc(&source, filename.and_then(Path::to_str))?;

    Ok(result)
}


fn compile_to_midi<P, Q>(input: &Option<P>, output: &Option<Q>) -> Result<(), Error>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    use mktemp::Temp;
    use std::ffi::OsStr;
    use std::process::Command;

    let abc = compile_to_abc(input)?;
    let abc_out = Temp::new_file()?;
    write_output(&abc, Some(&abc_out))?;

    let mid_out = Temp::new_file()?;
    let mid_out_arg: &OsStr = match *output
    {
        Some(ref path) => path.as_ref().as_ref(),
        None => mid_out.as_ref().as_os_str(),
    };

    eprintln!("    Compiling to MIDI...");

    let command_output = Command::new("abc2midi")
        .arg(abc_out.as_ref().as_os_str())
        .arg("-o")
        .arg(mid_out_arg)
        .output()?;

    if !command_output.status.success()
    {
        use std::io::Write;

        std::io::stderr().write_all(&command_output.stderr)?;
        return Err(failure::err_msg("    Compiling to MIDI failed."))
    }

    if output.is_none()
    {
        let mid = read_binary(Some(mid_out_arg))?;
        write_binary(&mid, output.as_ref())?;
    }

    Ok(())
}
