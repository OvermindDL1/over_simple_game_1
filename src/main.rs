use anyhow::Context as AnyContext;
use ggez::{event, ContextBuilder};
use over_simple_game_1::GameState;

fn main() -> anyhow::Result<()> {
	let (mut ctx, mut event_loop) = ContextBuilder::new("over-simple-game-1", "OvermindDL1")
		.build()
		.context("Failed to create GGEZ Context")?;

	let mut game_state = GameState::new();

	event::run(&mut ctx, &mut event_loop, &mut game_state).context("Game Error Occurred")?;

	Ok(())
}
