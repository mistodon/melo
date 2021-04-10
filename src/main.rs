use std::path::Path;

use ansi_term::Style;
use eyre::{eyre, Result};
use structopt::StructOpt;

use melo::colors::{CYAN, RED, WHITE};
use melo::MidiGenerationOptions;

#[derive(Debug, StructOpt)]
enum MeloCommand {
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

fn run_command(command: MeloCommand) -> Result<()> {
    use mktemp::Temp;
    use std::process::Command;

    match command {
        MeloCommand::Mid {
            input,
            output,
            ticks_per_beat,
        } => {
            let options = MidiGenerationOptions { ticks_per_beat };
            let midi = compile_to_midi(&input, &options)?;
            write_binary(&midi, output)
        }

        MeloCommand::Play {
            input,
            ticks_per_beat,
        } => {
            let mid_out = Temp::new_file()?;

            let options = MidiGenerationOptions { ticks_per_beat };
            let midi = compile_to_midi(&input, &options)?;
            write_binary(&midi, Some(&mid_out))?;

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
                return Err(eyre!("Compile and play failed."));
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

fn read_input<P>(input: Option<P>) -> Result<String>
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

fn write_binary<P>(content: &[u8], output: Option<P>) -> Result<()>
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

fn compile_to_midi<P>(input: &Option<P>, options: &MidiGenerationOptions) -> Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    log(CYAN, "Compiling", "to MIDI ...");
    let source = read_input(input.as_ref())?;
    let filename = input.as_ref().map(|s| s.as_ref());

    let result = melo::compile_to_midi(&source, filename.and_then(Path::to_str), options)?;

    Ok(result)
}
