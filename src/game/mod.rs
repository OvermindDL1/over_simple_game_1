use anyhow::Context as AnyContext;
use ggez::graphics::{Color, DrawParam, Drawable, FilterMode, Rect, Vertex};
use ggez::input::{keyboard, mouse};
use ggez::{graphics, Context, ContextBuilder, GameError};
use winit::{
	dpi, ElementState, Event, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta,
	VirtualKeyCode, WindowEvent,
};

use ggez::conf::{FullscreenType, NumSamples, WindowMode, WindowSetup};
use ggez::nalgebra::Point2;
use over_simple_game_1::prelude::*;
use over_simple_game_1::TileType;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::fmt;
use std::path::PathBuf;

// mod point2_de_serialize {
// 	use ggez::nalgebra::Point2;
// 	use serde::de::Visitor;
// 	use serde::ser::SerializeTuple;
// 	use serde::{Deserializer, Serializer};
//
// 	pub fn serialize<S>(point: &Point2<f32>, se: S) -> Result<S::Ok, S::Error>
// 	where
// 		S: Serializer,
// 	{
// 		let mut tu = se.serialize_tuple(2)?;
// 		tu.serialize_element(&point.x)?;
// 		tu.serialize_element(&point.y)?;
// 		tu.end()
// 	}
//
// 	struct Point2Visitor();
//
// 	impl<'de> Visitor<'de> for Point2Visitor {
// 		type Value = Point2<f32>;
//
// 		fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
// 			formatter.write_str("a 2-tuple of 2 f32's")
// 		}
// 	}
//
// 	pub fn deserialize<'de, D>(de: D) -> Result<Point2<f32>, D::Error>
// 	where
// 		D: Deserializer<'de>,
// 	{
// 		de.deserialize_tuple(2, Point2Visitor())
// 	}
// }

#[derive(Debug, Serialize, Deserialize)]
enum TileDrawable {
	HexTile {
		uv: Rect,
		bounds: Rect,
	},
	#[serde(other)]
	Unmapped,
}
// struct TileDrawable {
// 	uv: Rect,
// 	bounds: Rect,
// 	// #[serde(with = "point2_de_serialize")]
// 	// tile_center: Point2<f32>,
// }

struct TilesDrawable {
	uv: Rect,
	bounds: Rect,
	color: Color,
}

struct GameState {
	ctx: Context,
	visible_map: String,
	screen_tiles: f32,
	zoom: f32,
	scale: Point2<f32>,
	view_center: Point2<f32>,
	aspect_ratio: f32,
	tiles_image: Option<graphics::Image>,
	tiles_mesh: Option<graphics::Mesh>,
	tiles_drawable: Vec<TilesDrawable>,
}

pub struct Game {
	state: GameState,
	engine: over_simple_game_1::Engine<GameState>,
	events_loop: ggez::event::EventsLoop,
	// gamepad_enabled: bool,
}

impl fmt::Debug for GameState {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("GameState")
			.field("visible_map", &self.visible_map)
			.field("screen_tiles", &self.screen_tiles)
			.field("zoom", &self.zoom)
			.field("tiles_image", &self.tiles_image)
			.field("tiles_mesh", &self.tiles_mesh)
			.finish()
	}
}

impl SimpleIO for GameState {
	type ReadError = GameError;
	type Read = ggez::filesystem::File;

	fn read(&mut self, file_path: PathBuf) -> Result<Self::Read, Self::ReadError> {
		let mut path = PathBuf::from("/");
		path.push(file_path);
		ggez::filesystem::open(&mut self.ctx, path)
	}

	type TileInterface = ();

	fn blank_tile_interface() -> Self::TileInterface {
		()
	}

	type TileAddedError = Infallible;

	fn tile_added(
		&mut self,
		index: usize,
		tile_type: &mut TileType<Self>,
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

		Ok(Game {
			state: GameState::new(ctx),
			engine: over_simple_game_1::Engine::new(),
			events_loop,
			// gamepad_enabled,
		})
	}

	pub fn setup(&mut self) -> anyhow::Result<()> {
		self.engine.setup(&mut self.state)?;
		self.state.setup(&mut self.engine)?;
		let mut generator =
			SimpleAlternationMapGenerator::new(&mut self.engine, &["dirt", "grass", "sand"])?;
		let name = self.state.visible_map.clone();
		self.engine
			.generate_map(&mut self.state, name, 0, 0, false, &mut generator)?;

		Ok(())
	}

	pub fn run(&mut self) -> anyhow::Result<()> {
		while self.state.ctx.continuing {
			self.run_once()?;
		}

		Ok(())
	}

