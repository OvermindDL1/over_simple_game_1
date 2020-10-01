use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::Context as AnyContext;
use ggez::conf::{FullscreenType, NumSamples, WindowMode, WindowSetup};
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::graphics::{Color, DrawMode, DrawParam, Drawable, FilterMode, Rect, Vertex};
use ggez::input::{keyboard, mouse};
use ggez::nalgebra as na;
use ggez::{graphics, Context, ContextBuilder, GameError};
use log::*;
use serde::{Deserialize, Serialize};
use shipyard::*;
use winit::{
	dpi, ElementState, Event, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta,
	VirtualKeyCode, WindowEvent,
};

use over_simple_game_1::core::engine::MapCoord;
use over_simple_game_1::core::map::generator::SimpleAlternationMapGenerator;
use over_simple_game_1::games::civ::CivGame;
use over_simple_game_1::prelude::*;

use crate::game::atlas::{AtlasId, MultiAtlas, MultiAtlasBuilder};
use crate::game::components::DrawSprite;

mod atlas;

mod components;

mod cli;

#[derive(Clone, Copy, Debug)]
enum MapAtlas {}

#[derive(Clone, Copy, Debug)]
enum EntityAtlas {}

fn serde_hex_bound() -> Rect {
	Rect {
		x: -0.5,
		y: -0.5833333,
		w: 1.0,
		h: 1.1666666,
	}
}

fn serde_hex_color() -> Color {
	Color::new(1.0, 1.0, 1.0, 1.0)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TileDrawableInfo {
	#[serde(default = "serde_hex_bound")]
	bounds: Rect,
	#[serde(default = "serde_hex_color")]
	color: Color,
}

struct TilesDrawable {
	atlas_id: AtlasId<MapAtlas>,
	info: TileDrawableInfo,
}

struct MouseButtonPressedData {
	screen: na::Point2<f32>,
	time: Instant,
}

impl MouseButtonPressedData {
	fn new(x: f32, y: f32) -> MouseButtonPressedData {
		MouseButtonPressedData {
			screen: [x, y].into(),
			time: Instant::now(),
		}
	}
}

struct GameState {
	ctx: Context,
	visible_map: String,
	screen_tiles: f32,
	zoom: f32,
	view_center: na::Point2<f32>,
	screen_size: dpi::LogicalSize,
	aspect_ratio: f32,
	tiles_atlas: MultiAtlas<graphics::Image, MapAtlas>,
	tiles_meshes: Vec<Option<graphics::Mesh>>,
	tiles_drawable: Vec<TilesDrawable>,
	entity_atlas: MultiAtlas<graphics::Image, EntityAtlas>,
	entity_spritebatches: Vec<SpriteBatch>,
	selected: Option<EntityId>,
	selected_mesh: Option<graphics::Mesh>,
	click_leeway: f32,
	mouse_buttons_clicked: HashMap<MouseButton, MouseButtonPressedData>,
	mouse_last_position: na::Point2<f32>,
}

pub struct Game {
	state: GameState,
	ecs: shipyard::World,
	engine: Engine<GameState>,
	civ: CivGame,
	events_loop: ggez::event::EventsLoop,
	cli_commands: std::sync::mpsc::Receiver<cli::CliCommand>,
	// gamepad_enabled: bool,
}

impl fmt::Debug for GameState {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("GameState")
			.field("visible_map", &self.visible_map)
			.field("screen_tiles", &self.screen_tiles)
			.field("zoom", &self.zoom)
			.field("tiles_meshes", &self.tiles_meshes)
			.finish()
	}
}

impl EngineIO for GameState {
	type ReadError = GameError;
	type Read = ggez::filesystem::File;

	fn read(&mut self, file_path: PathBuf) -> Result<Self::Read, Self::ReadError> {
		let mut path = PathBuf::from("/");
		path.push(file_path);
		ggez::filesystem::open(&mut self.ctx, path)
	}

	type TileInterface = ();

	fn blank_tile_interface() -> Self::TileInterface {}

	type TileAddedError = Infallible;

	fn tile_added(
		&mut self,
		_index: TileIdx,
		_tile_type: &mut TileType<Self>,
	) -> Result<(), Self::TileAddedError> {
		Ok(())
	}
}

