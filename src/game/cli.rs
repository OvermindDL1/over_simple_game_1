use std::io::stdin;
use std::{sync, thread};

use log::*;

pub enum CliCommand {
	Mississppi,
}

enum CliParseError<'a> {
	UnknownCommand(&'a str),
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

		while let Some(result) = parse_command(&mut next_words) {
			match result {
				Ok(c) => out.send(c)?,
				Err(_) => (), // Do some stuff here later
			}
		}
	}
}

fn parse_command<'a>(
	iter: &mut dyn Iterator<Item = &'a str>,
) -> Option<Result<CliCommand, CliParseError<'a>>> {
	match iter.next() {
		Some(s) => Some(match s.trim() {
			"mississippi" => Ok(CliCommand::Mississppi),
			"" => parse_command(iter)?,
			unparsed_command => Err(CliParseError::UnknownCommand(unparsed_command)),
		}),
		None => None,
	}
}
