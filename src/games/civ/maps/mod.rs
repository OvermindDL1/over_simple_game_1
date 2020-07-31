use crate::core::engine::io::EngineIO;
use crate::core::map::generator::MapGenerator;
use crate::core::map::tile::{TileIdx, TileTypes};
use crate::core::map::tile_map::TileMap;

pub struct NoiseMap {
	dirt: TileIdx,
	grass: TileIdx,
	sand: TileIdx,
}

impl NoiseMap {
	pub fn new<IO: EngineIO>(tile_types: &TileTypes<IO>) -> NoiseMap {
		todo!()
		// NoiseMap {
		// 	dirt: tile_types.
		// }
	}
}

impl MapGenerator for NoiseMap {
	fn generate(&mut self, tile_map: &mut TileMap) -> anyhow::Result<()> {
		tile_map.tiles.clear();
		for y in 0usize..=(tile_map.width as usize) {
			for x in 0usize..=(tile_map.height as usize) {
				let idx = (y * tile_map.height as usize) + x;
				// let tile_idx = self.0[idx % self.0.len()];
				// tile_map.tiles.push(Tile::new(tile_idx));
			}
		}

		Ok(())
	}
}
