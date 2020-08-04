use shipyard::*;
use thiserror::*;

#[derive(Debug, Error)]
pub enum ComponentAutoLoadError {
	#[error("error while acquiring storage: {storage_name}")]
	GetStorageError {
		source: shipyard::error::GetStorage,
		//backtrace: Backtrace, // Still needs nightly...
		storage_name: String,
	},
	#[error("error while adding a component to an entity {entity:?}: {storage_name}")]
	AddComponentError {
		source: shipyard::error::AddComponent,
		//backtrace: Backtrace, // Still needs nightly...
		entity: EntityId,
		storage_name: String,
	},
}

#[typetag::serde()] // tag = "component"
pub trait ComponentAutoLoadable {
	fn add_to_entity(
		&self,
		entity: EntityId,
		all_storages: &mut AllStoragesViewMut,
	) -> Result<(), ComponentAutoLoadError>;
}

#[macro_export]
macro_rules! component_auto_loadable {
	($typ:ty) => {
		#[typetag::serde]
		impl over_simple_game_1::core::component::ComponentAutoLoadable for $typ {
			fn add_to_entity(
				&self,
				entity: shipyard::EntityId,
				all_storages: &mut shipyard::AllStoragesViewMut,
			) -> Result<(), over_simple_game_1::core::component::ComponentAutoLoadError> {
				use over_simple_game_1::core::component::ComponentAutoLoadError;
				use shipyard::*;
				let entities = all_storages
					.try_borrow::<EntitiesView>()
					.map_err(|source| ComponentAutoLoadError::GetStorageError {
						source,
						storage_name: stringify!($typ).to_owned(),
					})?;
				let mut storage = all_storages
					.try_borrow::<ViewMut<$typ>>()
					.map_err(|source| ComponentAutoLoadError::GetStorageError {
						source,
						storage_name: stringify!($typ).to_owned(),
					})?;
				(&entities)
					.try_add_component(&mut storage, self.clone(), entity)
					.map_err(|source| ComponentAutoLoadError::AddComponentError {
						source,
						entity,
						storage_name: stringify!($typ).to_owned(),
					})
			}
		}
	};
}
