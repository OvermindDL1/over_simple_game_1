pub mod core;
pub mod games;

pub mod prelude {
	pub use crate::core::engine::io::EngineIO;
	pub use crate::core::engine::Engine;
	pub use crate::core::map::coord::Coord;
	pub use crate::core::map::tile::{Tile, TileIdx, TileType};
}
