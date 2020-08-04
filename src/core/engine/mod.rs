pub mod io;

use thiserror::*;

use crate::core::map::generator::MapGenerator;
use crate::core::map::tile::{TileTypes, TileTypesError};
use crate::core::map::tile_map::{TileMap, TileMapError};

//use std::backtrace::Backtrace;
use std::fmt::Debug;

use crate::core::engine::io::EngineIO;
use crate::core::map::coord::MapCoord;
use crate::core::structures::typed_index_map::{TypedIndexMap, TypedIndexMapIndex};
use shipyard::{EntitiesView, EntityId, ViewMut};

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

	#[error("requested map does not exist at ID: {0:?}")]
	MapDoesNotExistsIdx(TypedIndexMapIndex<IndexMaps>),

	#[error("failed to generate tile map")]
	TileMapGenerationFailed {
		#[from]
		source: TileMapError,
		//backtrace: Backtrace, // Still needs nightly...
	},

	#[error("invalid coordinate requested on map `{map_name}` for: {coord:?}")]
	CoordIsOutOfRange { map_name: String, coord: MapCoord },
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum IndexMaps {}

pub struct Engine<IO: EngineIO> {
	pub tile_types: TileTypes<IO>,
	pub maps: TypedIndexMap<IndexMaps, String, TileMap>,
}

impl<IO: EngineIO> Engine<IO> {
	/// Creates a new game Engine.
	///
	/// ```
	/// let engine = over_simple_game_1::core::engine::Engine::<over_simple_game_1::core::engine::io::DirectFilesystemSimpleIO>::new();
	/// ```
	pub fn new() -> Engine<IO> {
		Engine {
			tile_types: TileTypes::new(),
			maps: TypedIndexMap::new(),
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

	pub fn move_entity_to_coord(
		&mut self,
		entity: EntityId,
		c: MapCoord,
		entities: EntitiesView,
		mut storage: ViewMut<MapCoord>,
	) -> Result<(), EngineError<IO>> {
		if !storage.contains(entity) {
			entities.add_component(&mut storage, c, entity);
			let (map_name, map) = self
				.maps
				.get_index_mut(c.map)
				.ok_or_else(|| EngineError::MapDoesNotExistsIdx(c.map))?;
			let tile = map
				.get_tile_mut(c.coord)
				.ok_or_else(|| EngineError::CoordIsOutOfRange {
					map_name: (*map_name).clone(),
					coord: c,
				})?;
			tile.entities.insert(entity);
		} else {
			let coord = &mut storage[entity];
			if coord.map == c.map && coord.coord != c.coord {
				let (map_name, map) = self
					.maps
					.get_index_mut(c.map)
					.ok_or_else(|| EngineError::MapDoesNotExistsIdx(c.map))?;
				// Old tile
				map.get_tile_mut(coord.coord)
					.ok_or_else(|| EngineError::CoordIsOutOfRange {
						map_name: (*map_name).clone(),
						coord: c,
					})?
					.entities
					.remove(&entity);
				// New tile
				map.get_tile_mut(c.coord)
					.ok_or_else(|| EngineError::CoordIsOutOfRange {
						map_name: (*map_name).clone(),
						coord: c,
					})?
					.entities
					.insert(entity);
				coord.coord = c.coord;
			} else if coord.coord != c.coord {
				{
					// Old tile
					let (old_map_name, old_map) = self
						.maps
						.get_index_mut(coord.map)
						.ok_or_else(|| EngineError::MapDoesNotExistsIdx(c.map))?;
					old_map
						.get_tile_mut(c.coord)
						.ok_or_else(|| EngineError::CoordIsOutOfRange {
							map_name: (*old_map_name).clone(),
							coord: c,
						})?
						.entities
						.remove(&entity);
				}
				{
					// New tile
					let (new_map_name, new_map) = self
						.maps
						.get_index_mut(coord.map)
						.ok_or_else(|| EngineError::MapDoesNotExistsIdx(c.map))?;
					new_map
						.get_tile_mut(c.coord)
						.ok_or_else(|| EngineError::CoordIsOutOfRange {
							map_name: (*new_map_name).clone(),
							coord: c,
						})?
						.entities
						.insert(entity);
				}
				*coord = c;
			}
		}

		Ok(())
	}
}
