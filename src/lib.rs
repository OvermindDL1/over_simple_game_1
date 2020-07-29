use anyhow::Context as AnyContext;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Debug;
use std::ops::{Add, Neg, Sub};
use std::path::PathBuf;
use thiserror::Error;
//use std::backtrace::Backtrace;

/// A simple Input/Output interface to get readers/writers from file names.
///
/// Implement this to allow the engine to be able to talk to storage.
pub trait SimpleIO: Debug + Sized {
	type ReadError: std::error::Error + Send + Sync;
	type Read: std::io::Read;
	fn read(&mut self, file_path: PathBuf) -> Result<Self::Read, Self::ReadError>;

	type TileInterface: Debug + Serialize + DeserializeOwned;
	fn blank_tile_interface() -> Self::TileInterface;

	type TileAddedError: std::error::Error + Send + Sync + 'static;
	fn tile_added(
		&mut self,
		index: usize,
		tile_type: &mut TileType<Self>,
	) -> Result<(), Self::TileAddedError>;
}

/// Hex Coordinates, cubic notation but axial stored.
///
/// This creates a rhombus shape of hex tiles, 0,0 in top left, 255,255 in bottom right, each row
/// down the rhombus shifts a half tile right compared to the prior.  Wraps cleanly left/right.
///
/// Coordinates can be acquired either via cubic via x/y/z (x/z stored, y calculated) or via axial q/r.
///
/// Axial coordinates are just cubic's x/z where y is -x-z since x+y+z=0 to get the cubic plane.
///
/// ```
/// let coord = over_simple_game_1::CoordHex::new_axial(0, 1);
/// assert_eq!(coord.x(), 0);
/// assert_eq!(coord.y(), -1);
/// assert_eq!(coord.z(), 1);
/// assert_eq!(coord.q(), 0);
/// assert_eq!(coord.r(), 1);
/// ```
#[derive(Clone, Copy, Default, Debug, Hash, PartialOrd, PartialEq, Ord, Eq)]
pub struct CoordHex(i8, i8);

// #[derive(Clone, Copy, Debug, Hash, PartialOrd, PartialEq, Ord, Eq)]
// pub enum CoordHexDirection {
// 	XY = 0,
// 	ZY = 1,
// 	ZX = 2,
// 	YX = 3,
// 	YZ = 4,
// 	XZ = 5,
// }
//
// impl CoordHexDirection {
// 	pub fn as_coord(self) -> CoordHex {
// 		match self {
// 			XY => CoordHex::new_cubic(1, 255, 0),
// 			ZY => CoordHex::new_cubic(0, 255, 1),
// 			ZX => CoordHex::new_cubic(255, 0, 1),
// 			YX => CoordHex::new_cubic(255, 1, 0),
// 			YZ => CoordHex::new_cubic(0, 1, 255),
// 			XZ => CoordHex::new_cubic(1, 0, 255),
// 		}
// 	}
//
// 	pub fn cw(self) -> CoordHexDirection {}
// }
//
// pub const COORD_HEX_DIRECTIONS: [CoordHexDirection; 6] = [
// 	CoordHexDirection::XY,
// 	CoordHexDirection::ZY,
// 	CoordHexDirection::ZX,
// 	CoordHexDirection::YX,
// 	CoordHexDirection::YZ,
// 	CoordHexDirection::XZ,
// ];

impl CoordHex {
	// pub const DIRECTIONS: [CoordHexDirection; 6] = COORD_HEX_DIRECTIONS;

	/// Uses axial coordinates to create a new `CoordHex`
	///
	/// Axial coordinate, 0,0 is top-left, 255,255 is bottom-right.
	///
	/// ```
	/// let coord = over_simple_game_1::CoordHex::new_axial(0, 1);
	/// assert_eq!(coord.q(), 0);
	/// assert_eq!(coord.r(), 1);
	/// ```
	pub fn new_axial(q: i8, r: i8) -> CoordHex {
		CoordHex(q, r)
	}

