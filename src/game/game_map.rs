use ggez::graphics::Rect;
use shipyard::EntityId;
use std::collections::HashMap;
use std::fmt::Debug;
use std::num::NonZeroI8;

/*
pub struct GeodesicSubdivision {
	planes: [Box<GeodesicPlane>; 4],
	top_point: bool,
}

pub struct GeodesicData {}

pub enum Geodesic /*Trianguloid*/ {
	Subdivision(GeodesicSubdivision),
	Data(GeodesicData),
}

impl Geodesic {
	fn new(subdivisions: u8, is_top_point: bool) -> Geodesic {
		if subdivisions > 4 {
			panic!("Invalid subdivision count for new Geodesix, valid range is up to and including 4, value given:  {}", subdivisions);
		}
		if subdivisions == 0 {
			Geodesic::Data(GeodesicData {})
		} else {
			Geodesic::Subdivision(GeodesicSubdivision {})
		}
	}
}

#[cfg(test)]
mod geodesic_tests {
	#[test]
	fn geodesic_test() {
		let geo = Geodesic::new();
	}
}
*/

/*
pub struct GeodesicRhombus {
	tiles: Vec<Tile>,
}

pub struct GeodesicLongitude {
	rhomubses: Vec<GeodesicRhombus>,
}

pub struct GeodesicLatitude {
	longitude: Vec<GeodesicLongitude>,
}

pub struct GeodesicMap {
	latitude: GeodesicLatitude,
}
*/

// const COORD_IN_CHUNK_MASk: i8 = 0b0000_1111;
// const CHUNK_COORD_MASK: i8 = 0b1111_0000;
// const COORD_IN_CHUNK_STRIDE: usize = COORD_IN_CHUNK_MASk as usize + 1;
// const CHUNK_STRIDE: usize = ((CHUNK_COORD_MASK as usize) >> 4) + 1;
// const COORDS_IN_CHUNK_SIZE: usize = (COORD_IN_CHUNK_STRIDE as usize).pow(2);
// const CHUNK_COUNT: usize = (CHUNK_STRIDE as usize + 1).pow(2);
//
// #[derive(Clone, Copy, Default, Debug, Hash, PartialOrd, PartialEq, Ord, Eq)]
// pub struct Coord(i8, i8);
//
// impl Coord {
// 	pub fn new(x: i8, y: i8, z: i8) -> Coord {
// 		assert_eq!(x + y + z, 0);
// 		Coord(x, y)
// 	}
//
// 	pub fn from_idx(idx: CoordIdx) -> Coord {
// 		let x = (idx.0 >> 8) as i8;
// 		let y = (idx.0 & 0xFF) as i8;
// 		Coord(x, y)
// 	}
//
// 	pub fn from_idxs(chunk: ChunkIdx, coord: ChunkCoordIdx) -> Coord {
// 		let x = (chunk.0 & 0b1111_0000) + (coord.0 >> 4);
// 		let y = (chunk.0 << 4) + (coord.0 & 0b0000_1111);
// 		Coord(x, y)
// 	}
//
// 	pub fn x(&self) -> i8 {
// 		self.0
// 	}
//
// 	pub fn y(&self) -> i8 {
// 		self.1
// 	}
//
// 	pub fn z(&self) -> i8 {
// 		-self.0 - self.1
// 	}
//
// 	pub fn as_tuple(&self) -> (i8, i8, i8) {
// 		(self.0, self.1, self.z())
// 	}
//
// 	pub fn as_chunk_tuple(&self) -> (i8, i8, i8) {
// 		(
// 			(self.0 & 0b1111_0000) >> 4,
// 			(self.1 & 0b1111_0000) >> 4,
// 			(self.z() & 0b1111_0000) >> 4,
// 		)
// 	}
//
// 	pub fn as_chunk_tuple_shifted(&self) -> (i8, i8, i8) {
// 		(
// 			self.0 & 0b1111_0000,
// 			self.1 & 0b1111_0000,
// 			self.z() & 0b1111_0000,
// 		)
// 	}
//
// 	pub fn as_chunk_coord_tuple(&self) -> (i8, i8, i8) {
// 		(
// 			self.0 & 0b0000_1111,
// 			self.1 & 0b0000_1111,
// 			self.z() & 0b0000_1111,
// 		)
// 	}
//
// 	pub fn idx(&self) -> CoordIdx {
// 		CoordIdx(((self.0 as i16) << 8) + self.1 as i16)
// 	}
//
// 	pub fn chunk_idx(&self) -> ChunkIdx {
// 		ChunkIdx((self.0 & 0b1111_0000) + (self.1 >> 4))
// 	}
//
// 	pub fn chunk_coord_idx(&self) -> ChunkCoordIdx {
// 		ChunkCoordIdx((self.1 << 4) + (self.1 & 0b0000_1111))
// 	}
//
// 	pub fn iterate_coords_in_chunk(&self) -> CoordsInChunkIterator {
// 		CoordsInChunkIterator {
// 			coord: Coord(self.0 & CHUNK_COORD_MASK, self.1 & CHUNK_COORD_MASK),
// 			done: false,
// 		}
// 	}
// }
//
// struct CoordsInChunkIterator {
// 	coord: Coord,
// 	done: bool,
// }
// impl Iterator for CoordsInChunkIterator {
// 	type Item = Coord;
//
// 	fn next(&mut self) -> Option<Self::Item> {
// 		if self.done {
// 			return None;
// 		}
// 		let coord = self.coord;
//
// 		if (self.coord.0 & COORD_IN_CHUNK_MASk) == COORD_IN_CHUNK_MASk {
// 			if (self.coord.1 & COORD_IN_CHUNK_MASk) == COORD_IN_CHUNK_MASk {
// 				self.done = true;
// 				return None;
// 			}
// 			self.coord.0 = self.coord.0 & CHUNK_COORD_MASK;
// 			self.coord.1 += 1;
// 		} else {
// 			self.coord.0 += 1;
// 		}
//
// 		Some(coord)
// 	}
//
// 	fn size_hint(&self) -> (usize, Option<usize>) {
// 		let x = COORD_IN_CHUNK_STRIDE - (self.0 as usize)
// 		let z = COORD_IN_CHUNK_STRIDE - self.x() as usize;
// 		let y = COORD_IN_CHUNK_STRIDE - self.y() as usize;
// 		let x = COORD_IN_CHUNK_STRIDE - self.z() as usize;
// 		let remaining = x * y * z;
// 		(remaining, Some(remaining))
// 	}
// }
//
// #[derive(Clone, Copy, Debug)]
// struct Tile(u16, Option<EntityId>); // (tileid, tileentityid if any)
//
// pub struct Chunk {
// 	chunk_idx: ChunkIdx,
// 	tiles: Vec<Tile>,
// }
// impl Chunk {
// 	fn new(chunk_idx: ChunkIdx, generator: &mut MapGenerator) -> Chunk {
// 		let mut chunk = Chunk {
// 			chunk_idx,
// 			tiles: Vec::with_capacity(CHUNK_SIZE),
// 		};
//
// 		generator.fill_chunk(&mut chunk);
//
// 		chunk
// 	}
//
// 	pub fn chunk_idx(&self) -> ChunkIdx {
// 		self.chunk_idx
// 	}
// }

