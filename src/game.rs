use crate::prelude::*;

use std::collections::VecDeque;

const CURSOR_TRAIL_TIME: f32 = 0.4;
const CURSOR_FADE_TIME: f32 = 0.2;

const KEYS_ATTACK: [MouseButton; 1] = [MouseButton::Left];
const KEYS_DEFEND: [MouseButton; 1] = [MouseButton::Right];

const COLOR_IDLE: Color = Color::WHITE;
const COLOR_ATTACK: Color = Color::RED;
const COLOR_DEFEND: Color = Color::BLUE;

pub struct State {
    pub geng: Geng,
    pub assets: Rc<Assets>,
    pub framebuffer_size: vec2<usize>,
    pub texture: ugli::Texture,
    pub texture_target: Aabb2<f32>,

    pub camera: Camera2d,
    pub real_time: Time,

    /// World position of the cursor.
    pub cursor_pos: vec2<Coord>,
    pub cursor_history: VecDeque<CursorEntry>,
    pub cursor_state: CursorState,

    pub player: Player,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub pos: vec2<Coord>,
    pub reach: Coord,
}

#[derive(Debug, Clone, Copy)]
pub struct CursorEntry {
    /// World position of the cursor.
    pub pos: vec2<Coord>,
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

impl State {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            framebuffer_size: vec2(1, 1),
            texture: {
                let mut texture = geng_utils::texture::new_texture(geng.ugli(), vec2(640, 360));
                texture.set_filter(ugli::Filter::Nearest);
                texture
            },
            texture_target: Aabb2::ZERO,

            camera: Camera2d {
                center: vec2::ZERO,
                rotation: Angle::ZERO,
                fov: 10.0,
            },
            real_time: Time::ZERO,

            cursor_pos: vec2::ZERO,
            cursor_history: VecDeque::new(),
            cursor_state: CursorState::Idle,

            player: Player {
                pos: vec2::ZERO,
                reach: r32(2.0),
            },
        }
    }
}

impl geng::State for State {
    fn update(&mut self, delta_time: f64) {
        let delta_time = r32(delta_time as f32);
        self.real_time += delta_time;

        {
            // Validate cursor state (in case some event is missed, e.g. when window loses focus)
            let attack = geng_utils::key::is_key_pressed(self.geng.window(), KEYS_ATTACK);
            let defend = geng_utils::key::is_key_pressed(self.geng.window(), KEYS_DEFEND);
            match self.cursor_state {
                CursorState::Idle => {
                    if attack {
                        self.cursor_state = CursorState::Attack;
                    } else if defend {
                        self.cursor_state = CursorState::Defend;
                    }
                }
                CursorState::Attack => {
                    if !attack {
                        self.cursor_state = CursorState::Idle;
                    }
                }
                CursorState::Defend => {
                    if !defend {
                        self.cursor_state = CursorState::Idle;
                    }
                }
            }
        }

        self.cursor_history
            .retain(|entry| self.real_time - entry.time < r32(CURSOR_TRAIL_TIME));
    }

    fn handle_event(&mut self, event: geng::Event) {
        // Cursor state
        if geng_utils::key::is_event_press(&event, KEYS_ATTACK) {
            self.cursor_state = CursorState::Attack;
        } else if geng_utils::key::is_event_release(&event, KEYS_ATTACK) {
            if let CursorState::Attack = self.cursor_state {
                self.cursor_state = CursorState::Idle;
            }
        } else if geng_utils::key::is_event_press(&event, KEYS_DEFEND) {
            self.cursor_state = CursorState::Defend;
        } else if geng_utils::key::is_event_release(&event, KEYS_DEFEND) {
            if let CursorState::Defend = self.cursor_state {
                self.cursor_state = CursorState::Idle;
            }
        }

        if let geng::Event::CursorMove { position } = event {
            let position = position.as_f32();
            let position = (position - self.texture_target.bottom_left())
                / self.framebuffer_size.as_f32()
                * self.texture.size().as_f32();
            let position = self
                .camera
                .screen_to_world(self.texture.size().as_f32(), position)
                .as_r32();

            // Clamp by reach
            let position =
                self.player.pos + (position - self.player.pos).clamp_len(..=self.player.reach);
            self.cursor_pos = position;

            self.cursor_history.push_back(CursorEntry {
                pos: self.cursor_pos,
                time: self.real_time,
                state: self.cursor_state,
            });
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(Color::BLACK), None, None);

        let screen = Aabb2::ZERO.extend_positive(framebuffer.size().as_f32());
        self.texture_target =
            geng_utils::layout::fit_aabb(self.texture.size().as_f32(), screen, vec2(0.5, 0.5));

        {
            let framebuffer =
                &mut geng_utils::texture::attach_texture(&mut self.texture, self.geng.ugli());
            ugli::clear(framebuffer, Some(Color::BLACK), None, None);

            let camera = &self.camera;

            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Ellipse::circle(self.player.pos.as_f32(), 0.5, Color::WHITE),
            );

            let offset = self.cursor_pos - self.player.pos;
            let sword_pos = self.player.pos + offset.clamp_len(..=self.player.reach);
            let sword_pos = geng_utils::pixel::pixel_perfect_aabb(
                sword_pos.as_f32(),
                vec2(0.5, 0.5),
                self.assets.sword.size(),
                camera,
                framebuffer.size().as_f32(),
            );

            let mut angle = offset.as_f32().arg();
            if let CursorState::Defend = self.cursor_state {
                angle += Angle::from_degrees(50.0);
            }

            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::TexturedQuad::new(
                    Aabb2::ZERO.extend_symmetric(sword_pos.size() / 2.0),
                    &*self.assets.sword,
                )
                .rotate(angle)
                .translate(sword_pos.center()),
            );

            let vertices = self
                .cursor_history
                .iter()
                .map(|entry| {
                    let t = (self.real_time - entry.time - r32(CURSOR_FADE_TIME)).max(Time::ZERO)
                        / r32(CURSOR_TRAIL_TIME - CURSOR_FADE_TIME);
                    let t = crate::util::smoothstep(t).as_f32();

                    let color_a = match entry.state {
                        CursorState::Idle => COLOR_IDLE,
                        CursorState::Attack => COLOR_ATTACK,
                        CursorState::Defend => COLOR_DEFEND,
                    };
                    let color_b = Color { a: 0.0, ..color_a };

                    draw2d::ColoredVertex {
                        a_pos: entry.pos.as_f32(),
                        a_color: Color::lerp(color_a, color_b, t),
                    }
                })
                .collect();
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Chain::new_gradient(vertices, 0.1, 0),
            );
        }

        let mut draw = geng_utils::texture::DrawTexture::new(&self.texture);
        draw.target = self.texture_target;
        draw.draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);
    }
}