	/// Uses cubic coordinates to create a new `CoordHex`
	///
	/// cubic coordinates are the 3 axis of a 3d Cube, though constrained to the diagonal plane as
	/// `x + y + z = 0`.
	///
	/// ```
	/// let coord = over_simple_game_1::CoordHex::new_axial(0, 1);
	/// assert_eq!(coord.x(), 0);
	/// assert_eq!(coord.y(), -1);
	/// assert_eq!(coord.z(), 1);
	/// ```
	pub fn new_cubic(x: i8, y: i8, z: i8) -> CoordHex {
		assert_eq!(x.wrapping_add(y).wrapping_add(z), 0);
		CoordHex(x, z)
	}

	/// Uses linear (pixel) coordinate to create a new `CoordHex`
	///
	/// ```
	/// # use over_simple_game_1::CoordHex;
	/// assert_eq!(CoordHex::from_linear(0.0, 0.0), CoordHex::new_axial(0, 0));
	/// assert_eq!(CoordHex::from_linear(1.0, 1.0), CoordHex::new_axial(1, 1));
	/// assert_eq!(CoordHex::from_linear(-1.0, -1.0), CoordHex::new_axial(-1, -1));
	/// assert_eq!(CoordHex::from_linear(1.0, -1.0), CoordHex::new_axial(1, -1));
	/// assert_eq!(CoordHex::from_linear(-1.0, 1.0), CoordHex::new_axial(-1, 1));
	/// ```
	pub fn from_linear(x: f32, y: f32) -> CoordHex {
		let x = x - y * 0.5;
		// let q = x.round().rem(256.0) as i8;
		// let r = y.round().rem(256.0) as i8;
		// let q = (x.round() as isize & 0xFF) as u8;
		// let r = (y.round() as isize & 0xFF) as u8;
		let q = x.round() as i8;
		let r = y.round() as i8;
		CoordHex::new_axial(q, r)
		// let s3 = 3.0f32.sqrt();
		// let is3 = 1.0 / s3;
		// let fx = (-2.0 / 3.0) * x;
		// let fy = (1.0 / 3.0) * x + is3 * y;
		// let fz = (1.0 / 3.0) * x - is3 * y;
		// let a = (fx - fy).ceil();
		// let b = (fy - fz).ceil();
		// let c = (fz - fx).ceil();
		// let x = ((a - c) / 3.0).round() as i8;
		// let y = ((b - a) / 3.0).round() as i8;
		// let z = ((c - b) / 3.0).round() as i8;
		// CoordHex::new_cubic(x, y, z)
		// let s3 = 3.0f32.sqrt();
		// x /= s3;
		// y /= s3;
		// let p = (x + s3 * y + 1.0).sqrt();
		// let q = (((2.0 * x + 1.0).floor() + p) / 3.0).floor();
		// let r = ((((-x + s3 * y + 1.0).floor()) + p) / 3.0).floor();
		// CoordHex::new_axial(q as i8, r as i8)
	}

	pub fn to_linear(self) -> (f32, f32) {
		let q = self.0 as f32;
		let r = self.1 as f32;
		let offset_x = r * 0.5;
		(q + offset_x, r)
	}

	pub fn q(&self) -> i8 {
		self.0
	}

	pub fn r(&self) -> i8 {
		self.1
	}

	pub fn to_axial_tuple(&self) -> (i8, i8) {
		(self.q(), self.r())
	}

	pub fn x(&self) -> i8 {
		self.0
	}

	pub fn y(&self) -> i8 {
		self.0.wrapping_neg().wrapping_sub(self.1)
	}

	pub fn z(&self) -> i8 {
		self.1
	}

	pub fn to_cubic_tuple(&self) -> (i8, i8, i8) {
		(self.x(), self.y(), self.z())
	}

