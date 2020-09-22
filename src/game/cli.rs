use std::io::stdin;
use std::{sync, thread};

use log::*;

use thiserror::*;

pub enum CliCommand {
	ZoomSet(f32),
	ZoomChange(f32),
	Clean,
}

#[derive(Debug, Clone, Error)]
enum CliParseError {
	#[error("Unable to parse command: {0}")]
	UnknownCommand(String),

	#[error("Unable to parse number: {0}")]
	NotANumber(String),

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
		let mut next_words = next_line.split(' ').map(|s| s.trim()).filter(|s| *s != "");

		while let Some(result) = parse_maybe_command(&mut next_words) {
			match result {
				Ok(c) => out.send(c)?,
				Err(e) => error!("{}", e),
			}
		}
	}
}

fn parse_maybe_command<'a>(
	iter: &mut dyn Iterator<Item = &'a str>,
) -> Option<Result<CliCommand, CliParseError>> {
	match iter.next() {
		Some(s) => Some(parse_definite_command(s, iter)),
		None => None,
	}
}

fn parse_definite_command<'a>(
	command_str: &'a str,
	iter: &mut dyn Iterator<Item = &'a str>,
) -> Result<CliCommand, CliParseError> {
	macro_rules! next_arg {
		() => {
			iter.next().ok_or(CliParseError::NotEnoughArgs)?
		};
	}

	macro_rules! parse_next_arg {
		($T:ty) => {
			next_arg!()
				.parse::<$T>()
				.map_err(|e| CliParseError::NotANumber(format!("{}", e)))
		};
	}

	match command_str {
		"zoom" => match next_arg!() {
			"set" => Ok(CliCommand::ZoomSet(parse_next_arg!(f32)?)),
			"change" => Ok(CliCommand::ZoomChange(parse_next_arg!(f32)?)),
			otherwise => Err(CliParseError::UnknownCommand(otherwise.to_owned())),
		},
		"clean" => Ok(CliCommand::Clean),
		unparsed_command => Err(CliParseError::UnknownCommand(unparsed_command.to_owned())),
	}
}
