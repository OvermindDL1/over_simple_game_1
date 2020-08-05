use thiserror::*;

use crate::core::map::coord::{Coord, CoordOrientation, CoordOrientationNeighborIterator};
use crate::core::map::generator::MapGenerator;
use crate::core::map::tile::Tile;

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

	pub fn get_tile_mut(&mut self, c: Coord) -> Option<&mut Tile> {
		let idx = c.idx(self.width, self.height, self.wraps_x)?;
		Some(&mut self.tiles[idx])
	}

	pub fn coord_to_in_map_bounds(&self, coord: Coord) -> Coord {
		let q = coord.q().rem_euclid(self.width + 1);
		let r = coord.r().rem_euclid(self.height + 1);
		Coord::new_axial(q, r)
	}

	pub fn iter_neighbors_around(
		&self,
		center: Coord,
		distance: u8,
	) -> TileMapNeighborsAroundIterator {
		TileMapNeighborsAroundIterator {
			map: self,
			center,
			iter: CoordOrientationNeighborIterator::new(distance),
		}
	}
}

pub struct TileMapNeighborsAroundIterator<'a> {
	map: &'a TileMap,
	center: Coord,
	iter: CoordOrientationNeighborIterator,
}

impl<'a> Iterator for TileMapNeighborsAroundIterator<'a> {
	type Item = (CoordOrientation, &'a Tile);

	fn next(&mut self) -> Option<Self::Item> {
		let mut co = self.iter.next()?;
		loop {
			if let Some(c) =
				self.center
					.offset_by(co, self.map.width, self.map.height, self.map.wraps_x)
			{
				if let Some(tile) = self.map.get_tile(c) {
					return Some((co, tile));
				}
			}
			co = self.iter.next()?;
		}
	}
}