	pub fn idx(self, max_x: u8, max_z: u8, wraps_x: bool) -> Option<usize> {
		if self.1 as u8 > max_z || (!wraps_x && self.0 as u8 > max_x) {
			return None;
		}
		let x = (self.0 as u8) as usize % (max_x as usize + 1);
		let z = (self.1 as u8) as usize;
		Some((z * max_x as usize) + x)

		// let x = (self.0 as u8) as usize % (max_x as usize + 1);
		// let z = (self.1 as u8) as usize % (max_z as usize + 1);
		// (z * max_x as usize) + x

		// let y1 = self.1 as u8;
		// let y2 = y1 as usize;
		// let m1 = max_x as usize;
		// let x1 = self.0 as u8;
		// let x2 = x1 as usize;
		// let y3 = y2 * m1;
		// let r = y3 + x2;
		// r

		// (self.1 as u8 as usize * max_x as usize) + self.0 as u8 as usize
	}

	pub fn scale(self, scale: i8) -> CoordHex {
		CoordHex(self.0.wrapping_mul(scale), self.1.wrapping_mul(scale))
	}

	pub fn cw(self) -> CoordHex {
		let (x, y, z) = (-self).to_cubic_tuple();
		CoordHex::new_cubic(z, x, y)
	}

	pub fn ccw(self) -> CoordHex {
		let (x, y, z) = (-self).to_cubic_tuple();
		CoordHex::new_cubic(y, z, x)
	}

	pub fn cw_offset(self, center: CoordHex) -> CoordHex {
		(center - self).cw() + center
	}

	pub fn ccw_offset(self, center: CoordHex) -> CoordHex {
		(center - self).ccw() + center
	}

	// pub fn iterate_linear_box(self, to: CoordHex) -> CoordLinearViewIterator {
	// 	let offset_x: i8 = to.1.wrapping_sub(self.1).wrapping_add(1).wrapping_div(2);
	// 	let stride: i8 = to.0.wrapping_sub(self.0).wrapping_add(offset_x);
	// 	CoordLinearViewIterator {
	// 		stride,
	// 		stride_remaining: stride,
	// 		from: self,
	// 		current: self,
	// 		to,
	// 		done: false,
	// 	}
	// }

	pub fn iter_neighbors_ring(self, distance: i8) -> CoordHexRingIterator {
		CoordHexRingIterator::new(self, distance)
	}

	pub fn iter_neighbors(self, distance: i8) -> CoordHexNeighborIterator {
		CoordHexNeighborIterator::new(self, distance)
	}
}

impl Add for CoordHex {
	type Output = CoordHex;

	fn add(self, rhs: Self) -> Self::Output {
		CoordHex(self.0.wrapping_add(rhs.0), self.1.wrapping_add(rhs.1))
	}
}

impl Sub for CoordHex {
	type Output = CoordHex;

	fn sub(self, rhs: Self) -> Self::Output {
		CoordHex(self.0.wrapping_sub(rhs.0), self.1.wrapping_sub(rhs.1))
	}
}

impl Neg for CoordHex {
	type Output = CoordHex;

	fn neg(self) -> Self::Output {
		CoordHex(-self.0, -self.1)
	}
}

// // Unsure if this is good...
// pub struct CoordLinearViewIterator {
// 	stride: i8,
// 	stride_remaining: i8,
// 	from: CoordHex,
// 	current: CoordHex,
// 	to: CoordHex,
// 	done: bool,
// }
//
// impl Iterator for CoordLinearViewIterator {
// 	type Item = CoordHex;
//
// 	fn next(&mut self) -> Option<Self::Item> {
// 		if self.done {
// 			return None;
// 		}
// 		let coord = if self.stride_remaining == 0 {
// 			if self.current.1 == self.to.1 {
// 				self.done = true;
// 				self.current
// 			} else {
// 				let coord = self.current;
// 				self.current.1 = self.current.1.wrapping_add(1);
// 				let offset_x = self
// 					.current
// 					.1
// 					.wrapping_sub(self.from.1)
// 					.wrapping_add(1)
// 					.wrapping_div(2);
// 				self.current.0 = self.from.0.wrapping_add(offset_x);
// 				self.stride_remaining = self.stride;
// 				coord
// 			}
// 		} else {
// 			let coord = self.current;
// 			self.current.0 = self.current.0.wrapping_add(1);
// 			self.stride_remaining -= 1;
// 			coord
// 		};
// 		Some(coord)
// 	}
// }

