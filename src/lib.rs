use ggez::event::EventHandler;
use ggez::{graphics, Context, GameResult};

pub struct GameState {}

impl GameState {
	pub fn new() -> GameState {
		GameState {}
	}
}

impl EventHandler for GameState {
	fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
		graphics::clear(ctx, graphics::BLACK);
		graphics::present(ctx)
	}
}