	pub fn run_once(&mut self) -> anyhow::Result<()> {
		let state = &mut self.state;
		let events_loop = &mut self.events_loop;
		let engine = &mut self.engine;
		state.ctx.timer_context.tick();
		events_loop.poll_events(|event| {
			state.ctx.process_event(&event);
			state.dispatch_event(engine, event)
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
		self.state.update(&mut self.engine)?;
		self.state.draw(&mut self.engine)?;

		Ok(())
	}
}

impl GameState {
	fn new(ctx: Context) -> GameState {
		GameState {
			ctx,
			visible_map: "world0".to_owned(),
			screen_tiles: 2.0,
			zoom: 2.0,
			scale: Point2::from([1.0, 6.0 / 7.0]),
			view_center: Point2::from([0.0, 0.0]),
			aspect_ratio: 1.0,
			tiles_image: None,
			tiles_mesh: None,
			tiles_drawable: vec![],
		}
	}

	pub fn setup(&mut self, engine: &mut Engine<GameState>) -> anyhow::Result<()> {
		self.tiles_drawable.clear();
		self.tiles_drawable
			.reserve(engine.tile_types.tile_types.len());
		let tile_width = 120.0 / 1024.0f32;
		let tile_height = 140.0 / 2048.0f32;
		for tile_type in engine.tile_types.tile_types.iter() {
			let name: &str = &tile_type.name;
			let tile_drawable: TilesDrawable = match name {
				"dirt" => TilesDrawable {
					uv: Rect::new(732.0 / 1024.0, 710.0 / 2048.0, tile_width, tile_height),
					bounds: Rect::new(-0.5, -0.5833333, 1.0, 1.1666666),
					// bounds: Rect::new(-0.5, -2.0 / 3.0, 1.0, 4.0 / 3.0),
					color: Color::new(1.0, 1.0, 1.0, 1.0),
				},
				"grass" => TilesDrawable {
					uv: Rect::new(610.0 / 1024.0, 142.0 / 2048.0, tile_width, tile_height),
					bounds: Rect::new(-0.5, -0.5833333, 1.0, 1.1666666),
					// bounds: Rect::new(-0.5, -2.0 / 3.0, 1.0, 4.0 / 3.0),
					color: Color::new(1.0, 1.0, 1.0, 1.0),
				},
				"sand" => TilesDrawable {
					uv: Rect::new(244.0 / 1024.0, 426.0 / 2048.0, tile_width, tile_height),
					bounds: Rect::new(-0.5, -0.5833333, 1.0, 1.1666666),
					// bounds: Rect::new(-0.5, -2.0 / 3.0, 1.0, 4.0 / 3.0),
					color: Color::new(1.0, 1.0, 1.0, 1.0),
				},
				_ => TilesDrawable {
					uv: Rect::new(0.0, 0.0, 0.0, 0.0),
					bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
					color: Color::new(1.0, 0.0, 1.0, 1.0),
				},
			};
			self.tiles_drawable.push(tile_drawable);
		}
		Ok(())
	}

	fn dispatch_event(&mut self, engine: &mut Engine<GameState>, event: Event) {
		match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::Resized(logical_size) => {
					self.resize_event(engine, logical_size).unwrap();
				}
				WindowEvent::CloseRequested => {
					if self.quit_event(engine).unwrap() {
						ggez::event::quit(&mut self.ctx);
					}
				}
				WindowEvent::Focused(gained) => {
					self.focus_event(engine, gained).unwrap();
				}
				WindowEvent::ReceivedCharacter(ch) => {
					self.text_input_event(engine, ch).unwrap();
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
					self.key_down_event(engine, keycode, modifiers, repeat)
						.unwrap();
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
					self.key_up_event(engine, keycode, modifiers).unwrap();
				}
				WindowEvent::MouseWheel { delta, .. } => {
					let (x, y) = match delta {
						MouseScrollDelta::LineDelta(x, y) => (x, y),
						MouseScrollDelta::PixelDelta(dpi::LogicalPosition { x, y }) => {
							(x as f32, y as f32)
						}
					};
					self.mouse_wheel_event(engine, x, y).unwrap();
				}
				WindowEvent::MouseInput {
					state: element_state,
					button,
					..
				} => {
					let position = mouse::position(&self.ctx);
					match element_state {
						ElementState::Pressed => {
							self.mouse_button_down_event(engine, button, position.x, position.y)
								.unwrap();
						}
						ElementState::Released => {
							self.mouse_button_up_event(engine, button, position.x, position.y)
								.unwrap();
						}
					}
				}
				WindowEvent::CursorMoved { .. } => {
					let position = mouse::position(&self.ctx);
					let delta = mouse::delta(&self.ctx);
					self.mouse_motion_event(engine, position.x, position.y, delta.x, delta.y)
						.unwrap();
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
	}

	// Callbacks
	fn resize_event(
		&mut self,
		_engine: &mut Engine<GameState>,
		logical_size: dpi::LogicalSize,
	) -> anyhow::Result<()> {
		self.aspect_ratio = (logical_size.width / logical_size.height) as f32;
		dbg!(self.aspect_ratio);
		self.tiles_mesh = None;
		Ok(())
	}

	fn quit_event(&mut self, _engine: &mut Engine<GameState>) -> anyhow::Result<bool> {
		Ok(true)
	}

	fn focus_event(
		&mut self,
		_engine: &mut Engine<GameState>,
		_gained: bool,
	) -> anyhow::Result<()> {
		Ok(())
	}

	fn text_input_event(
		&mut self,
		_engine: &mut Engine<GameState>,
		_ch: char,
	) -> anyhow::Result<()> {
		Ok(())
	}

	fn key_down_event(
		&mut self,
		_engine: &mut Engine<GameState>,
		_keycode: VirtualKeyCode,
		_modifiers: ModifiersState,
		_repeat: bool,
	) -> anyhow::Result<()> {
		Ok(())
	}

	fn key_up_event(
		&mut self,
		_engine: &mut Engine<GameState>,
		_keycode: VirtualKeyCode,
		_modifiers: ModifiersState,
	) -> anyhow::Result<()> {
		Ok(())
	}

	fn mouse_wheel_event(
		&mut self,
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
		self.tiles_mesh = None;
		dbg!((y, self.screen_tiles));
		Ok(())
	}

	fn mouse_button_down_event(
		&mut self,
		_engine: &mut Engine<GameState>,
		_button: MouseButton,
		x: f32,
		y: f32,
	) -> anyhow::Result<()> {
		// let x = ;
		dbg!((x, y));
		Ok(())
	}

	fn mouse_button_up_event(
		&mut self,
		_engine: &mut Engine<GameState>,
		_button: MouseButton,
		_x: f32,
		_y: f32,
	) -> anyhow::Result<()> {
		Ok(())
	}

	fn mouse_motion_event(
		&mut self,
		_engine: &mut Engine<GameState>,
		_abs_x: f32,
		_abs_y: f32,
		_delta_x: f32,
		_delta_y: f32,
	) -> anyhow::Result<()> {
		Ok(())
	}

	fn update(&mut self, _engine: &mut Engine<GameState>) -> anyhow::Result<()> {
		Ok(())
	}

	fn draw(&mut self, engine: &mut Engine<GameState>) -> anyhow::Result<()> {
		let delta = ggez::timer::delta(&self.ctx);
		self.zoom -= (self.zoom - self.screen_tiles) * (delta.as_secs_f32() * 5.0);
		graphics::clear(&mut self.ctx, graphics::BLACK);
		// let screen = Rect::new(-300.0, -300.0, 600.0, 600.0);
		// let mut screen = self.viewable_rect;
		// screen.x -= 4.0;
		// screen.y -= 4.0;
		// screen.w += 8.0;
		// screen.h += 8.0;
		// graphics::set_screen_coordinates(&mut self.ctx, screen)?;
		let mut screen_coords = Rect::new(
			self.view_center.x - self.zoom * 0.5 * self.aspect_ratio,
			self.view_center.y - self.zoom * 0.5,
			self.zoom * self.aspect_ratio,
			self.zoom,
		);
		graphics::set_screen_coordinates(&mut self.ctx, screen_coords)?;
		self.draw_map(engine)?;
		graphics::present(&mut self.ctx)?;
		Ok(())
	}

	fn draw_map(&mut self, engine: &mut Engine<GameState>) -> anyhow::Result<()> {
		if self.tiles_mesh.is_none() {
			if self.tiles_image.is_none() {
				let mut tiles_image = graphics::Image::new(&mut self.ctx, "/tiles/map_tiles.png")?;
				tiles_image.set_filter(FilterMode::Nearest);
				self.tiles_image = Some(tiles_image);
			}

			let tiles_image = self
				.tiles_image
				.clone()
				.context("Unable to clone `tiles_image`")?;
			let mut mesh = graphics::MeshBuilder::new();
			let tile_map = &engine
				.maps
				.get(&self.visible_map)
				.with_context(|| format!("Unable to load visible map: {}", self.visible_map))?;
			let radius = self.screen_tiles * self.aspect_ratio;
			let radius = if radius.abs() > 16.0 {
				16i8
			} else {
				radius.abs() as i8
			};
			for c in
				CoordHex::from_linear(self.view_center.x, self.view_center.y).iter_neighbors(radius)
			{
				let (px, py) = c.to_linear();
				let px = px * self.scale.x;
				let py = py * self.scale.y;
				// let idx = c.idx(tile_map.width, tile_map.height);
				if let Some(tile) = tile_map.get_tile(c) {
					// let tile = &tile_map.tiles[idx];
					// let tile_type = &engine.tile_types.tile_types[tile.id as usize];
					let tile_drawable = &self.tiles_drawable[tile.id as usize];
					let uv = tile_drawable.uv;
					let mut pos = tile_drawable.bounds;
					pos.translate([px, py]);
					let color = tile_drawable.color;
					let color: [f32; 4] = [color.r, color.g, color.b, color.a];
					mesh.raw(
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
			}
			let mesh = mesh.texture(tiles_image).build(&mut self.ctx)?;
			self.tiles_mesh = Some(mesh);
		}
		let param = DrawParam::new();
		// .dest(Point2::new(0.0, 0.0))
		// .scale(Vector2::new(self.zoom, self.zoom));
		if let Some(mesh) = &self.tiles_mesh {
			mesh.draw(&mut self.ctx, param)?;
		}
		Ok(())
	}

	// fn draw_map(&mut self, ctx: &mut Context) -> GameResult<()> {
	// 	if let None = self.tiles_mesh {
	// 		if let None = self.tiles_image {
	// 			let mut tiles_image = graphics::Image::new(ctx, "/tiles/map_tiles.png")?;
	// 			tiles_image.set_filter(FilterMode::Nearest);
	// 			self.tiles_image = Some(tiles_image);
	// 		}
	//
	// 		let tiles_image = self.tiles_image.clone().unwrap();
	// 		let mut mesh = graphics::MeshBuilder::new();
	// 		for c in Coord::new(0, 0).iterate_coords_to(Coord::new(20, 20)) {
	// 			let tile_map = &self.game.maps[&self.visible_map].tiles[c.idx().0];
	// 			let tile = &self.game.tile_types[tile_map.id as usize];
	// 			// mesh.rectangle(
	// 			// 	DrawMode::fill(),
	// 			// 	Rect::new(c.x as f32, c.y as f32, 1.0, 1.0),
	// 			// 	graphics::WHITE,
	// 			// );
	// 			let tile_size = 1.0;
	// 			let uv_epsilon = if let FilterMode::Nearest = tiles_image.filter() {
	// 				0.0
	// 			} else {
	// 				// Yes this is huge but linear interpolation sucks for atlas images...
	// 				8.0 * 1024.0 * f32::EPSILON
	// 			};
	// 			mesh.raw(
	// 				&[
	// 					Vertex {
	// 						// top-left
	// 						pos: [c.x as f32, c.y as f32],
	// 						uv: [tile.uv.x + uv_epsilon, tile.uv.y + uv_epsilon],
	// 						color: [1.0, 1.0, 1.0, 1.0],
	// 					},
	// 					Vertex {
	// 						// bottom-left
	// 						pos: [c.x as f32, c.y as f32 + tile_size],
	// 						uv: [tile.uv.x + uv_epsilon, tile.uv.y + tile.uv.h - uv_epsilon],
	// 						color: [1.0, 1.0, 1.0, 1.0],
	// 					},
	// 					Vertex {
	// 						// bottom-right
	// 						pos: [c.x as f32 + tile_size, c.y as f32 + tile_size],
	// 						uv: [
	// 							tile.uv.x + tile.uv.w - uv_epsilon,
	// 							tile.uv.y + tile.uv.h - uv_epsilon,
	// 						],
	// 						color: [1.0, 1.0, 1.0, 1.0],
	// 					},
	// 					Vertex {
	// 						// top-right
	// 						pos: [c.x as f32 + tile_size, c.y as f32],
	// 						uv: [tile.uv.x + tile.uv.w - uv_epsilon, tile.uv.y + uv_epsilon],
	// 						color: [1.0, 1.0, 1.0, 1.0],
	// 					},
	// 				],
	// 				&[0, 1, 2, 0, 2, 3],
	// 				None,
	// 			);
	// 		}
	// 		let mesh = mesh.texture(tiles_image).build(ctx)?;
	// 		self.tiles_mesh = Some(mesh);
	// 	}
	// 	let param = DrawParam::new()
	// 		.dest(Point2::new(0.0, 0.0))
	// 		.scale(Vector2::new(self.zoom, self.zoom));
	// 	if let Some(mesh) = &self.tiles_mesh {
	// 		mesh.draw(ctx, param)?;
	// 	}
	// 	Ok(())
	// }
}

// impl EventHandler for GameState {
// 	fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
// 		Ok(())
// 	}
//
// 	fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
// 		graphics::clear(ctx, graphics::BLACK);
// 		// self.draw_map(ctx)?;
// 		graphics::present(ctx)
// 	}
// }