pub struct CoordHexRingIterator {
	point: Option<CoordHex>,
	side: CoordHex,
	distance: i8,
	offset: i8,
}

impl CoordHexRingIterator {
	fn new(center: CoordHex, distance: i8) -> CoordHexRingIterator {
		if distance == 0 {
			CoordHexRingIterator {
				point: Some(center),
				side: CoordHex(-1, 1).ccw(),
				distance: 0,
				offset: 0,
			}
		} else {
			let side = CoordHex(1, 0);
			CoordHexRingIterator {
				point: Some(center + side.scale(distance)),
				side: (-side).ccw(),
				distance,
				offset: 0,
			}
		}
	}
}

impl Iterator for CoordHexRingIterator {
	type Item = CoordHex;

	fn next(&mut self) -> Option<Self::Item> {
		let point = self.point? + self.side.scale(self.offset);
		if self.offset >= self.distance {
			self.offset = 1;
			self.side = self.side.cw();
			if self.side == CoordHex(-1, 1) {
				self.point = None;
			} else {
				self.point = Some(point)
			}
		} else {
			self.offset += 1;
		}
		Some(point)
	}
}

pub struct CoordHexNeighborIterator {
	ring_iter: CoordHexRingIterator,
	center: CoordHex,
	distance: i8,
}

impl CoordHexNeighborIterator {
	fn new(center: CoordHex, distance: i8) -> CoordHexNeighborIterator {
		CoordHexNeighborIterator {
			ring_iter: CoordHexRingIterator::new(center, 0),
			center,
			distance,
		}
	}
}

impl Iterator for CoordHexNeighborIterator {
	type Item = CoordHex;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(coord) = self.ring_iter.next() {
			return Some(coord);
		}
		if self.distance <= self.ring_iter.distance {
			return None;
		}
		self.ring_iter = CoordHexRingIterator::new(self.center, self.ring_iter.distance + 1);
		self.ring_iter.next()
	}
}

// #[cfg(test)]
// mod coord_hex_tests {
// 	#[test]
// 	fn from_linear_test() {
// 	}
// }

pub trait MapGenerator {
	fn generate(&mut self, tile_map: &mut TileMap) -> anyhow::Result<()>;
}

pub struct SimpleAlternationMapGenerator(Vec<TileIdx>);
impl MapGenerator for SimpleAlternationMapGenerator {
	fn generate(&mut self, tile_map: &mut TileMap) -> Result<(), anyhow::Error> {
		tile_map.tiles.clear();
		for y in 0usize..=(tile_map.width as usize) {
			for x in 0usize..=(tile_map.height as usize) {
				let idx = (y * tile_map.height as usize) + x;
				let tile_idx = self.0[idx % self.0.len()];
				tile_map.tiles.push(Tile::new(tile_idx));
			}
		}

		Ok(())
	}
}
impl SimpleAlternationMapGenerator {
	pub fn new<'a, NameIter: IntoIterator, IO: SimpleIO>(
		engine: &mut Engine<IO>,
		names: NameIter,
	) -> Result<SimpleAlternationMapGenerator, anyhow::Error>
	where
		NameIter::Item: AsRef<str>,
	{
		let mut tiles = Vec::new();
		for name in names {
			let name: &str = name.as_ref();
			let idx = engine
				.tile_types
				.lookup
				.get(name)
				.cloned()
				.with_context(|| format!("missing tile type: {}", name))?;
			tiles.push(idx)
		}
		Ok(SimpleAlternationMapGenerator(tiles))
	}
}

