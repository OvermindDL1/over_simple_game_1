use std::path::{Path, PathBuf};

use anyhow::Context as AnyContext;
use shipyard::*;

use crate::core::component::ComponentAutoLoadable;
use crate::core::engine::io::EngineIO;

pub mod maps;

pub struct CivGame {
	base_resource_path: PathBuf,
}

impl CivGame {
	pub fn new<P: AsRef<Path>>(base_resource_path: P) -> CivGame {
		CivGame {
			base_resource_path: base_resource_path.as_ref().into(),
		}
	}

	// pub fn setup<IO: EngineIO>(&mut self) {}

	// This entity is not yet attached to the world
	pub fn create_entity_from_template<IO: 'static + EngineIO>(
		&mut self,
		io: &mut IO,
		template: &str,
		all_storages: &mut AllStoragesViewMut,
	) -> anyhow::Result<EntityId> {
		// TODO: Build a cache for this
		let mut path = self.base_resource_path.clone();
		path.push("entities");
		path.push(format!("{}.ron", template));
		let reader = io.read(path.as_path())?;
		let components: Vec<Box<dyn ComponentAutoLoadable>> = ron::de::from_reader(reader)
			.with_context(|| format!("Failed loading component template for: {}", template))?;
		let entity = all_storages
			.try_borrow::<EntitiesViewMut>()?
			.add_entity((), ());
		for c in components {
			c.add_to_entity(entity, all_storages)?;
		}
		Ok(entity)
	}
}
