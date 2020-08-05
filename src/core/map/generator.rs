use super::tile::{Tile, TileIdx};
use crate::core::engine::io::EngineIO;
use crate::core::engine::Engine;
use crate::core::map::tile_map::TileMap;
use anyhow::Context as AnyContext;

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
	pub fn new<NameIter: IntoIterator, IO: EngineIO>(
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
				.tile_types
				.get_index_of(name)
				.with_context(|| format!("missing tile type: {}", name))?;
			tiles.push(idx)
		}
		Ok(SimpleAlternationMapGenerator(tiles))
	}
}
