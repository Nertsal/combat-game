use super::*;

/// Event sent to the model.
#[derive(Debug, Clone)]
pub enum Event {
    CursorMove { delta: vec2<Coord> },
    Charge(WeaponIntent),
    Release,
}
