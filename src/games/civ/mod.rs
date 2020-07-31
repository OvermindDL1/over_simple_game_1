pub mod maps;

use crate::core::engine::io::EngineIO;
use std::path::{Path, PathBuf};

pub struct CivGame {
	base_resource_path: PathBuf,
}

impl CivGame {
	pub fn new<P: AsRef<Path>>(base_resource_path: P) -> CivGame {
		CivGame {
			base_resource_path: base_resource_path.as_ref().into(),
		}
	}

	pub fn setup<IO: EngineIO>(&mut self) {}
}
