use crate::prelude::*;

#[derive(geng::asset::Load)]
pub struct Assets {
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub sword: Rc<ugli::Texture>,
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub mannequin: Rc<ugli::Texture>,
}

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize)]
#[load(serde = "ron")]
pub struct Config {
    pub cursor: CursorConfig,
    pub controls: Controls,
    pub palette: Palette,
    pub player: PlayerConfig,
    pub weapon: WeaponConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Cursor")]
pub struct CursorConfig {
    pub sensitivity: Coord,
    pub trail_time: Time,
    pub fade_time: Time,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Weapon")]
pub struct WeaponConfig {
    pub acceleration: Coord,
    pub speed_max: Coord,
    pub power_min: R32,
    pub power_max: R32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Player")]
pub struct PlayerConfig {
    pub walk_speed: Coord,
    pub acceleration: Coord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Controls {
    pub attack: Vec<EventKey>,
    pub defend: Vec<EventKey>,
    pub up: Vec<EventKey>,
    pub down: Vec<EventKey>,
    pub left: Vec<EventKey>,
    pub right: Vec<EventKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Palette {
    pub idle: Color,
    pub attack: Color,
    pub defend: Color,
}
