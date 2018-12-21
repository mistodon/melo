use std::path::Path;

use ansi_term::Style;
use failure::Error;
use structopt_derive::StructOpt;

use melo::colors::{CYAN, RED, WHITE, YELLOW};
use melo::MidiGenerationOptions;

#[derive(Debug, StructOpt)]
enum MeloCommand {
    #[structopt(
        name = "abc",
        about = "Compile melo to abc notation.\n\
                 (Note that this is deprecated and will be removed soon.)"
    )]
    Abc {
        #[structopt(help = "Input file, or stdin if not specified.")]
        input: Option<String>,

        #[structopt(
            short = "o",
            long = "output",
            help = "Output file, or stdout if not specified."
        )]
        output: Option<String>,
    },

    #[structopt(name = "mid", about = "Compile melo to a MIDI file.")]
    Mid {
        #[structopt(help = "Input file, or stdin if not specified.")]
        input: Option<String>,

        #[structopt(
            short = "d",
            long = "division",
            help = "MIDI ticks per beat.",
            default_value = "480"
        )]
        ticks_per_beat: i16,

        #[structopt(
            short = "o",
            long = "output",
            help = "Output file, or stdout if not specified."
        )]
        output: Option<String>,

        #[structopt(
            long = "abcmidi",
            help = "First generate ABC, the convert that to MIDI. Requires `abc2midi`.\n\
                    Note that this is deprecated and will be removed soon."
        )]
        abcmidi: bool,
    },

    #[structopt(
        name = "play",
        about = "Compile and play melo as MIDI. Uses `timidity` by default.\n\
                 You can change this with the `MELO_MIDI_PLAYER` environment variable."
    )]
    Play {
        #[structopt(help = "Input file, or stdin if not specified.")]
        input: Option<String>,

        #[structopt(
            short = "d",
            long = "division",
            help = "MIDI ticks per beat.",
            default_value = "480"
        )]
        ticks_per_beat: i16,

        #[structopt(
            long = "abcmidi",
            help = "First generate ABC, the convert that to MIDI and play. \
                    Requires `abc2midi`.\n\
                    Note that this is deprecated and will be removed soon."
        )]
        abcmidi: bool,
    },

    #[structopt(name = "ref", about = "View useful information for composing in melo.")]
    Ref {
        #[structopt(subcommand)]
        subcommand: RefCommand,
    },
}

#[derive(Debug, StructOpt)]
enum RefCommand {
    #[structopt(name = "notes", about = "View information about valid notes.")]
    Notes,

    #[structopt(
        name = "instruments",
        about = "View the program numbers for GM instruments."
    )]
    Instruments,
}

fn main() {
    use structopt::StructOpt;

    let command = MeloCommand::from_args();

    if let Err(err) = run_command(command) {
        eprintln!("{}", err);
        log(RED, "error:", "Command failed.");
        std::process::exit(1)
    }
}

fn log(color: Style, prefix: &str, message: &str) {
    eprintln!("{} {}", color.paint(prefix), WHITE.paint(message));
}

fn run_command(command: MeloCommand) -> Result<(), Error> {
    use mktemp::Temp;
    use std::process::Command;

    match command {
        MeloCommand::Abc { input, output } => {
            unimplemented!()
            //             let abc = compile_to_abc(&input)?;
            //             write_output(&abc, output)
        }

        MeloCommand::Mid {
            input,
            output,
            ticks_per_beat,
            abcmidi,
        } => {
            if abcmidi {
                compile_to_midi_via_abc(&input, &output)
            } else {
                let options = MidiGenerationOptions { ticks_per_beat };
                let midi = compile_to_midi(&input, &options)?;
                write_binary(&midi, output)
            }
        }

        MeloCommand::Play {
            input,
            ticks_per_beat,
            abcmidi,
        } => {
            let mid_out = Temp::new_file()?;

            if abcmidi {
                compile_to_midi_via_abc(&input, &Some(&mid_out))?;
            } else {
                let options = MidiGenerationOptions { ticks_per_beat };
                let midi = compile_to_midi(&input, &options)?;
                write_binary(&midi, Some(&mid_out))?;
            }

            log(CYAN, "Playing", "...");

            let midi_player_command =
                std::env::var_os("MELO_MIDI_PLAYER").unwrap_or_else(|| "timidity".into());

            let command_output = Command::new(&midi_player_command)
                .arg(mid_out.as_ref().as_os_str())
                .output();

            let command_output = match command_output {
                Ok(x) => x,
                Err(e) => {
                    log(
                        RED,
                        "error:",
                        &format!(
                            "Failed to run external command {:?}.\n\
                             You can change the command used to play MIDI files by setting \
                             the `MELO_MIDI_PLAYER` environment variable.",
                            midi_player_command
                        ),
                    );
                    return Err(e.into());
                }
            };

            if !command_output.status.success() {
                use std::io::Write;

                std::io::stderr().write_all(&command_output.stderr)?;
                return Err(failure::err_msg("Compile and play failed."));
            }

            Ok(())
        }

        MeloCommand::Ref { subcommand } => {
            match subcommand {
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

    match input {
        Some(filename) => {
            File::open(filename.as_ref())?.read_to_string(&mut content)?;
        }
        None => {
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

    if let Some(filename) = output {
        File::create(filename.as_ref())?.write_all(content.as_bytes())?;
    } else {
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

    match input {
        Some(filename) => {
            File::open(filename.as_ref())?.read_to_end(&mut content)?;
        }
        None => {
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

    if let Some(filename) = output {
        File::create(filename.as_ref())?.write_all(content)?;
    } else {
        std::io::stdout().write_all(content)?;
    }

    Ok(())
}

fn compile_to_abc<P>(input: &Option<P>) -> Result<String, Error>
where
    P: AsRef<Path>,
{
    unimplemented!()
    //     log(CYAN, "Compiling", "to abc ...");
    //     log(
    //         YELLOW,
    //         "warning:",
    //         "Compilation to abc is deprecated and will be removed soon.\n",
    //     );
    //
    //     let source = read_input(input.as_ref())?;
    //     let filename = input.as_ref().map(|s| s.as_ref());
    //
    //     #[allow(deprecated)]
    //     let result = melo::compile_to_abc(&source, filename.and_then(Path::to_str))?;
    //
    //     Ok(result)
}

fn compile_to_midi<P>(input: &Option<P>, options: &MidiGenerationOptions) -> Result<Vec<u8>, Error>
where
    P: AsRef<Path>,
{
    log(CYAN, "Compiling", "to MIDI ...");
    let source = read_input(input.as_ref())?;
    let filename = input.as_ref().map(|s| s.as_ref());

    let result = melo::compile_to_midi(&source, filename.and_then(Path::to_str), options)?;

    Ok(result)
}

fn compile_to_midi_via_abc<P, Q>(input: &Option<P>, output: &Option<Q>) -> Result<(), Error>
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
    let mid_out_arg: &OsStr = match *output {
        Some(ref path) => path.as_ref().as_ref(),
        None => mid_out.as_ref().as_os_str(),
    };

    log(CYAN, "Compiling", "to MIDI via abc ...");

    let command_output = Command::new("abc2midi")
        .arg(abc_out.as_ref().as_os_str())
        .arg("-o")
        .arg(mid_out_arg)
        .output()?;

    if !command_output.status.success() {
        use std::io::Write;

        std::io::stderr().write_all(&command_output.stderr)?;
        return Err(failure::err_msg("Compiling to MIDI failed."));
    }

    if output.is_none() {
        let mid = read_binary(Some(mid_out_arg))?;
        write_binary(&mid, output.as_ref())?;
    }

    Ok(())
}