type TileIdx = u16;

#[derive(Debug)]
pub struct Tile {
	pub id: TileIdx,
	pub entities: Vec<shipyard::EntityId>,
}

impl Tile {
	fn new(id: TileIdx) -> Tile {
		Tile {
			id,
			entities: vec![],
		}
	}
}

#[derive(Error, Debug)]
pub enum TileMapError
//<IO: SimpleIO>
//where
//	IO::ReadError: 'static,
{
	#[error("error whilegenerating map")]
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

	pub fn get_tile(&self, c: CoordHex) -> Option<&Tile> {
		let idx = c.idx(self.width, self.height, self.wraps_x)?;
		Some(&self.tiles[idx])
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TileType<IO: SimpleIO> {
	pub name: String,
	pub interface: IO::TileInterface,
}

#[derive(Debug)]
pub struct TileTypes<IO: SimpleIO> {
	pub tile_types: Vec<TileType<IO>>,
	pub lookup: HashMap<String, TileIdx>,
}

#[derive(Error, Debug)]
pub enum TileTypesError<IO: SimpleIO>
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

	#[error("callback to register_tile in SimpleIO failed")]
	SimpleIORegisterTileError {
		source: IO::TileAddedError,
		//backtrace: Backtrace, // Still needs nightly...
	},
}

impl<IO: SimpleIO> TileTypes<IO> {
	fn new() -> TileTypes<IO> {
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
			.map_err(|source| TileTypesError::SimpleIORegisterTileError { source })?;
		self.lookup.insert(tile_type.name.clone(), idx as TileIdx);
		self.tile_types.push(tile_type);
		Ok(())
	}

	fn load_tiles(&mut self, io: &mut IO) -> Result<(), TileTypesError<IO>> {
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

	// pub fn get_index<IO: SimpleIO>(
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

#[derive(Error, Debug)]
pub enum EngineError<IO: SimpleIO + 'static> {
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

pub struct Engine<IO: SimpleIO> {
	ecs: shipyard::World,
	pub tile_types: TileTypes<IO>,
	pub maps: HashMap<String, TileMap>,
}

impl<IO: SimpleIO> Engine<IO> {
	/// Creates a new game Engine.
	///
	/// ```
	/// let engine = over_simple_game_1::Engine::new();
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
		io: &mut IO,
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

#[derive(Debug)]
pub struct DirectFSIO(pub std::path::PathBuf);

impl DirectFSIO {
	pub fn new(base_path: &str) -> DirectFSIO {
		DirectFSIO::with_path(std::path::PathBuf::from(base_path))
	}

	pub fn with_path(base_path: std::path::PathBuf) -> DirectFSIO {
		DirectFSIO(base_path)
	}

	pub fn with_cwd() -> DirectFSIO {
		let path = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
			std::path::PathBuf::from(manifest_dir)
		} else {
			std::path::PathBuf::from(".")
		};
		DirectFSIO::with_path(path)
	}
}

impl SimpleIO for DirectFSIO {
	type ReadError = std::io::Error;
	type Read = std::fs::File;

	fn read(&mut self, file_path: PathBuf) -> Result<Self::Read, Self::ReadError> {
		let mut path = self.0.clone();
		path.push(file_path);
		std::fs::File::open(path)
	}

	type TileInterface = ();

	fn blank_tile_interface() -> Self::TileInterface {
		()
	}

	type TileAddedError = Infallible;

	fn tile_added(
		&mut self,
		_index: usize,
		_tile_type: &mut TileType<Self>,
	) -> Result<(), Self::TileAddedError> {
		Ok(())
	}
}

pub mod prelude {
	pub use super::CoordHex;
	pub use super::Engine;
	pub use super::SimpleAlternationMapGenerator;
	pub use super::SimpleIO;
}
