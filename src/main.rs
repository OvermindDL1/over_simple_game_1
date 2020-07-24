mod game;

use anyhow::Context as AnyContext;
use ggez::conf::{FullscreenType, NumSamples, WindowMode, WindowSetup};
use ggez::{event, ContextBuilder};

fn main() -> anyhow::Result<()> {
	let window_setup = WindowSetup {
		title: "OverSimpleGame1".to_string(),
		samples: NumSamples::Zero,
		vsync: false,
		icon: "".to_string(), // TODO: Create an icon
		srgb: false,
	};

	let window_mode = WindowMode {
		width: 800.0,
		height: 600.0,
		maximized: false,
		fullscreen_type: FullscreenType::Windowed,
		borderless: false,
		min_width: 640.0,
		min_height: 480.0,
		max_width: 0.0,
		max_height: 0.0,
		resizable: true,
	};

	let (mut ctx, mut event_loop) = ContextBuilder::new("over-simple-game-1", "OvermindDL1")
		.window_setup(window_setup)
		.window_mode(window_mode)
		.add_resource_path(
			if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
				let mut path = std::path::PathBuf::from(manifest_dir);
				path.push("resources");
				path
			} else {
				std::path::PathBuf::from("./resources")
			},
		)
		.build()
		.context("Failed to create GGEZ Context")?;

	let mut game_state = game::GameState::new();

	event::run(&mut ctx, &mut event_loop, &mut game_state).context("Game Error Occurred")?;

	Ok(())
}
