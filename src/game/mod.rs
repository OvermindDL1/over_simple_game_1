mod components;
mod game_map;

use ggez::event::EventHandler;
use ggez::{graphics, Context, GameResult};

use game_map::*;

pub struct GameState {
	ecs: shipyard::World,
	map: GameMap,
}

impl GameState {
	pub fn new() -> GameState {
		let mut game_state = GameState {
			ecs: shipyard::World::new(),
			map: GameMap::new(),
		};

		game_state
	}

	fn draw_map(&mut self, ctx: &mut Context) -> GameResult<()> {
		for chunk in self
			.map
			.iter_generated_chunks_in_range(Coord(0, 0), Coord(10, 100))
		{
			dbg!(chunk.chunk_coord());
		}
		Ok(())
	}
}

impl EventHandler for GameState {
	fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
		self.map.ensure_generated(Coord(0, 0));
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
		graphics::clear(ctx, graphics::BLACK);
		self.draw_map(ctx)?;
		graphics::present(ctx)
	}
}
