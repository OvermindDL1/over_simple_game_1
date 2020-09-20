use std::io::{stdin, Read};
use std::{sync, thread};

use log::*;

pub enum CliCommand {}

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

		for word in next_line.split(' ') {
			match word.trim() {
				"mississippi" => info!("spelled correctly. conglaturations"),
				"" => (),
				_ => warn!("Could not understand command: {}", word),
			}
		}
	}
}
