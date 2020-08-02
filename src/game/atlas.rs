#![allow(dead_code)]

use guillotiere::{AllocId, Allocation, AllocatorOptions, AtlasAllocator, Size};
use indexmap::map::IndexMap;
use std::marker::PhantomData;
use std::collections::HashMap;
use thiserror::*;

#[derive(Error, Debug)]
pub enum AtlasError {
	#[error("error from caller code")]
	InterfaceError {
		#[from]
		source: anyhow::Error,
		//backtrace: Backtrace, // Still needs nightly...
	},

	#[error("out of space on atlas")]
	AllocationFailed,

	#[error("source image data is too short for the given width and height passed in for: {0}")]
	SourceImageTooSmallError(String),

	#[error("source image data is too long for the given width and height passed in for: {0}")]
	SourceImageTooLargeError(String),
}

#[derive(Clone, Copy, Debug)]
pub struct AtlasId<Unique: Copy>(usize, PhantomData<Unique>);

#[derive(Clone, Debug)]
pub struct AtlasEntry<Unique: Copy> {
	id: AtlasId<Unique>,
	atlas_id: usize,
	pub name: String,
	pub min: [f32; 2],
	pub max: [f32; 2],
}

impl<Unique: Copy> AtlasEntry<Unique> {
	pub fn left(&self) -> f32 {
		self.min[0]
	}

	pub fn right(&self) -> f32 {
		self.max[0]
	}

	pub fn top(&self) -> f32 {
		self.min[1]
	}

	pub fn bottom(&self) -> f32 {
		self.max[1]
	}

	pub fn get_id(&self) -> AtlasId<Unique> {
		self.id
	}

	pub fn get_atlas_idx(&self) -> usize {
		self.atlas_id
	}
}

pub struct AtlasBuilder<ImageType, Unique: Copy> {
	atlas_id: usize,
	allocator: AtlasAllocator,
	allocations: HashMap<AllocId, Allocation>,
	image_data: Vec<u8>,
	entries: IndexMap<String, AtlasEntry<Unique>>,
	_image: PhantomData<ImageType>,
}

pub struct Atlas<ImageType, Unique: Copy> {
	atlas_id: usize,
	pub image: ImageType,
	entries: IndexMap<String, AtlasEntry<Unique>>,
}

impl<ImageType, Unique: Copy> AtlasBuilder<ImageType, Unique> {
	pub fn new(width: u16, height: u16) -> AtlasBuilder<ImageType, Unique> {
		AtlasBuilder::new_multi(0, width, height)
	}

	fn new_multi(atlas_id: usize, width: u16, height: u16) -> AtlasBuilder<ImageType, Unique> {
		let allocator_options = AllocatorOptions::default();
		AtlasBuilder {
			atlas_id,
			allocator: AtlasAllocator::with_options(
				Size::new(width as i32, height as i32),
				&allocator_options,
			),
			allocations: HashMap::new(),
			image_data: vec![255; width as usize * height as usize * 4],
			entries: IndexMap::new(),
			_image: Default::default(),
		}
	}

	pub fn get_entry(&self, id: AtlasId<Unique>) -> &AtlasEntry<Unique> {
		match &self.entries.get_index(id.0) {
			None => panic!("looked up atlas entry with invalid atlas id, should never happen unless an atlas is recreated, id: {}", id.0),
			Some((_key, entry)) => entry,
		}
	}

	pub fn get_or_create_with<I, FI>(
		&mut self,
		name: &str,
		image_fn: FI,
	) -> Result<AtlasId<Unique>, AtlasError>
	where
		I: IntoIterator<Item = u8>,
		FI: FnOnce() -> Result<(u16, u16, I), anyhow::Error>,
	{
		if let Some(id) = self.entries.get_index_of(name) {
			return Ok(AtlasId(id, Default::default()));
		}

		let (width, height, image_data) = image_fn()?;
		match self
			.allocator
			.allocate((width as i32, height as i32).into())
		{
			None => Err(AtlasError::AllocationFailed),
			Some(alloc) => {
				let atlas_size = self.allocator.size();
				let rgba = &mut self.image_data;
				let stride = atlas_size.width * 4;
				let mut iter = image_data.into_iter();
				for y in alloc.rectangle.min.y..alloc.rectangle.max.y {
					for x in (alloc.rectangle.min.x * 4)..(alloc.rectangle.max.x * 4) {
						let idx = ((y * stride) + x) as usize;
						match iter.next() {
							None => {
								return Err(AtlasError::SourceImageTooSmallError(name.into()));
							}
							Some(v) => {
								rgba[idx] = v;
							}
						}
					}
				}
				if iter.next().is_some() {
					return Err(AtlasError::SourceImageTooLargeError(name.into()));
				}
				let id = AtlasId(self.entries.len(), Default::default());
				self.allocations.insert(alloc.id, alloc);
				let size = self.allocator.size();
				let entry = AtlasEntry {
					id,
					atlas_id: self.atlas_id,
					name: name.into(),
					min: [
						alloc.rectangle.min.x as f32 / size.width as f32,
						alloc.rectangle.min.y as f32 / size.height as f32,
					],
					max: [
						alloc.rectangle.max.x as f32 / size.height as f32,
						alloc.rectangle.max.y as f32 / size.height as f32,
					],
				};
				self.entries.insert(name.into(), entry);
				Ok(id)
			}
		}
	}

