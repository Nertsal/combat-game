use crate::prelude::*;

#[derive(geng::asset::Load)]
pub struct Assets {
    #[load(options(filter = "ugli::Filter::Nearest"))]
    pub sword: Rc<ugli::Texture>,
}

#[derive(geng::asset::Load, Debug, Clone, Serialize, Deserialize)]
#[load(serde = "ron")]
pub struct Config {
    pub cursor: CursorConfig,
    pub controls: Controls,
    pub palette: Palette,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Cursor")]
pub struct CursorConfig {
    pub trail_time: Time,
    pub fade_time: Time,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Controls {
    pub attack: Vec<EventKey>,
    pub defend: Vec<EventKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Palette {
    pub idle: Color,
    pub attack: Color,
    pub defend: Color,
}
