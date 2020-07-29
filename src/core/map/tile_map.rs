use crate::core::map::coord::Coord;
use crate::core::map::generator::MapGenerator;
use crate::core::map::tile::Tile;
use thiserror::*;

#[derive(Error, Debug)]
pub enum TileMapError
//<IO: SimpleIO>
//where
//	IO::ReadError: 'static,
{
	#[error("error while generating map")]
	GenerateError {
		source: anyhow::Error,
		//backtrace: Backtrace, // Still needs nightly...
	},
}

#[derive(Debug)]
pub struct TileMap {
	pub width: u8,
	pub height: u8,
	pub wraps_x: bool, // I.E. a planet
	pub tiles: Vec<Tile>,
}

impl TileMap {
	/// Creates a new TileMap
	///
	/// ```
	/// //let single_tile_map = over_simple_game_1::TileMap::new(0, 0, false, false);
	/// //let tiny_tile_map = over_simple_game_1::TileMap::new(16, 12, true, true);
	/// //let tile_map = over_simple_game_1::TileMap::new(96, 48, true, false);
	/// //let max_tile_map = over_simple_game_1::TileMap::new(255, 255, true, false);
	/// ```
	pub fn new(
		width: u8,
		height: u8,
		wraps_x: bool,
		generator: &mut impl MapGenerator,
	) -> Result<TileMap, TileMapError> {
		let mut tile_map = TileMap {
			width,
			height,
			wraps_x,
			tiles: Vec::with_capacity((width as usize + 1) * (height as usize + 1)),
		};

		generator
			.generate(&mut tile_map)
			.map_err(|source| TileMapError::GenerateError { source })?;

		Ok(tile_map)
	}

	pub fn get_tile(&self, c: Coord) -> Option<&Tile> {
		let idx = c.idx(self.width, self.height, self.wraps_x)?;
		Some(&self.tiles[idx])
	}
}