	pub fn generate<F>(&self, generate_image: &mut F) -> anyhow::Result<Atlas<ImageType, Unique>>
	where
		F: FnMut(u16, u16, &[u8]) -> anyhow::Result<ImageType>,
	{
		let size = self.allocator.size();
		Ok(Atlas {
			atlas_id: self.atlas_id,
			image: generate_image(size.width as u16, size.height as u16, &self.image_data)?,
			entries: self.entries.clone(),
		})
	}
}

impl<ImageType, Unique: Copy> Atlas<ImageType, Unique> {
	pub fn get_entry(&self, id: AtlasId<Unique>) -> &AtlasEntry<Unique> {
		match &self.entries.get_index(id.0) {
			None => panic!("looked up atlas entry with invalid atlas id, should never happen unless an atlas is recreated, id: {}", id.0),
			Some((_key, entry)) => entry,
		}
	}
}

pub struct MultiAtlasBuilder<ImageType, Unique: Copy> {
	atlases: Vec<AtlasBuilder<ImageType, Unique>>,
	entries: IndexMap<String, AtlasEntry<Unique>>,
}

pub struct MultiAtlas<ImageType, Unique: Copy> {
	atlases: Vec<Atlas<ImageType, Unique>>,
	entries: IndexMap<String, AtlasEntry<Unique>>,
}

impl<ImageType, Unique: Copy> MultiAtlasBuilder<ImageType, Unique> {
	pub fn new(width: u16, height: u16) -> MultiAtlasBuilder<ImageType, Unique> {
		MultiAtlasBuilder {
			atlases: vec![AtlasBuilder::new(width, height)],
			entries: IndexMap::new(),
		}
	}

	pub fn get_entry(&self, id: AtlasId<Unique>) -> &AtlasEntry<Unique> {
		match &self.entries.get_index(id.0) {
			None => panic!("looked up atlas entry with invalid atlas id, should never happen unless an atlas is recreated, id: {}", id.0),
			Some((_key, entry)) => entry,
		}
	}

	pub fn get_or_create_with<I, FI>(
		&mut self,
		name: &str,
		image_fn: FI,
	) -> Result<AtlasId<Unique>, AtlasError>
	where
		I: IntoIterator<Item = u8>,
		FI: FnOnce() -> Result<(u16, u16, I), anyhow::Error>,
	{
		if let Some(id) = self.entries.get_index_of(name) {
			return Ok(AtlasId(id, Default::default()));
		}

		let (width, height, image_data) = image_fn()?;
		for atlas in self.atlases.iter_mut() {
			match atlas.get_or_create_with(name, || Ok((width, height, image_data.into_iter()))) {
				Ok(result) => {
					let id = AtlasId(self.entries.len(), Default::default());
					let mut entry = (*atlas.get_entry(result)).clone();
					entry.id = id;
					self.entries.insert(name.into(), entry);
					return Ok(id);
				}
				Err(AtlasError::AllocationFailed) => {
					todo!();
				}
				err => {
					return err;
				}
			}
		}

		let (w, h) = self.atlases[0].allocator.size().to_tuple();
		let last = self.atlases.len() - 1;
		self.atlases
			.push(AtlasBuilder::new_multi(last, w as u16, h as u16));
		// If it can't fit on a new one then something is just wrong...
		self.atlases[last].get_or_create_with(name, || Ok((width, height, image_data.into_iter())))
	}

	pub fn generate<F>(
		&self,
		generate_image: &mut F,
	) -> anyhow::Result<MultiAtlas<ImageType, Unique>>
	where
		F: FnMut(u16, u16, &[u8]) -> anyhow::Result<ImageType>,
	{
		let mut atlases = Vec::with_capacity(self.atlases.len());
		for atlas in self.atlases.iter().filter(|a| !a.entries.is_empty()) {
			atlases.push(atlas.generate(generate_image)?);
		}
		atlases.shrink_to_fit();
		Ok(MultiAtlas {
			atlases,
			entries: self.entries.clone(),
		})
	}
}

impl<ImageType, Unique: Copy> MultiAtlas<ImageType, Unique> {
	pub fn get_entry(&self, id: AtlasId<Unique>) -> &AtlasEntry<Unique> {
		match &self.entries.get_index(id.0) {
			None => panic!("looked up atlas entry with invalid atlas id, should never happen unless an atlas is recreated, id: {}", id.0),
			Some((_key, entry)) => entry,
		}
	}

	pub fn len_atlases(&self) -> usize {
		self.atlases.len()
	}

	pub fn get_image(&self, id: AtlasId<Unique>) -> &ImageType {
		let entry = self.get_entry(id);
		&self.atlases[entry.atlas_id].image
	}

	pub fn get_image_by_index(&self, id: usize) -> Option<&ImageType> {
		if id >= self.atlases.len() {
			return None;
		}
		Some(&self.atlases[id].image)
	}
}
