mod collider;
mod event;
mod logic;

pub use self::{collider::*, event::Event};

use crate::{prelude::*, util::parabola::Parabola};

pub struct Model {
    pub geng: Geng, // TODO: Somehow not require the engine ref (used for keys pressed)
    pub config: Config,

    pub camera: Camera2d,
    pub real_time: Time,
    pub game_time: Time,

    pub player: Player,
    pub mannequins: Vec<Mannequin>,

    pub floating_texts: Vec<FloatingText>,
}

#[derive(Debug, Clone)]
pub struct Mannequin {
    pub collider: Collider,
    /// Last time the mannequin got hit.
    pub hit_time: Time,
}

#[derive(Debug, Clone)]
pub struct Cursor {
    /// Relative position of the cursor.
    pub pos: vec2<Coord>,
    pub history: VecDeque<CursorEntry>,
    pub state: CursorState,
    /// State on the previous frame.
    pub last_state: CursorState,
}

#[derive(Debug, Clone)]
pub struct Player {
    /// Cursor relative to the player position.
    pub cursor: Cursor,
    pub position: vec2<Coord>,
    pub velocity: vec2<Coord>,
    pub target_move_dir: vec2<Coord>,
    pub weapon: WeaponControl,
}

#[derive(Debug, Clone)]
pub struct WeaponControl {
    pub history: VecDeque<CursorEntry>,
    pub reach: Coord,
    pub acceleration: Coord,
    pub swing_boost: Coord,
    pub speed_max: Coord,
    /// Relative position of the weapon tip.
    pub position: vec2<Coord>,
    /// Relative velocity of the weapon tip.
    pub velocity: vec2<Coord>,
    pub action: WeaponAction,
}

#[derive(Debug, Clone)]
pub enum WeaponAction {
    Idle {
        target: vec2<Coord>,
    },
    Charging {
        target: vec2<Coord>,
        intent: WeaponIntent,
    },
    Swing(WeaponSwing),
}

impl WeaponAction {
    pub fn swinging(&self) -> bool {
        matches!(self, Self::Swing(_))
    }
}

#[derive(Debug, Clone)]
pub struct WeaponSwing {
    pub intent: WeaponIntent,
    pub power: R32,
    pub arc: Parabola<Coord>,
}

#[derive(Debug, Clone, Copy)]
pub enum WeaponIntent {
    Attack,
    Defend,
}

#[derive(Debug, Clone)]
pub struct FloatingText {
    pub text: String,
    pub pos: vec2<Coord>,
    pub lifetime: Bounded<Time>,
    pub initial_scale: Coord,
    pub rotation: Angle<Coord>,
}

#[derive(Debug, Clone, Copy)]
pub struct CursorEntry {
    /// World position of the cursor.
    pub world_pos: vec2<Coord>,
    /// Position relative to the player.
    pub relative_pos: vec2<Coord>,
    /// Time at which the position was registered.
    pub time: Time,
    pub state: CursorState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorState {
    Idle,
    Attack,
    Defend,
}

impl Model {
    pub fn new(geng: &Geng, config: Config) -> Self {
        Self {
            geng: geng.clone(),

            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            real_time: Time::ZERO,
            game_time: Time::ZERO,

            player: Player {
                cursor: Cursor {
                    pos: vec2::ZERO,
                    history: VecDeque::new(),
                    state: CursorState::Idle,
                    last_state: CursorState::Idle,
                },
                position: vec2::ZERO,
                velocity: vec2::ZERO,
                target_move_dir: vec2::ZERO,
                weapon: WeaponControl {
                    history: VecDeque::new(),
                    acceleration: config.weapon.acceleration,
                    swing_boost: config.weapon.swing_boost,
                    speed_max: config.weapon.speed_max,
                    reach: r32(2.0),
                    position: vec2::ZERO,
                    velocity: vec2::ZERO,
                    action: WeaponAction::Idle { target: vec2::ZERO },
                },
            },
            mannequins: vec![Mannequin {
                collider: Collider::aabb(Aabb2::point(vec2(3.0, 2.0)).extend_uniform(0.3).map(r32)),
                hit_time: Time::ZERO,
            }],

            floating_texts: Vec::new(),

            config,
        }
    }
}
