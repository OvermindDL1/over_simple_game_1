use std::io::stdin;
use std::{sync, thread};
use structopt::*;

use log::*;

#[derive(StructOpt)]
#[structopt(setting(clap::AppSettings::NoBinaryName))]
pub enum CliCommand {
	Zoom {
		#[structopt(subcommand)]
		sub: EditCommand,
	},

	View {
		x: f32,
		y: f32,
	},

	#[structopt(visible_alias("clear"))]
	Clean,

	List {
		#[structopt(subcommand)]
		sub: ListCommand,
	},

	Unit {
		index: usize,

		#[structopt(subcommand)]
		sub: UnitCommand,
	},

	Tile {
		q: u8,
		r: u8,

		#[structopt(subcommand)]
		sub: TileCommand,
	},
}

#[derive(StructOpt)]
pub enum EditCommand {
	Set { amount: f32 },
	Change { amount: f32 },
	Reset,
}

#[derive(StructOpt)]
pub enum ListCommand {
	Units,
	Tiles,
}

#[derive(StructOpt)]
pub enum UnitCommand {
	#[structopt(visible_alias("tp"))]
	Teleport { q: u8, r: u8 },
}

#[derive(StructOpt)]
pub enum TileCommand {
	Set { tile_type: String },
}

// returns a JoinHandle but you probably shouldn't join on it because it
// will block forever (or until it errors)
// TODO: replace join type with anyhow::Result<!> when the feature isn't nightly
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
			next_line.make_ascii_lowercase();
			next_line
		};
		let next_words = next_line.split(' ').map(|s| s.trim()).filter(|s| *s != "");

		match CliCommand::from_iter_safe(next_words) {
			Ok(c) => out.send(c)?,
			Err(e) => error!("{}", e),
		}
	}
}