impl Game {
	pub fn new() -> anyhow::Result<Game> {
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

		let (ctx, events_loop) = ContextBuilder::new("over-simple-game-1", "OvermindDL1")
			.window_setup(window_setup)
			.window_mode(window_mode)
			.add_resource_path(
				// if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
				// 	let mut path = std::path::PathBuf::from(manifest_dir);
				// 	path.push("resources");
				// 	path
				// } else {
				std::path::PathBuf::from("./resources"), // },
			)
			.build()
			.context("Failed to create GGEZ Context")?;

		// // This is... not right, why does ggez not let us test this ourselves?
		// let conf = ggez::conf::Conf::new();
		// let gamepad_enabled = conf.modules.gamepad;

		let state = GameState::new(ctx);
		let ecs = shipyard::World::new();
		let engine = Engine::new();
		let civ = CivGame::new("/civ");
		let (_, cli_commands) = cli::init_cli_thread();

		Ok(Game {
			state,
			ecs,
			engine,
			civ,
			events_loop,
			cli_commands,
			// gamepad_enabled,
		})
	}

	pub fn setup(&mut self) -> anyhow::Result<()> {
		self.engine.setup(&mut self.state)?;
		self.state.setup(&mut self.engine)?;
		let mut generator =
			SimpleAlternationMapGenerator::new(&mut self.engine, &["dirt", "grass", "sand"])?;
		// let mut generator = civ::maps::NoiseMap::new(&self.engine.tile_types);
		let name = self.state.visible_map.clone();
		self.engine
			.generate_map(&mut self.state, name, 6, 6, true, &mut generator)?;

		let coord = MapCoord {
			map: self
				.engine
				.maps
				.get_index_of(&self.state.visible_map)
				.context("visible map is missing")?,
			coord: Coord::new_axial(1, 1),
		};
		let state = &mut self.state;
		let engine = &mut self.engine;
		let ecs = &mut self.ecs;
		let civ = &mut self.civ;
		ecs.run(
			|mut all_storages: AllStoragesViewMut| -> anyhow::Result<()> {
				let entity =
					civ.create_entity_from_template(state, "test_unit", &mut all_storages)?;
				engine.move_entity_to_coord(
					entity,
					coord,
					all_storages.try_borrow()?,
					all_storages.try_borrow()?,
				)?;
				Ok(())
			},
		)?;

		Ok(())
	}

	pub fn run(&mut self) -> anyhow::Result<()> {
		while self.state.ctx.continuing {
			self.run_once()?;
		}

		Ok(())
	}

	pub fn run_once(&mut self) -> anyhow::Result<()> {
		while let Ok(cmd) = self.cli_commands.try_recv() {
			self.state.apply_cli_command(cmd);
		}
		let state = &mut self.state;
		let ecs = &mut self.ecs;
		let events_loop = &mut self.events_loop;
		let engine = &mut self.engine;
		state.ctx.timer_context.tick();
		events_loop.poll_events(|event| {
			state.ctx.process_event(&event);
			state.dispatch_event(ecs, engine, event).unwrap();
		});
		// Handle gamepad events if necessary.
		// Yeah okay, ggez has this entirely borked behind private...
		// if self.gamepad_enabled {
		// 	while let Some(gilrs::Event { id, event, .. }) =
		// 		self.state.ctx.gamepad_context.next_event()
		// 	{
		// 		match event {
		// 			gilrs::EventType::ButtonPressed(button, _) => {
		// 				self.state.gamepad_button_down_event(button, id)?;
		// 			}
		// 			gilrs::EventType::ButtonReleased(button, _) => {
		// 				self.state.gamepad_button_up_event(button, id)?;
		// 			}
		// 			gilrs::EventType::AxisChanged(axis, value, _) => {
		// 				self.state.gamepad_axis_event(axis, value, id)?;
		// 			}
		// 			_ => {}
		// 		}
		// 	}
		// }
		self.state.update(&mut self.ecs, &mut self.engine)?;
		self.state.draw(&mut self.ecs, &mut self.engine)?;

		Ok(())
	}
}

