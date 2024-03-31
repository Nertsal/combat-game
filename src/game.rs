use crate::prelude::*;

use std::collections::VecDeque;

pub struct State {
    pub geng: Geng,
    pub assets: Rc<Assets>,
    pub config: Config,

    pub framebuffer_size: vec2<usize>,
    pub texture: ugli::Texture,
    pub texture_target: Aabb2<f32>,

    pub camera: Camera2d,
    pub real_time: Time,

    pub cursor: Cursor,
    pub player: Player,

    pub floating_texts: Vec<FloatingText>,
}

#[derive(Debug, Clone)]
pub struct Cursor {
    /// World position of the cursor.
    pub pos: vec2<Coord>,
    pub history: VecDeque<CursorEntry>,
    pub state: CursorState,
    /// State on the previous frame.
    pub last_state: CursorState,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub position: vec2<Coord>,
    pub velocity: vec2<Coord>,
    pub target_move_dir: vec2<Coord>,
    pub weapon: WeaponControl,
}

#[derive(Debug, Clone)]
pub struct WeaponControl {
    pub reach: Coord,
    /// Relative position of the weapon tip.
    pub position: vec2<Coord>,
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
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: Config) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            config,

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

            cursor: Cursor {
                pos: vec2::ZERO,
                history: VecDeque::new(),
                state: CursorState::Idle,
                last_state: CursorState::Idle,
            },
            player: Player {
                position: vec2::ZERO,
                velocity: vec2::ZERO,
                target_move_dir: vec2::ZERO,
                weapon: WeaponControl {
                    reach: r32(2.0),
                    position: vec2::ZERO,
                },
            },

            floating_texts: Vec::new(),
        }
    }

    fn check_action(&mut self) {
        let Some(end) = self
            .cursor
            .history
            .iter()
            .rev()
            .position(|entry| entry.state == self.cursor.last_state)
        else {
            return;
        };

        let start = self
            .cursor
            .history
            .iter()
            .rev()
            .skip(end)
            .position(|entry| entry.state != self.cursor.last_state)
            .map(|len| self.cursor.history.len() - end - len)
            .unwrap_or(0);
        let end = self.cursor.history.len() - 1 - end;

        let start = self.cursor.history[start];
        let end = self.cursor.history[end];

        let pos = (start.pos + end.pos) / r32(2.0);

        let time = (end.time - start.time) / self.config.cursor.trail_time;

        let power_t = time; // Maybe sqrt
        let power = power_t * (self.config.weapon.power_max - self.config.weapon.power_min)
            + self.config.weapon.power_min;

        let text = match end.state {
            CursorState::Idle => return,
            CursorState::Attack => "Slash",
            CursorState::Defend => "Parry",
        };

        log::debug!(
            "{} at ({:.02}, {:.02}), power: {:.02}",
            text,
            pos.x,
            pos.y,
            power
        );

        let degrees = r32(thread_rng().gen_range(-15.0..=15.0));
        self.floating_texts.push(FloatingText {
            text: format!("{} {}", text, power.ceil()),
            pos,
            lifetime: Bounded::new_max(r32(0.5)),
            initial_scale: r32(1.0),
            rotation: Angle::from_degrees(degrees),
        });
    }
}

