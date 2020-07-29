pub mod io;

use std::collections::HashMap;

use thiserror::*;

use crate::core::map::generator::MapGenerator;
use crate::core::map::tile::{TileTypes, TileTypesError};
use crate::core::map::tile_map::{TileMap, TileMapError};

//use std::backtrace::Backtrace;
use std::fmt::Debug;

use crate::core::engine::io::EngineIO;

#[derive(Error, Debug)]
pub enum EngineError<IO: EngineIO + 'static> {
	#[error("failed to load tiledata")]
	TileDataError {
		#[from]
		source: TileTypesError<IO>,
		//backtrace: Backtrace, // Still needs nightly...
	},

	#[error("cannot generate map as it already exists: {0}")]
	MapAlreadyExists(String),

	#[error("requested map does not exist: {0}")]
	MapDoesNotExists(String),

	#[error("failed to generate tile map")]
	TileMapGenerationFailed {
		#[from]
		source: TileMapError,
		//backtrace: Backtrace, // Still needs nightly...
	},
}

pub struct Engine<IO: EngineIO> {
	pub ecs: shipyard::World,
	pub tile_types: TileTypes<IO>,
	pub maps: HashMap<String, TileMap>,
}

impl<IO: EngineIO> Engine<IO> {
	/// Creates a new game Engine.
	///
	/// ```
	/// let engine = over_simple_game_1::core::engine::Engine::<over_simple_game_1::core::engine::io::DirectFilesystemSimpleIO>::new();
	/// ```
	pub fn new() -> Engine<IO> {
		Engine {
			ecs: shipyard::World::new(),
			tile_types: TileTypes::new(),
			maps: HashMap::new(),
		}
	}

	pub fn setup(&mut self, io: &mut IO) -> Result<(), EngineError<IO>> {
		self.tile_types.load_tiles(io)?;

		Ok(())
	}

	pub fn generate_map(
		&mut self,
		_io: &mut IO,
		name: impl ToString,
		max_x: u8,
		max_y: u8,
		wraps_x: bool,
		generator: &mut impl MapGenerator,
	) -> Result<(), EngineError<IO>> {
		let name = name.to_string();
		if self.maps.contains_key(&name) {
			return Err(EngineError::MapAlreadyExists(name));
		}

		let tile_map = TileMap::new(max_x, max_y, wraps_x, generator)?;
		self.maps.insert(name, tile_map);

		Ok(())
	}
}
