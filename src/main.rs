mod game;

use anyhow::Context as AnyContext;

fn main() -> anyhow::Result<()> {
	let mut game = game::Game::new().context("Game init failed")?;

	game.setup().context("Game setup failed")?;

	game.run().context("Game run failed")?;

	Ok(())
}