impl geng::State for State {
    fn update(&mut self, delta_time: f64) {
        let delta_time = r32(delta_time as f32);
        self.real_time += delta_time;

        for text in &mut self.floating_texts {
            text.lifetime.change(-delta_time);
        }
        self.floating_texts
            .retain(|text| text.lifetime.is_above_min());

        {
            // Validate cursor state (in case some event is missed, e.g. when window loses focus)
            let attack =
                geng_utils::key::is_key_pressed(self.geng.window(), &self.config.controls.attack);
            let defend =
                geng_utils::key::is_key_pressed(self.geng.window(), &self.config.controls.defend);
            match self.cursor.state {
                CursorState::Idle => {
                    if attack {
                        self.cursor.state = CursorState::Attack;
                    } else if defend {
                        self.cursor.state = CursorState::Defend;
                    }
                }
                CursorState::Attack => {
                    if !attack {
                        self.cursor.state = CursorState::Idle;
                    }
                }
                CursorState::Defend => {
                    if !defend {
                        self.cursor.state = CursorState::Idle;
                    }
                }
            }
        }
        if self.cursor.state != self.cursor.last_state {
            self.check_action();
            self.cursor.last_state = self.cursor.state;
        }

        self.cursor
            .history
            .retain(|entry| self.real_time - entry.time < self.config.cursor.trail_time);

        let mut move_dir = vec2::<f32>::ZERO;
        if geng_utils::key::is_key_pressed(self.geng.window(), &self.config.controls.up) {
            move_dir.y += 1.0;
        }
        if geng_utils::key::is_key_pressed(self.geng.window(), &self.config.controls.down) {
            move_dir.y -= 1.0;
        }
        if geng_utils::key::is_key_pressed(self.geng.window(), &self.config.controls.left) {
            move_dir.x -= 1.0;
        }
        if geng_utils::key::is_key_pressed(self.geng.window(), &self.config.controls.right) {
            move_dir.x += 1.0;
        }
        self.player.target_move_dir = move_dir.as_r32();

        let target_velocity = self.player.target_move_dir * self.config.player.walk_speed;
        self.player.velocity += (target_velocity - self.player.velocity)
            .clamp_len(..=self.config.player.acceleration * delta_time);

        self.player.position += self.player.velocity * delta_time;

        let weapon_target =
            (self.cursor.pos - self.player.position).clamp_len(..=self.player.weapon.reach);
        self.player.weapon.position = weapon_target;
    }

    fn handle_event(&mut self, event: geng::Event) {
        // Cursor state
        if geng_utils::key::is_event_press(&event, &self.config.controls.attack) {
            self.cursor.state = CursorState::Attack;
        } else if geng_utils::key::is_event_release(&event, &self.config.controls.attack) {
            if let CursorState::Attack = self.cursor.state {
                self.cursor.state = CursorState::Idle;
            }
        } else if geng_utils::key::is_event_press(&event, &self.config.controls.defend) {
            self.cursor.state = CursorState::Defend;
        } else if geng_utils::key::is_event_release(&event, &self.config.controls.defend) {
            if let CursorState::Defend = self.cursor.state {
                self.cursor.state = CursorState::Idle;
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
            let position = self.player.position
                + (position - self.player.position).clamp_len(..=self.player.weapon.reach);
            self.cursor.pos = position;

            self.cursor.history.push_back(CursorEntry {
                pos: self.cursor.pos,
                time: self.real_time,
                state: self.cursor.state,
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
                &draw2d::Ellipse::circle(self.player.position.as_f32(), 0.5, Color::WHITE),
            );

            let offset = self.player.weapon.position;
            let sword_pos = self.player.position + offset;
            let sword_pos = geng_utils::pixel::pixel_perfect_aabb(
                sword_pos.as_f32(),
                vec2(0.5, 0.5),
                self.assets.sword.size(),
                camera,
                framebuffer.size().as_f32(),
            );

            let mut angle = offset.as_f32().arg();
            if let CursorState::Defend = self.cursor.state {
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
                .cursor
                .history
                .iter()
                .map(|entry| {
                    let t = (self.real_time - entry.time - self.config.cursor.fade_time)
                        .max(Time::ZERO)
                        / (self.config.cursor.trail_time - self.config.cursor.fade_time);
                    let t = crate::util::smoothstep(t).as_f32();

                    let color_a = match entry.state {
                        CursorState::Idle => self.config.palette.idle,
                        CursorState::Attack => self.config.palette.attack,
                        CursorState::Defend => self.config.palette.defend,
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

            for text in &self.floating_texts {
                let t = text.lifetime.get_ratio().as_f32();
                let t = crate::util::smoothstep(t);
                let scale = text.initial_scale.as_f32() * t;

                let transform = mat3::translate(text.pos.as_f32())
                    * mat3::rotate(text.rotation.map(R32::as_f32))
                    * mat3::scale_uniform(scale);
                let color = Color::lerp(Color::WHITE, Color::TRANSPARENT_WHITE, t);

                self.geng.default_font().draw(
                    framebuffer,
                    camera,
                    &text.text,
                    vec2::splat(geng::TextAlign(0.5)),
                    transform,
                    color,
                );
            }
        }

        let mut draw = geng_utils::texture::DrawTexture::new(&self.texture);
        draw.target = self.texture_target;
        draw.draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);
    }
}
