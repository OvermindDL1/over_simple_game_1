mod components;
mod coords;
mod game_map;

use ggez::event::EventHandler;
use ggez::{graphics, Context, GameResult};

use crate::game::coords::Coord;
use game_map::*;
use ggez::graphics::DrawMode::Fill;
use ggez::graphics::{DrawMode, DrawParam, Drawable, FilterMode, Rect, Vertex};
use ggez::nalgebra::{Point2, Vector2};

pub struct GameState {
	ecs: shipyard::World,
	game: Game,
	visible_map: String,
	zoom: f32, // Not a true zoom, more like 1 == 1 pixel per tile, so 16 is 16 pixels per tile
	tiles_image: Option<graphics::Image>,
	tiles_mesh: Option<graphics::Mesh>,
}

impl GameState {
	pub fn new() -> GameState {
		let mut game_state = GameState {
			ecs: shipyard::World::new(),
			game: Game::new(),
			visible_map: "world0".to_owned(),
			zoom: 128.0,
			tiles_image: None,
			tiles_mesh: None,
		};

		game_state.game.generate_map(
			game_state.visible_map.to_owned(),
			255,
			255,
			true,
			false,
			Box::new(SimpleMapGenerator::new(&game_state.game.tile_types)),
		);

		game_state
	}

	fn draw_map(&mut self, ctx: &mut Context) -> GameResult<()> {
		if let None = self.tiles_mesh {
			if let None = self.tiles_image {
				let mut tiles_image = graphics::Image::new(ctx, "/map_tiles.png")?;
				tiles_image.set_filter(FilterMode::Nearest);
				self.tiles_image = Some(tiles_image);
			}

			let tiles_image = self.tiles_image.clone().unwrap();
			let mut mesh = graphics::MeshBuilder::new();
			for c in Coord::new(0, 0).iterate_coords_to(Coord::new(20, 20)) {
				let tile_map = &self.game.maps[&self.visible_map].tiles[c.idx().0];
				let tile = &self.game.tile_types[tile_map.id as usize];
				// mesh.rectangle(
				// 	DrawMode::fill(),
				// 	Rect::new(c.x as f32, c.y as f32, 1.0, 1.0),
				// 	graphics::WHITE,
				// );
				let tile_size = 1.0;
				let uv_epsilon = if let FilterMode::Nearest = tiles_image.filter() {
					0.0
				} else {
					// Yes this is huge but linear interpolation sucks for atlas images...
					8.0 * 1024.0 * f32::EPSILON
				};
				mesh.raw(
					&[
						Vertex {
							// top-left
							pos: [c.x as f32, c.y as f32],
							uv: [tile.uv.x + uv_epsilon, tile.uv.y + uv_epsilon],
							color: [1.0, 1.0, 1.0, 1.0],
						},
						Vertex {
							// bottom-left
							pos: [c.x as f32, c.y as f32 + tile_size],
							uv: [tile.uv.x + uv_epsilon, tile.uv.y + tile.uv.h - uv_epsilon],
							color: [1.0, 1.0, 1.0, 1.0],
						},
						Vertex {
							// bottom-right
							pos: [c.x as f32 + tile_size, c.y as f32 + tile_size],
							uv: [
								tile.uv.x + tile.uv.w - uv_epsilon,
								tile.uv.y + tile.uv.h - uv_epsilon,
							],
							color: [1.0, 1.0, 1.0, 1.0],
						},
						Vertex {
							// top-right
							pos: [c.x as f32 + tile_size, c.y as f32],
							uv: [tile.uv.x + tile.uv.w - uv_epsilon, tile.uv.y + uv_epsilon],
							color: [1.0, 1.0, 1.0, 1.0],
						},
					],
					&[0, 1, 2, 0, 2, 3],
					None,
				);
			}
			let mesh = mesh.texture(tiles_image).build(ctx)?;
			self.tiles_mesh = Some(mesh);
		}
		let param = DrawParam::new()
			.dest(Point2::new(0.0, 0.0))
			.scale(Vector2::new(self.zoom, self.zoom));
		if let Some(mesh) = &self.tiles_mesh {
			mesh.draw(ctx, param)?;
		}
		Ok(())
	}
}

impl EventHandler for GameState {
	fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
		graphics::clear(ctx, graphics::BLACK);
		self.draw_map(ctx)?;
		graphics::present(ctx)
	}
}