impl GameState {
	fn new(mut ctx: Context) -> GameState {
		let tiles_atlas = MultiAtlasBuilder::new(1, 1)
			.generate(&mut |_width, _height, _data| {
				Ok(graphics::Image::solid(&mut ctx, 1, graphics::WHITE)?)
			})
			.unwrap();
		let entity_atlas = MultiAtlasBuilder::new(1, 1)
			.generate(&mut |_width, _height, _data| {
				Ok(graphics::Image::solid(&mut ctx, 1, graphics::WHITE)?)
			})
			.unwrap();
		GameState {
			ctx,
			visible_map: "world0".to_owned(),
			screen_tiles: 2.0,
			zoom: 2.0,
			view_center: na::Point2::from([0.0, 0.0]),
			screen_size: dpi::LogicalSize {
				width: 1.0,
				height: 1.0,
			},
			aspect_ratio: 1.0,
			tiles_atlas,
			tiles_meshes: vec![],
			tiles_drawable: vec![],
			entity_spritebatches: vec![],
			entity_atlas,
			selected: None,
			selected_mesh: None,
			click_leeway: 4.0,
			mouse_buttons_clicked: HashMap::new(),
			mouse_last_position: [0.0, 0.0].into(),
		}
	}

	pub fn setup(&mut self, engine: &mut Engine<GameState>) -> anyhow::Result<()> {
		self.tiles_drawable.clear();
		self.tiles_drawable
			.reserve(engine.tile_types.tile_types.len());
		let mut tile_atlas_builder = MultiAtlasBuilder::new(2048, 2048);
		for name in engine.tile_types.tile_types.values().map(|t| &t.name) {
			let ctx = &mut self.ctx;
			let id = tile_atlas_builder.get_or_create_with(name, || {
				use std::io::Read;
				let mut path = PathBuf::from("/tiles");
				path.push(format!("{}.png", name));

				let mut buf = Vec::new();
				let mut reader = ggez::filesystem::open(ctx, path)?;
				let _ = reader.read_to_end(&mut buf)?;
				let image = image::load_from_memory(&buf)?.to_rgba();
				let width = image.width() as u16;
				let height = image.height() as u16;
				let rgba = image.into_raw();

				Ok((width, height, rgba))
			})?;

			let mut path = PathBuf::from("/tiles");
			path.push(format!("{}.png.ron", name));
			let info = match ggez::filesystem::open(ctx, path) {
				Err(_e) => {
					debug!(
						"Unable to load ron data for tile of `{}.png.ron`, using defaults",
						name
					);
					TileDrawableInfo {
						bounds: Rect::new(-0.5, -0.5833333, 1.0, 1.1666666),
						color: Color::new(1.0, 1.0, 1.0, 1.0),
					}
				}
				Ok(file) => ron::de::from_reader::<_, TileDrawableInfo>(file)?,
			};

			self.tiles_drawable
				.push(TilesDrawable { atlas_id: id, info })
		}
		self.tiles_atlas = tile_atlas_builder.generate(&mut |width, height, rgba| {
			let mut image = graphics::Image::from_rgba8(&mut self.ctx, width, height, rgba)
				.context("failed converting tiles atlas texture")?;
			image.set_filter(FilterMode::Nearest);
			Ok(image)
		})?;
		// let image = self.tiles_atlas.get_image_by_index(0).unwrap();
		// image.encode(&mut self.ctx, graphics::ImageFormat::Png, "/tilemap0.png")?;

		// TODO: Make this more fancy like the tiles atlas to load user defined entity files and all
		let mut entity_atlas_builder = MultiAtlasBuilder::new(2048, 2048);
		let path = PathBuf::from("/sprites/_load.ron");
		let reader = ggez::filesystem::open(&mut self.ctx, path)?;
		let entity_sprites: Vec<String> = ron::de::from_reader(reader)?;
		for sprite_name in entity_sprites {
			let ctx = &mut self.ctx;
			let id = entity_atlas_builder.get_or_create_with(&sprite_name, || {
				use std::io::Read;
				let mut path = PathBuf::from("/sprites");
				path.push(format!("{}.png", sprite_name));

				let mut buf = Vec::new();
				let mut reader = ggez::filesystem::open(ctx, path)?;
				let _ = reader.read_to_end(&mut buf)?;
				let image = image::load_from_memory(&buf)?.to_rgba();
				let width = image.width() as u16;
				let height = image.height() as u16;
				let rgba = image.into_raw();

				Ok((width, height, rgba))
			})?;
		}
		self.entity_atlas = entity_atlas_builder.generate(&mut |width, height, rgba| {
			let mut image = graphics::Image::from_rgba8(&mut self.ctx, width, height, rgba)
				.context("failed converting entities atlas texture")?;
			image.set_filter(FilterMode::Nearest);
			Ok(image)
		})?;
		Ok(())
	}

