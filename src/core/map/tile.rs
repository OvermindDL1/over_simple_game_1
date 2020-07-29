use crate::core::engine::io::EngineIO;
use serde::{Deserialize, Serialize};
use shipyard::EntityId;
use std::collections::HashMap;
use thiserror::*;

pub type TileIdx = u16;

#[derive(Debug)]
pub struct Tile {
	pub id: TileIdx,
	pub entities: Vec<EntityId>,
}

impl Tile {
	pub(crate) fn new(id: TileIdx) -> Tile {
		Tile {
			id,
			entities: vec![],
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TileType<IO: EngineIO> {
	pub name: String,
	pub interface: IO::TileInterface,
}

#[derive(Debug)]
pub struct TileTypes<IO: EngineIO> {
	pub tile_types: Vec<TileType<IO>>,
	pub lookup: HashMap<String, TileIdx>,
}

#[derive(Error, Debug)]
pub enum TileTypesError<IO: EngineIO>
where
	IO::ReadError: 'static,
{
	#[error("failed to load tiledata information file")]
	FileReadError {
		source: IO::ReadError,
		//backtrace: Backtrace, // Still needs nightly...
	},

	#[error("failed to parse tiledata information")]
	FileParseError {
		source: ron::error::Error,
		//backtrace: Backtrace, // Still needs nightly...
	},

	#[error(
		"tile types already filled to the max of {} when inserting {0}",
		TileIdx::MAX
	)]
	TileTypesFilled(String),

	#[error("tile types have already been loaded")]
	TileTypesAlreadyFilled(),

	#[error("generic invalid tile type data with message: {0}")]
	InvalidTileTypeData(String),

	#[error("attempted to insert a duplicate tile type name: {0}")]
	DuplicateTileTypeName(String),

	#[error("callback to register_tile in EngineIO failed")]
	EngineIORegisterTileError {
		source: IO::TileAddedError,
		//backtrace: Backtrace, // Still needs nightly...
	},
}

impl<IO: EngineIO> TileTypes<IO> {
	pub(crate) fn new() -> TileTypes<IO> {
		TileTypes {
			tile_types: Vec::new(),
			lookup: HashMap::new(),
		}

		// let _ = tile_datas.get_index("unknown")?;
	}

	fn add_tile(
		&mut self,
		io: &mut IO,
		mut tile_type: TileType<IO>,
	) -> Result<(), TileTypesError<IO>> {
		if tile_type.name.is_empty() {
			return Err(TileTypesError::InvalidTileTypeData("name is empty".into()));
		}
		if self.lookup.contains_key(&tile_type.name) {
			return Err(TileTypesError::DuplicateTileTypeName(tile_type.name));
		}

		let idx = self.tile_types.len();
		if idx > TileIdx::MAX as usize {
			return Err(TileTypesError::TileTypesFilled(tile_type.name));
		}

		io.tile_added(idx, &mut tile_type)
			.map_err(|source| TileTypesError::EngineIORegisterTileError { source })?;
		self.lookup.insert(tile_type.name.clone(), idx as TileIdx);
		self.tile_types.push(tile_type);
		Ok(())
	}

	pub(crate) fn load_tiles(&mut self, io: &mut IO) -> Result<(), TileTypesError<IO>> {
		if self.tile_types.len() != 0 {
			return Err(TileTypesError::TileTypesAlreadyFilled());
		}

		self.add_tile(
			io,
			TileType {
				name: "unknown".into(),
				interface: IO::blank_tile_interface(),
			},
		)?;

		let reader = io
			.read("tiles/tile_types.ron".into())
			.map_err(|source| TileTypesError::FileReadError { source })?;

		let tile_types: Vec<TileType<IO>> = ron::de::from_reader(reader)
			.map_err(|source| TileTypesError::FileParseError { source })?;

		for tile_type in tile_types {
			self.add_tile(io, tile_type)?;
		}

		Ok(())
	}

	// pub fn get_index<IO: EngineIO>(
	// 	&mut self,
	// 	io: &mut IO,
	// 	name: &str,
	// ) -> Result<TileIdx, TileDataError<IO>> {
	// 	use anyhow::Context;
	// 	if let Some(&idx) = self.lookup.get(name) {
	// 		return Ok(idx);
	// 	}
	// 	let mut reader = self
	// 		.io
	// 		.read(PathBuf::from(&format!("{}.ron", name)))
	// 		.map_err(|source| TileDataError::FileReadError {
	// 			name: name.to_string(),
	// 			source,
	// 		})?;
	// 	// let mut all = String::new();
	// 	// reader.read_to_string(&mut all);
	// 	// dbg!(all);
	// 	let tile_data: TileType =
	// 		ron::de::from_reader(reader).map_err(|source| TileDataError::FileParseError {
	// 			name: name.to_string(),
	// 			source,
	// 		})?;
	// 	let idx = self.tile_datas.len();
	// 	if idx > TileIdx::MAX as usize {
	// 		return Err(TileDataError::TileDataFilled());
	// 	}
	// 	self.tile_datas.push(tile_data);
	// 	self.lookup.insert(name.to_owned(), idx as TileIdx);
	// 	Ok(idx as TileIdx)
	// }
}