use shipyard::EntityId;
use std::collections::HashMap;

#[derive(Clone, Copy, Default, Debug, Hash, PartialOrd, PartialEq, Ord, Eq)]
pub struct Coord(pub i16, pub i16);
#[derive(Clone, Copy, Default, Debug, Hash, PartialOrd, PartialEq, Ord, Eq)]
pub struct ChunkCoord(pub i8, pub i8);

impl Coord {
	fn new(x: i16, y: i16) -> Coord {
		Coord(x, y)
	}

	fn get_chunkcoord(&self) -> ChunkCoord {
		ChunkCoord((self.0 >> 8) as i8, (self.1 >> 8) as i8)
	}

	fn as_chunk_idx(&self) -> usize {
		let x = self.0 & 255;
		let y = self.0 & 255;
		((y << 8) + x) as usize
	}
}

impl ChunkCoord {
	fn new(x: i8, y: i8) -> ChunkCoord {
		ChunkCoord(x, y)
	}

	fn get_coord(&self, x: i8, y: i8) -> Coord {
		Coord(
			((self.0 as i16) << 8) + x as i16,
			((self.1 as i16) << 8) + y as i16,
		)
	}
}

#[derive(Clone, Copy, Debug)]
struct Tile(u16, Option<EntityId>); // (tileid, tileentityid if any)

const CHUNK_SIZE: usize = 256 * 256;
pub struct Chunk {
	chunk_coord: ChunkCoord,
	tiles: Vec<Tile>,
}
impl Chunk {
	fn new(chunk_coord: ChunkCoord, generator: &mut MapGenerator) -> Chunk {
		let mut chunk = Chunk {
			chunk_coord,
			tiles: Vec::with_capacity(CHUNK_SIZE),
		};

		generator.fill_chunk(&mut chunk);

		chunk
	}

	pub fn chunk_coord(&self) -> ChunkCoord {
		self.chunk_coord
	}
}

struct MapGenerator();

impl MapGenerator {
	fn new() -> MapGenerator {
		MapGenerator()
	}

	fn get_tile(&mut self, _x: u16, _y: u16) -> Tile {
		Tile(0, None)
	}

	fn fill_chunk(&mut self, chunk: &mut Chunk) {
		chunk.tiles.clear();
		for x in 0..256 {
			for y in 0..256 {
				// chunk.tiles[y as usize * 256 + x as usize] = self.get_tile(x, y);
				chunk.tiles.push(self.get_tile(x, y))
			}
		}
	}
}

pub struct TileType {
	name: String,
}

pub struct GameMap {
	generator: MapGenerator,
	tile_types: Vec<TileType>,
	chunks: HashMap<ChunkCoord, Chunk>,
	entities: Vec<EntityId>,
}

impl GameMap {
	pub fn new() -> GameMap {
		GameMap {
			generator: MapGenerator::new(),
			tile_types: GameMap::generate_default_tile_types(),
			chunks: Default::default(),
			entities: Default::default(),
		}
	}

	fn generate_default_tile_types() -> Vec<TileType> {
		vec![
			TileType {
				// 0
				name: "Dirt".into(),
			},
			TileType {
				// 1
				name: "Grass".into(),
			},
			TileType {
				// 2
				name: "Water".into(),
			},
		]
	}

	pub fn ensure_generated(&mut self, coord: Coord) {
		let chunk_coord = coord.get_chunkcoord();
		if !self.chunks.contains_key(&chunk_coord) {
			let chunk = Chunk::new(chunk_coord, &mut self.generator);
			self.chunks.insert(chunk_coord, chunk);
		}
	}

	pub fn iter_generated_chunks_in_range(
		&mut self,
		left_top: Coord,
		right_bottom: Coord,
	) -> ChunkRangeIterator {
		let left_top = left_top.get_chunkcoord();
		let right_bottom = right_bottom.get_chunkcoord();
		ChunkRangeIterator {
			chunks: &self.chunks,
			x: left_top.0,
			x_start: left_top.0,
			x_end: right_bottom.0,
			y: left_top.1,
			y_end: right_bottom.1,
			done: false,
		}
	}
}

pub struct ChunkRangeIterator<'a> {
	chunks: &'a HashMap<ChunkCoord, Chunk>,
	x: i8,
	x_start: i8,
	x_end: i8,
	y: i8,
	y_end: i8,
	done: bool,
}
impl<'a> Iterator for ChunkRangeIterator<'a> {
	type Item = &'a Chunk;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			if self.done {
				return None;
			}
			let chunk_coord = ChunkCoord(self.x, self.y);
			if self.x == self.x_end && self.y == self.y_end {
				self.done = true; // Always at least one iteration since it exists the specific point that is called
			}
			if self.x == self.x_end {
				self.x = self.x_start;
				self.y = self.y.wrapping_add(1);
			} else {
				self.x = self.x.wrapping_add(1);
			}
			if let Some(ret) = self.chunks.get(&chunk_coord) {
				return Some(ret);
			}
		}
	}
}
