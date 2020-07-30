use std::fmt::Debug;
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::core::map::tile::TileType;
use std::convert::Infallible;

/// A simple Input/Output interface to get readers/writers from file names.
///
/// Implement this to allow the engine to be able to talk to storage.
pub trait EngineIO: Debug + Sized {
	type ReadError: std::error::Error + Send + Sync;
	type Read: std::io::Read;
	fn read(&mut self, file_path: PathBuf) -> Result<Self::Read, Self::ReadError>;

	type TileInterface: Debug + Serialize + DeserializeOwned;
	fn blank_tile_interface() -> Self::TileInterface;

	type TileAddedError: std::error::Error + Send + Sync + 'static;
	fn tile_added(
		&mut self,
		index: usize,
		tile_type: &mut TileType<Self>,
	) -> Result<(), Self::TileAddedError>;
}

#[derive(Debug)]
pub struct DirectFilesystemSimpleIO(pub std::path::PathBuf);

impl DirectFilesystemSimpleIO {
	pub fn new(base_path: &str) -> DirectFilesystemSimpleIO {
		DirectFilesystemSimpleIO::with_path(std::path::PathBuf::from(base_path))
	}

	pub fn with_path(base_path: std::path::PathBuf) -> DirectFilesystemSimpleIO {
		DirectFilesystemSimpleIO(base_path)
	}

	pub fn with_cwd() -> DirectFilesystemSimpleIO {
		let path = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
			std::path::PathBuf::from(manifest_dir)
		} else {
			std::path::PathBuf::from(".")
		};
		DirectFilesystemSimpleIO::with_path(path)
	}
}

impl EngineIO for DirectFilesystemSimpleIO {
	type ReadError = std::io::Error;
	type Read = std::fs::File;

	fn read(&mut self, file_path: PathBuf) -> Result<Self::Read, Self::ReadError> {
		let mut path = self.0.clone();
		path.push(file_path);
		std::fs::File::open(path)
	}

	type TileInterface = ();

	#[allow(clippy::unused_unit)]
	fn blank_tile_interface() -> Self::TileInterface {
		()
	}

	type TileAddedError = Infallible;

	fn tile_added(
		&mut self,
		_index: usize,
		_tile_type: &mut TileType<Self>,
	) -> Result<(), Self::TileAddedError> {
		Ok(())
	}
}
