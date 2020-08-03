use crate::core::engine::io::EngineIO;
use crate::core::structures::typed_index_map::TypedIndexMap;
use serde::{Deserialize, Serialize};
use shipyard::EntityId;
use std::collections::HashSet;
use thiserror::*;

pub type TileIdx = u16;

#[derive(Debug)]
pub struct Tile {
	pub id: TileIdx,
	pub entities: HashSet<EntityId>,
}

impl Tile {
	pub(crate) fn new(id: TileIdx) -> Tile {
		Tile {
			id,
			entities: HashSet::new(),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TileType<IO: EngineIO> {
	pub name: String,
	pub interface: IO::TileInterface,
}

pub enum TileTypesMap {}

#[derive(Debug)]
pub struct TileTypes<IO: EngineIO> {
	pub tile_types: TypedIndexMap<TileTypesMap, String, TileType<IO>>,
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
			tile_types: TypedIndexMap::new(),
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
		if self.tile_types.contains_key(&tile_type.name) {
			return Err(TileTypesError::DuplicateTileTypeName(tile_type.name));
		}

		let idx = self.tile_types.len();
		if idx > TileIdx::MAX as usize {
			return Err(TileTypesError::TileTypesFilled(tile_type.name));
		}

		io.tile_added(idx, &mut tile_type)
			.map_err(|source| TileTypesError::EngineIORegisterTileError { source })?;
		self.tile_types.insert(tile_type.name.clone(), tile_type);
		Ok(())
	}

	pub(crate) fn load_tiles(&mut self, io: &mut IO) -> Result<(), TileTypesError<IO>> {
		if !self.tile_types.is_empty() {
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

#[cfg(test)]
mod tile_tests {
	use super::*;
	use proptest::prelude::*;
	use std::{convert::Infallible, path::PathBuf};

	#[derive(Debug, Default)]
	struct DummyIO {}

	impl EngineIO for DummyIO {
		type ReadError = Infallible;
		type Read = &'static [u8];

		fn read(&mut self, _: PathBuf) -> Result<Self::Read, Self::ReadError> {
			Ok(b"")
		}

		type TileInterface = ();

		fn blank_tile_interface() -> Self::TileInterface {}

		type TileAddedError = Infallible;

		fn tile_added(
			&mut self,
			_: usize,
			_: &mut TileType<Self>,
		) -> Result<(), Self::TileAddedError> {
			Ok(())
		}
	}

	fn rand_dummy_tiletype_strategy() -> BoxedStrategy<TileType<DummyIO>> {
		any::<String>()
			.prop_map(|s| TileType {
				name: s,
				interface: (),
			})
			.boxed()
	}

    fn non_empty_tiletype_strategy() -> BoxedStrategy<TileType<DummyIO>> {
        rand_dummy_tiletype_strategy()
            .prop_filter("empty tiletype", |t| t.name != "")
            .boxed()
    }

	proptest!(
		#[test]
		fn first_tiletype_should_be_unique(tt in rand_dummy_tiletype_strategy()) {
			let mut dummy_io = DummyIO::default();
			let mut tts = TileTypes::new();
			if let Err(TileTypesError::DuplicateTileTypeName(_)) = tts.add_tile(&mut dummy_io, tt) {
				prop_assert!(false);
			}
		}
	);

    proptest!(
        #[test]
        fn non_empty_tiletypes_are_valid(tt in non_empty_tiletype_strategy()) {
			let mut dummy_io = DummyIO::default();
			let mut tts = TileTypes::new();
            let name = tt.name.clone();     // I really don't want to do this, but add_tile consumes
			if let Err(TileTypesError::InvalidTileTypeData(s)) = tts.add_tile(&mut dummy_io, tt) {
				prop_assert!(false, "TileType {} marked invalid because {}", name, s);
			}
        }
    );
}
