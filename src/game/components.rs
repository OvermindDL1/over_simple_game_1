use ggez::graphics::Rect;
use over_simple_game_1::component_auto_loadable;
use serde::{Deserialize, Serialize};

pub struct IsSelected();

#[derive(Clone, Deserialize, Serialize)]
pub struct DrawSprite {
	pub sprite_name: String,
	pub rect: Rect,
}
component_auto_loadable!(DrawSprite);

#[derive(Clone, Deserialize, Serialize)]
pub struct Blorp {}
component_auto_loadable!(Blorp);
