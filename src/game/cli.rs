use std::io::stdin;
use std::{sync, thread};

use log::*;

use thiserror::*;

pub enum CliCommand {
    ZoomSet(f32),
    ZoomChange(f32),
}

#[derive(Debug, Copy, Clone, Error)]
enum CliParseError<'a> {
    #[error("Unable to parse command: {0}")]
	UnknownCommand(&'a str),

    #[error("Unable to parse \"{0}\" as a number")]
    NotANumber(&'a str),

    #[error("That command requires more arguments than given")]
    NotEnoughArgs,
}

// returns a JoinHandle but you probably shouldn't join on it because it
// will block forever (or until it errors)
pub fn init_cli_thread() -> (
	thread::JoinHandle<anyhow::Result<()>>,
	sync::mpsc::Receiver<CliCommand>,
) {
	let (s, r) = sync::mpsc::channel();
	let t = thread::spawn(move || {
		if let Err(e) = cli_thread(s) {
			warn!("cli broke: {}", e);
			return Err(e);
		}

		Ok(())
	});
	(t, r)
}

fn cli_thread(out: sync::mpsc::Sender<CliCommand>) -> anyhow::Result<()> {
	let input_buffer = stdin();
	loop {
		let next_line = {
			let mut next_line = String::new();
			input_buffer.read_line(&mut next_line)?;
			next_line
		};
		let mut next_words = next_line.split(' ');

		while let Some(result) = parse_maybe_command(&mut next_words) {
			match result {
				Ok(c) => out.send(c)?,
				Err(_) => (), // Do some stuff here later
			}
		}
	}
}

fn parse_maybe_command<'a>(
	iter: &mut dyn Iterator<Item = &'a str>,
) -> Option<Result<CliCommand, CliParseError<'a>>> {
	match iter.next() {
		Some(s) => match s.trim() {
            "" => parse_maybe_command(iter),
            otherwise => Some(parse_definite_command(otherwise.trim(), iter)),
        }
		None => None,
	}
}


fn parse_definite_command<'a>(
    command_str: &'a str,
	iter: &mut dyn Iterator<Item = &'a str>,
) -> Result<CliCommand, CliParseError<'a>> {

    macro_rules! next_arg {
        () => { iter.next().ok_or(CliParseError::NotEnoughArgs)?.trim() };
    }

    match command_str {
        "zoom" => match next_arg!() {
            "set" => {
                let n_str = next_arg!();
                let n_num = n_str.parse::<f32>().map_err(|_| CliParseError::NotANumber(n_str));
                match n_num {
                    Ok(n) => Ok(CliCommand::ZoomSet(n)),
                    Err(_) => Err(CliParseError::NotANumber(n_str)),
                }
            }
            "change" => {
                let n_str = next_arg!();
                let n_num = n_str.parse::<f32>().map_err(|_| CliParseError::NotANumber(n_str));
                match n_num {
                    Ok(n) => Ok(CliCommand::ZoomChange(n)),
                    Err(_) => Err(CliParseError::NotANumber(n_str)),
                }
            }
            otherwise => Err(CliParseError::UnknownCommand(otherwise))
        }
        unparsed_command => Err(CliParseError::UnknownCommand(unparsed_command)),
    }
}
