use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use shipyard::EntityId;
use thiserror::*;

use crate::core::engine::io::EngineIO;
use crate::core::structures::typed_index_map::{
	TypedIndexMap, TypedIndexMapError, TypedIndexMapIndex,
};

#[derive(Clone, Copy, Debug)]
pub enum TileTypesMap {}

pub type TileIdx = TypedIndexMapIndex<TileTypesMap, u16>;

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

#[derive(Debug)]
pub struct TileTypes<IO: EngineIO> {
	pub tile_types: TypedIndexMap<TileTypesMap, String, TileType<IO>, u16>,
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
		"tile types already filled to the max of {} when inserting {}",
		TileIdx::MAX, .0.name
	)]
	TileTypesFilled(TileType<IO>),

	#[error("tile types have already been loaded")]
	TileTypesAlreadyFilled(),

	#[error("generic invalid tile type data with message: {0}")]
	InvalidTileTypeData(String, TileType<IO>),

	#[error("attempted to insert a duplicate tile type name: {}", .0.name)]
	DuplicateTileTypeName(TileType<IO>),

	#[error("callback to register_tile in EngineIO failed")]
	EngineIORegisterTileError {
		source: IO::TileAddedError,
		//backtrace: Backtrace, // Still needs nightly...
		tile_type: TileType<IO>,
	},
}

impl<IO: EngineIO> TileTypes<IO> {
	pub(crate) fn new() -> TileTypes<IO> {
		TileTypes {
			tile_types: TypedIndexMap::new(),
		}
	}

	fn add_tile(
		&mut self,
		io: &mut IO,
		mut tile_type: TileType<IO>,
	) -> Result<(), TileTypesError<IO>> {
		if tile_type.name.is_empty() {
			return Err(TileTypesError::InvalidTileTypeData(
				"name is empty".into(),
				tile_type,
			));
		}
		if self.tile_types.contains_key(&tile_type.name) {
			return Err(TileTypesError::DuplicateTileTypeName(tile_type));
		}

		let (index, old_value) = self
			.tile_types
			.insert_full(tile_type.name.clone(), tile_type)
			.map_err(|e| match e {
				TypedIndexMapError::TypedIndexMapFull(_max, _key, tile_type) => {
					TileTypesError::TileTypesFilled(tile_type)
				}
			})?;

		assert!(old_value.is_none());

		match io.tile_added(
			index,
			self.tile_types
				.get_index_mut(index)
				.expect("unable to lookup just inserted value")
				.1,
		) {
			Ok(()) => Ok(()),
			Err(source) => {
				let (_name, tile_type) = self
					.tile_types
					.pop()
					.expect("unable to pop just inserted value?");
				Err(TileTypesError::EngineIORegisterTileError { source, tile_type })
			}
		}
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
	use std::{convert::Infallible, hash::Hasher, path::PathBuf};

	#[derive(Debug, Default, Eq, PartialEq)]
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
			_: TileIdx,
			_: &mut TileType<Self>,
		) -> Result<(), Self::TileAddedError> {
			Ok(())
		}
	}

	impl PartialEq for TileType<DummyIO> {
		fn eq(&self, other: &Self) -> bool {
			self.name == other.name
		}
	}
	impl Eq for TileType<DummyIO> {}

	impl std::hash::Hash for TileType<DummyIO> {
		fn hash<H: Hasher>(&self, state: &mut H) {
			self.name.hash(state);
		}
	}

	fn tiletype_strategy_generator(regex: &str) -> BoxedStrategy<TileType<DummyIO>> {
		prop::string::string_regex(regex)
			.expect("failed to generate strategy from regex")
			.prop_map(|s| TileType {
				name: s,
				interface: (),
			})
			.boxed()
	}

	fn rand_dummy_tiletype_strategy() -> BoxedStrategy<TileType<DummyIO>> {
		tiletype_strategy_generator(".*")
	}

	fn non_empty_tiletype_strategy() -> BoxedStrategy<TileType<DummyIO>> {
		tiletype_strategy_generator(".+")
	}

	proptest!(
		#[test]
		fn first_tiletype_should_be_unique(tt in rand_dummy_tiletype_strategy()) {
			let mut dummy_io = DummyIO::default();
			let mut tts = TileTypes::new();
			if let Err(TileTypesError::DuplicateTileTypeName(tile_type)) = tts.add_tile(&mut dummy_io, tt) {
				prop_assert!(false, "TileType {} marked as duplicate, but it is the only one added", tile_type.name);
			}
		}
	);

	proptest!(
		#[test]
		fn non_empty_tiletypes_are_valid(tt in non_empty_tiletype_strategy()) {
			let mut dummy_io = DummyIO::default();
			let mut tts = TileTypes::new();
			if let Err(TileTypesError::InvalidTileTypeData(reason, tile_type)) = tts.add_tile(&mut dummy_io, tt) {
				prop_assert!(false, "TileType {} marked invalid because {}", tile_type.name, reason);
			}
		}
	);

	proptest!(
		#![proptest_config(ProptestConfig::with_cases(30))]
		#[test]
		fn many_valid_tiletypes_get_accepted(
			// I would use 2..TileIdx::MAX for the size but it pins the cpu
			tt_set in prop::collection::hash_set(non_empty_tiletype_strategy(), 2..500)
		) {
			let mut dummy_io = DummyIO::default();
			let mut tts = TileTypes::new();
			for tt in tt_set {
				// I would use prop_assert_ne!(add_tile, Ok(())) but it needs PartialEq
				if let Err(e) = tts.add_tile(&mut dummy_io, tt) {
					prop_assert!(false, "{}", e);
				}
			}
		}
	);

	#[test]
	fn empty_tile_types_are_rejected() {
		let tt = TileType::<DummyIO> {
			name: String::from(""),
			interface: (),
		};
		let mut dummy_io = DummyIO::default();
		let mut tts = TileTypes::new();

		if let Err(TileTypesError::InvalidTileTypeData(reason, tile_type)) =
			tts.add_tile(&mut dummy_io, tt)
		{
			if reason != "name is empty" {
				panic!(
					"empty string marked with incorrect error: {}",
					tile_type.name
				);
			}
		} else {
			panic!("empty string not marked as error");
		}
	}
}