	fn dispatch_event(
		&mut self,
		ecs: &mut shipyard::World,
		engine: &mut Engine<GameState>,
		event: Event,
	) -> anyhow::Result<()> {
		match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::Resized(logical_size) => {
					self.resize_event(ecs, engine, logical_size)?;
				}
				WindowEvent::CloseRequested => {
					if self.quit_event(ecs, engine)? {
						ggez::event::quit(&mut self.ctx);
					}
				}
				WindowEvent::Focused(gained) => {
					self.focus_event(ecs, engine, gained)?;
				}
				WindowEvent::ReceivedCharacter(ch) => {
					self.text_input_event(ecs, engine, ch)?;
				}
				WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							state: ElementState::Pressed,
							virtual_keycode: Some(keycode),
							modifiers,
							..
						},
					..
				} => {
					let repeat = keyboard::is_key_repeated(&self.ctx);
					self.key_down_event(ecs, engine, keycode, modifiers, repeat)?;
				}
				WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							state: ElementState::Released,
							virtual_keycode: Some(keycode),
							modifiers,
							..
						},
					..
				} => {
					self.key_up_event(ecs, engine, keycode, modifiers)?;
				}
				WindowEvent::MouseWheel { delta, .. } => {
					let (x, y) = match delta {
						MouseScrollDelta::LineDelta(x, y) => (x, y),
						MouseScrollDelta::PixelDelta(dpi::LogicalPosition { x, y }) => {
							(x as f32, y as f32)
						}
					};
					self.mouse_wheel_event(ecs, engine, x, y)?;
				}
				WindowEvent::MouseInput {
					state: element_state,
					button,
					..
				} => {
					let position = mouse::position(&self.ctx);
					match element_state {
						ElementState::Pressed => {
							self.mouse_button_down_event(
								ecs, engine, button, position.x, position.y,
							)?;
						}
						ElementState::Released => {
							self.mouse_button_up_event(
								ecs, engine, button, position.x, position.y,
							)?;
						}
					}
				}
				WindowEvent::CursorMoved { .. } => {
					let position = mouse::position(&self.ctx);
					let delta = mouse::delta(&self.ctx);
					self.mouse_motion_event(ecs, engine, position.x, position.y, delta.x, delta.y)?;
				}
				_x => {
					// trace!("ignoring window event {:?}", x);
				}
			},
			Event::DeviceEvent { event: _event, .. } => {
				// match event {
				// 	_ => (),
				// }
			}
			Event::Awakened => (),
			Event::Suspended(_) => (),
		}

		Ok(())
	}

	// Callbacks
	fn resize_event(
		&mut self,
		_ecs: &mut shipyard::World,
		_engine: &mut Engine<GameState>,
		logical_size: dpi::LogicalSize,
	) -> anyhow::Result<()> {
		self.screen_size = logical_size;
		self.aspect_ratio = (logical_size.width / logical_size.height) as f32;
		self.tiles_meshes.clear();
		Ok(())
	}

	fn quit_event(
		&mut self,
		_ecs: &mut shipyard::World,
		_engine: &mut Engine<GameState>,
	) -> anyhow::Result<bool> {
		Ok(true)
	}

	fn focus_event(
		&mut self,
		_ecs: &mut shipyard::World,
		_engine: &mut Engine<GameState>,
		_gained: bool,
	) -> anyhow::Result<()> {
		Ok(())
	}

	fn text_input_event(
		&mut self,
		_ecs: &mut shipyard::World,
		_engine: &mut Engine<GameState>,
		_ch: char,
	) -> anyhow::Result<()> {
		Ok(())
	}

	fn key_down_event(
		&mut self,
		_ecs: &mut shipyard::World,
		_engine: &mut Engine<GameState>,
		_keycode: VirtualKeyCode,
		_modifiers: ModifiersState,
		_repeat: bool,
	) -> anyhow::Result<()> {
		Ok(())
	}

	fn key_up_event(
		&mut self,
		_ecs: &mut shipyard::World,
		_engine: &mut Engine<GameState>,
		keycode: VirtualKeyCode,
		modifiers: ModifiersState,
	) -> anyhow::Result<()> {
		use VirtualKeyCode::*;
		match (keycode, modifiers) {
			(Escape, _) => ggez::event::quit(&mut self.ctx),
			(W, _) => (),
			(A, _) => (),
			(S, _) => (),
			(D, _) => (),
			_ => (),
		}
		Ok(())
	}

	fn mouse_wheel_event(
		&mut self,
		_ecs: &mut shipyard::World,
		_engine: &mut Engine<GameState>,
		_x: f32,
		y: f32,
	) -> anyhow::Result<()> {
		self.screen_tiles += (-y * 0.5) * (1.0 + self.screen_tiles * 0.5);
		if self.screen_tiles < 1.0 {
			self.screen_tiles = 1.0;
		} else if self.screen_tiles > 16.0 {
			self.screen_tiles = 16.0;
		}
		self.tiles_meshes.clear();
		Ok(())
	}

	fn mouse_button_down_event(
		&mut self,
		_ecs: &mut shipyard::World,
		_engine: &mut Engine<GameState>,
		button: MouseButton,
		x: f32,
		y: f32,
	) -> anyhow::Result<()> {
		let screen_x = x / self.screen_size.width as f32;
		let screen_y = y / self.screen_size.height as f32;
		self.mouse_buttons_clicked
			.insert(button, MouseButtonPressedData::new(screen_x, screen_y));
		self.mouse_last_position = [screen_x, screen_y].into();
		Ok(())
	}

	fn set_selected_coord(
		&mut self,
		ecs: &mut shipyard::World,
		engine: &mut Engine<GameState>,
		coord: MapCoord,
	) -> anyhow::Result<()> {
		self.remove_selected(ecs, engine)?;
		let tile_map = engine
			.maps
			.get_index_mut(coord.map)
			.context("unable to lookup the visible map")?
			.1;
		match tile_map.get_tile_mut(coord.coord) {
			None => (),
			Some(tile) => {
				let entity = ecs
					.try_entity_builder()?
					.try_with(components::IsSelected())?
					.try_with(coord)?
					.try_build()?;
				tile.entities.insert(entity);
				self.selected = Some(entity);
			}
		}

		Ok(())
	}

	fn _set_selected_entity(
		&mut self,
		ecs: &mut shipyard::World,
		_engine: &mut Engine<GameState>,
		entity: EntityId,
	) {
		ecs.run(
			|entities: EntitiesView, mut selected: ViewMut<components::IsSelected>| {
				entities.add_component(&mut selected, components::IsSelected(), entity);
			},
		)
	}

	fn remove_selected(
		&mut self,
		ecs: &mut shipyard::World,
		engine: &mut Engine<GameState>,
	) -> anyhow::Result<()> {
		match self.selected {
			None => (),
			Some(entity) => {
				self.selected = None;
				let maps = &mut engine.maps;
				ecs.run(
					|mut all_storages: AllStoragesViewMut| -> anyhow::Result<()> {
						{
							let coords = all_storages.try_borrow::<View<MapCoord>>()?;
							let coord = coords[entity];
							let tile_map = maps
								.get_index_mut(coord.map)
								.context("unable to lookup the visible map")?
								.1;
							let tile = tile_map
								.get_tile_mut(coord.coord)
								.context("tile cannot be found that should exist")?;
							tile.entities.remove(&entity);
						}
						all_storages.delete(entity);
						Ok(())
					},
				)?;
			}
		}

		Ok(())
	}

	fn screen_ratio_to_map(&self, screen_x: f32, screen_y: f32) -> (f32, f32) {
		let visible_width = self.screen_tiles * self.aspect_ratio;
		let visible_height = self.screen_tiles;
		let offset_x = (visible_width * 0.5) - self.view_center.x;
		let offset_y = (visible_height * 0.5) - self.view_center.y;
		let map_x = (screen_x * visible_width) - offset_x;
		let map_y = (screen_y * visible_height) - offset_y;
		(map_x, map_y)
	}

	fn mouse_button_up_event(
		&mut self,
		ecs: &mut shipyard::World,
		engine: &mut Engine<GameState>,
		button: MouseButton,
		x: f32,
		y: f32,
	) -> anyhow::Result<()> {
		let screen_x = x / self.screen_size.width as f32;
		let screen_y = y / self.screen_size.height as f32;
		if let Some(button_pressed_data) = self.mouse_buttons_clicked.get(&button) {
			// Test if a proper click
			if self.is_proper_click(button_pressed_data, screen_x, screen_y) {
				let (map_x, map_y) = self.screen_ratio_to_map(screen_x, screen_y);
				let coord = Coord::from_linear(map_x, map_y);
				let map_coord = MapCoord {
					map: engine
						.maps
						.get_index_of(&self.visible_map)
						.context("visible map doesn't exist")?,
					coord,
				};
				self.set_selected_coord(ecs, engine, map_coord)?;
			}
		}
		self.mouse_buttons_clicked.remove(&button);
		self.mouse_last_position = [screen_x, screen_y].into();
		Ok(())
	}

	fn is_proper_click(
		&self,
		button_pressed_data: &MouseButtonPressedData,
		screen_x: f32,
		screen_y: f32,
	) -> bool {
		(button_pressed_data.screen.x - screen_x).abs()
			< (1.0 / self.screen_size.width as f32) * self.click_leeway
			&& (button_pressed_data.screen.y - screen_y).abs()
				< (1.0 / self.screen_size.height as f32) * self.click_leeway
	}

	fn mouse_motion_event(
		&mut self,
		_ecs: &mut shipyard::World,
		_engine: &mut Engine<GameState>,
		abs_x: f32,
		abs_y: f32,
		_delta_x: f32,
		_delta_y: f32,
	) -> anyhow::Result<()> {
		let screen_x = abs_x / self.screen_size.width as f32;
		let screen_y = abs_y / self.screen_size.height as f32;
		if let Some(_button_pressed_data) = self.mouse_buttons_clicked.get(&MouseButton::Left) {
			let (old_map_x, old_map_y) = self.screen_ratio_to_map(screen_x, screen_y);
			let (new_map_x, new_map_y) =
				self.screen_ratio_to_map(self.mouse_last_position.x, self.mouse_last_position.y);
			let (delta_map_x, delta_map_y) = (new_map_x - old_map_x, new_map_y - old_map_y);
			self.view_center.x += delta_map_x;
			self.view_center.y += delta_map_y;
			self.tiles_meshes.clear();
		}
		self.mouse_last_position = [screen_x, screen_y].into();
		Ok(())
	}

	fn apply_cli_command(&mut self, command: cli::CliCommand) {
		use cli::{CliCommand::*, EditCommand::*};
		match command {
			Zoom { sub } => match sub {
				Set { amount } => {
					self.screen_tiles = amount;
					self.zoom = amount;
				}
				Change { amount } => self.screen_tiles += amount,
                Reset => {
                    self.screen_tiles = 2f32;
                    self.zoom = 2f32;
                }
			},

            View { x, y } => {
                self.view_center.x = x;
                self.view_center.y = y;
            },

			Clean => self.tiles_meshes.clear(),
		}
	}

	fn update(
		&mut self,
		_ecs: &mut shipyard::World,
		_engine: &mut Engine<GameState>,
	) -> anyhow::Result<()> {
		Ok(())
	}

	fn restrict_view_center(&mut self, engine: &Engine<GameState>) -> anyhow::Result<()> {
		let map = engine
			.maps
			.get(&self.visible_map)
			.context("visible map does not exist")?;

		let (_full_max_x, max_y) = Coord::new_axial(map.width, map.height).to_linear();
		if self.view_center.y < 0.0 {
			self.view_center.y = 0.0;
		} else if self.view_center.y > max_y {
			self.view_center.y = max_y;
		}

		let view_coord = Coord::from_linear(self.view_center.x, self.view_center.y);
		let (min_x, _y) = Coord::new_axial(0, view_coord.r()).to_linear();
		let (max_x, _y) = Coord::new_axial(map.width, view_coord.r()).to_linear();
		if self.view_center.x < min_x - 0.5 {
			if map.wraps_x {
				trace!("Wrapping map on X min");
				self.view_center.x += max_x - min_x + 1.0;
			} else {
				self.view_center.x = min_x;
			}
		} else if self.view_center.x > max_x + 0.5 {
			if map.wraps_x {
				trace!("Wrapping map on X max");
				self.view_center.x -= max_x - min_x + 1.0;
			} else {
				self.view_center.x = max_x;
			}
		}

		Ok(())
	}

	fn draw(
		&mut self,
		ecs: &mut shipyard::World,
		engine: &mut Engine<GameState>,
	) -> anyhow::Result<()> {
		let delta = ggez::timer::delta(&self.ctx);
		self.zoom -= (self.zoom - self.screen_tiles) * (delta.as_secs_f32() * 5.0);
		self.restrict_view_center(engine)?;
		let screen_coords = Rect::new(
			self.view_center.x - self.zoom * 0.5 * self.aspect_ratio,
			self.view_center.y - self.zoom * 0.5,
			self.zoom * self.aspect_ratio,
			self.zoom,
		);
		graphics::set_screen_coordinates(&mut self.ctx, screen_coords)?;
		graphics::clear(&mut self.ctx, graphics::BLACK);
		self.draw_map(ecs, engine)?;
		self.draw_entities(ecs, engine)?;
		self.draw_selection(ecs, engine)?;
		graphics::present(&mut self.ctx)?;
		Ok(())
	}

	fn draw_entities(
		&mut self,
		ecs: &mut shipyard::World,
		engine: &mut Engine<GameState>,
	) -> anyhow::Result<()> {
		// TODO: SpriteBatch doesn't seem terribly efficient, examine if it would be better to either cache and reuse it like the map mesh, or to build a mesh for it instead...
		if self.entity_atlas.len_atlases() != self.entity_spritebatches.len() {
			self.entity_spritebatches.clear();
			self.entity_spritebatches
				.reserve(self.entity_atlas.len_atlases());
			for i in 0..self.entity_atlas.len_atlases() {
				self.entity_spritebatches.push(SpriteBatch::new(
					self.entity_atlas
						.get_image_by_index(i)
						.context("Atlas is missing an image")?
						.clone(),
				));
			}
		}

		let tile_map = engine
			.maps
			.get(&self.visible_map)
			.with_context(|| format!("Unable to load visible map: {}", self.visible_map))?;
		let radius = self.screen_tiles * self.aspect_ratio + 1.0;
		let radius = if radius.abs() > 16.0 {
			16u8
		} else {
			radius.abs() as u8
		};
		let draw_sprites = ecs.try_borrow::<View<DrawSprite>>()?;
		let center = Coord::from_linear(self.view_center.x, self.view_center.y);
		let (center_x, center_y) = center.to_linear();
		for (co, tile) in tile_map.iter_neighbors_around(center, radius) {
			for &entity in &tile.entities {
				if let Ok(draw) = draw_sprites.get(entity) {
					if let Some(sprite) = self.entity_atlas.get_entry_by_name(&draw.sprite_name) {
						let (opx, opy) = co.to_linear();
						let px = center_x + opx;
						let py = center_y + opy;
						let idx = sprite.get_atlas_idx();
						let image_dim = self.entity_atlas.get_image(sprite.get_id()).dimensions();
						let batch = &mut self.entity_spritebatches[idx];
						let src =
							Rect::new(sprite.left(), sprite.top(), sprite.width(), sprite.height());
						let dest = [px + draw.rect.x, py + draw.rect.y];
						let offset = [0.5, 0.5];
						// No clue why the size of the sprite is dependent on the size of the source image..
						// Seems like an excessively bad mis-design...  o.O
						// So... undo that ggez brokenness...
						let scale = [
							1.0 / (image_dim.w * sprite.width()),
							1.0 / (image_dim.h * sprite.height()),
						];
						let params = DrawParam::new()
							.src(src)
							.dest(dest)
							.offset(offset)
							.scale(scale);
						batch.add(params);
					}
				}
			}
		}
		let params = DrawParam::new();
		for batch in &mut self.entity_spritebatches {
			batch.draw(&mut self.ctx, params)?;
			batch.clear();
		}

		Ok(())
	}

	fn draw_map(
		&mut self,
		_ecs: &mut shipyard::World,
		engine: &mut Engine<GameState>,
	) -> anyhow::Result<()> {
		if self.tiles_meshes.is_empty() {
			let mut mesh_builders: Vec<_> = (0..self.tiles_atlas.len_atlases())
				.map(|_| (false, graphics::MeshBuilder::new()))
				.collect();

			let tile_map = engine
				.maps
				.get(&self.visible_map)
				.with_context(|| format!("Unable to load visible map: {}", self.visible_map))?;
			let radius = self.screen_tiles * self.aspect_ratio + 1.0;
			let radius = if radius.abs() > 20.0 {
				20u8
			} else {
				radius.abs() as u8
			};
			let center = Coord::from_linear(self.view_center.x, self.view_center.y);
			let (center_x, center_y) = center.to_linear();
			for (co, tile) in tile_map.iter_neighbors_around(center, radius) {
				let (opx, opy) = co.to_linear();
				let px = center_x + opx;
				let py = center_y + opy;
				let idx: usize = tile.id.into();
				let tile_drawable = &self.tiles_drawable[idx];
				let uv = self.tiles_atlas.get_entry(tile_drawable.atlas_id);
				let mut pos = tile_drawable.info.bounds;
				pos.translate([px, py]);
				let color = tile_drawable.info.color;
				let color: [f32; 4] = [color.r, color.g, color.b, color.a];
				let (active, mesh_builder) = &mut mesh_builders[uv.get_atlas_idx()];
				*active = true;
				mesh_builder.raw(
					&[
						Vertex {
							// left-top
							pos: [pos.left(), pos.top()],
							uv: [uv.left(), uv.top()],
							color,
						},
						Vertex {
							// left-bottom
							pos: [pos.left(), pos.bottom()],
							uv: [uv.left(), uv.bottom()],
							color,
						},
						Vertex {
							// right-bottom
							pos: [pos.right(), pos.bottom()],
							uv: [uv.right(), uv.bottom()],
							color,
						},
						Vertex {
							// right-top
							pos: [pos.right(), pos.top()],
							uv: [uv.right(), uv.top()],
							color,
						},
					],
					&[0, 1, 2, 0, 2, 3],
					None,
				);
			}
			self.tiles_meshes.clear();
			for (idx, (active, mut builder)) in mesh_builders.into_iter().enumerate() {
				if !active {
					self.tiles_meshes.push(None);
				} else {
					let texture = self
						.tiles_atlas
						.get_image_by_index(idx)
						.context("failed to get image that must exist")?;
					self.tiles_meshes
						.push(Some(builder.texture(texture.clone()).build(&mut self.ctx)?));
				}
			}
		}
		let param = DrawParam::new();
		for mesh in &self.tiles_meshes {
			match mesh {
				Some(mesh) => mesh.draw(&mut self.ctx, param)?,
				None => (),
			}
		}
		Ok(())
	}

	fn draw_selection(
		&mut self,
		ecs: &mut shipyard::World,
		_engine: &mut Engine<GameState>,
	) -> anyhow::Result<()> {
		if None == self.selected_mesh {
			self.selected_mesh = Some(graphics::Mesh::new_circle(
				&mut self.ctx,
				DrawMode::stroke(0.1),
				na::Point2::new(0.0, 0.0),
				1.0,
				0.2,
				graphics::WHITE,
			)?);
		}
		let selected_mesh = &self.selected_mesh;
		let ctx = &mut self.ctx;
		if let Some(mesh) = selected_mesh {
			ecs.run(
				|coords: View<MapCoord>,
				 selected: View<components::IsSelected>|
				 -> anyhow::Result<()> {
					for (_, c) in (&selected, &coords).iter() {
						let (x, y) = c.coord.to_linear();
						mesh.draw(ctx, DrawParam::new().dest(na::Point2::new(x, y)))?;
					}

					Ok(())
				},
			)?;
		}
		Ok(())
	}
}