pub trait MapGenerator: Debug {
	fn get_default_tile(&mut self, coord: Coord) -> Tile;
	fn gen_default_map(&mut self, max_x: u8, max_y: u8) -> Vec<Tile> {
		let mut tiles = Vec::with_capacity((max_x as usize + 1) * (max_y as usize + 1));
		for x in 0..=max_x {
			for y in 0..=max_y {
				tiles.push(self.get_default_tile(Coord::new(x, y)));
			}
		}
		tiles
	}
}

#[derive(Debug)]
pub struct SimpleMapGenerator {
	dirt: u16,
	grass: u16,
}

impl SimpleMapGenerator {
	fn find_tile_id(tile_types: &Vec<TileType>, name: &str) -> u16 {
		let found = tile_types.iter().enumerate().find_map(|(idx, tile_type)| {
			if tile_type.name == name {
				Some(idx as u16)
			} else {
				None
			}
		});
		match found {
			Some(idx) => idx,
			None => panic!(
				"SimpleMapGenerator attempted to find the tile name `{}` but failed to locate it",
				name
			),
		}
	}
	pub fn new(tile_types: &Vec<TileType>) -> SimpleMapGenerator {
		let dirt = SimpleMapGenerator::find_tile_id(tile_types, "Dirt");
		let grass = SimpleMapGenerator::find_tile_id(tile_types, "Grass");
		SimpleMapGenerator { dirt, grass }
	}
}

impl MapGenerator for SimpleMapGenerator {
	fn get_default_tile(&mut self, coord: Coord) -> Tile {
		let id = if (coord.x as u16 + coord.y as u16) % 2 == 0 {
			self.dirt
		} else {
			self.grass
		};
		Tile {
			id,
			entities: vec![],
		}
	}

