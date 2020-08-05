mod game;

use anyhow::Context as AnyContext;
use log::*;
use over_simple_game_1::core::map::coord::*;
use std::collections::HashSet;
use std::path::Path;

fn main() -> anyhow::Result<()> {
	setup_logging("./log4rs.yaml")?;

	let mut game = game::Game::new().context("Game init failed")?;

	game.setup().context("Game setup failed")?;

	game.run().context("Game run failed")?;

	Ok(())
}

fn setup_logging<P: AsRef<Path>>(config_path: P) -> anyhow::Result<()> {
	let config_path = config_path.as_ref();
	let init = log4rs::init_file(config_path, Default::default());
	match init {
		Ok(()) => (),
		Err(_) => {
			println!("Failed loading the logger configuration file at: {}\nCreating default logger configuration file at: {}", config_path.display(), config_path.display());
			let base_config_path = config_path.parent().with_context(|| {
				format!(
					"unable to acquire the parent path of the logger configuration path: {}",
					config_path.display()
				)
			})?;
			if !base_config_path.exists() {
				std::fs::create_dir_all(base_config_path).with_context(|| {
					format!(
						"unable to create missing paths to the logger configuration file of: {}",
						config_path.display()
					)
				})?;
			}
			std::fs::write(config_path, DEFAULT_LOGGING_YAML).with_context(|| {
				format!(
					"failed writing default logger configuration file at: {}",
					config_path.display()
				)
			})?;
			log4rs::init_file(config_path, Default::default()).with_context(|| {
				format!(
					"failed parsing or loading logger configuration file at: {}",
					config_path.display()
				)
			})?;
		}
	}

	info!(
		"Successfully initialized the logging system from configuration: {}",
		config_path.display()
	);

	Ok(())
}

const DEFAULT_LOGGING_YAML: &str = r#"
refresh_rate: 30 seconds
appenders:
  stdout:
    kind: console
    encoder:
      pattern: "{d} [{t}:{I}:{T}] {h({l})} {M}: {m}{n}"

root:
  level: trace
  appenders:
    - stdout
"#;