	// fn gen_default_map(&mut self, width: u8, height: u8) -> Vec<Tile> {
	// 	(0..(width as usize * height as usize))
	// 		.map(|_| self.0.clone())
	// 		.collect()
	// }
}

#[derive(Debug)]
pub struct TileType {
	pub name: String,
	pub uv: Rect,
}

#[derive(Clone, Debug)]
pub struct Tile {
	pub id: u16, // Index into TileType array
	pub entities: Vec<EntityId>,
}

impl Tile {
	pub fn new(id: u16, entities: Vec<EntityId>) -> Tile {
		Tile { id, entities }
	}
}

pub struct TileMap {
	generator: Box<dyn MapGenerator>,
	pub tiles: Vec<Tile>,
	wrap_x: bool,
	wrap_y: bool,
}

pub struct Game {
	pub tile_types: Vec<TileType>,
	pub maps: HashMap<String, TileMap>,
}

impl Game {
	pub fn new() -> Game {
		Game {
			tile_types: Game::load_tile_types(),
			maps: Default::default(),
		}
	}

	fn load_tile_types() -> Vec<TileType> {
		let image_size = 512.0;
		let tile_size = 32.0;
		let tile_scale = tile_size / image_size;
		vec![
			TileType {
				// 0
				name: "Dirt".into(),
				uv: Rect::new(0.0 * tile_scale, 0.0 * tile_scale, tile_scale, tile_scale),
			},
			TileType {
				// 1
				name: "Grass".into(),
				uv: Rect::new(1.0 * tile_scale, 0.0 * tile_scale, tile_scale, tile_scale),
			},
			TileType {
				// 2
				name: "Water".into(),
				uv: Rect::new(2.0 * tile_scale, 0.0 * tile_scale, tile_scale, tile_scale),
			},
		]
	}

	pub fn generate_map(
		&mut self,
		name: String,
		max_x: u8,
		max_y: u8,
		wrap_x: bool,
		wrap_y: bool,
		mut generator: Box<dyn MapGenerator>,
	) {
		if self.maps.contains_key(&name) {
			panic!(
				"Map already generated, double generation attempted `{}`: {:?}",
				name, generator
			);
		}
		let tiles = generator.gen_default_map(max_x, max_y);
		self.maps.insert(
			name,
			TileMap {
				generator,
				tiles,
				wrap_x,
				wrap_y,
			},
		);
	}

	// pub fn iter_map_bounds(
	// 	&mut self,
	// 	name: &str,
	// 	left_top: Coord,
	// 	right_bottom: Coord,
	// ) -> impl Iterator {
	// 	let tile_map = self.maps.get(name);
	// 	if let None = tile_map {
	// 		panic!(
	// 			"Map `{}` attempted to be iterated with bounds but does not exist",
	// 			name
	// 		);
	// 	}
	// 	let tile_map = tile_map.unwrap();
	// 	left_top
	// 		.iterate_coords_to(right_bottom)
	// 		.map(|c| (c, &tile_map.tiles[c.idx().0]))
	// 	// MapRangeIterator {
	// 	// 	tile_map,
	// 	// 	iter: left_top.iterate_coords_to(right_bottom),
	// 	// }
	// }
}

// pub struct MapRangeIterator<'a> {
// 	tile_map: &'a TileMap,
// 	iter: CoordsRangeIterator,
// }
//
// impl<'a> Iterator for MapRangeIterator<'a> {
// 	type Item = ();
//
// 	fn next(&mut self) -> Option<Self::Item> {
// 		unimplemented!()
// 	}
// }
// impl<'a> Iterator for ChunkRangeIterator<'a> {
// 	type Item = &'a Chunk;
//
// 	fn next(&mut self) -> Option<Self::Item> {
// 		loop {
// 			if self.done {
// 				return None;
// 			}
// 			let chunk_coord = ChunkCoord(self.x, self.y);
// 			if self.x == self.x_end && self.y == self.y_end {
// 				self.done = true; // Always at least one iteration since it exists the specific point that is called
// 			}
// 			if self.x == self.x_end {
// 				self.x = self.x_start;
// 				self.y = self.y.wrapping_add(1);
// 			} else {
// 				self.x = self.x.wrapping_add(1);
// 			}
// 			if let Some(ret) = self.chunks.get(&chunk_coord) {
// 				return Some(ret);
// 			}
// 		}
// 	}
// }
