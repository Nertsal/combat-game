use crate::prelude::*;

use std::collections::VecDeque;

const CURSOR_TRAIL_TIME: f32 = 0.4;
const CURSOR_FADE_TIME: f32 = 0.2;

pub struct State {
    pub geng: Geng,
    pub assets: Rc<Assets>,
    pub framebuffer_size: vec2<usize>,
    pub texture: ugli::Texture,
    pub texture_target: Aabb2<f32>,

    pub camera: Camera2d,
    /// World position of the cursor.
    pub cursor_pos: vec2<Coord>,
    pub cursor_history: VecDeque<CursorEntry>,
    pub real_time: Time,
}

#[derive(Debug, Clone, Copy)]
struct CursorEntry {
    /// World position of the cursor.
    pub pos: vec2<Coord>,
    /// Time at which the position was registered.
    pub time: Time,
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
            cursor_pos: vec2::ZERO,
            cursor_history: VecDeque::new(),
            real_time: Time::ZERO,
        }
    }
}

impl geng::State for State {
    fn update(&mut self, delta_time: f64) {
        let delta_time = r32(delta_time as f32);
        self.real_time += delta_time;

        self.cursor_history
            .retain(|entry| self.real_time - entry.time < r32(CURSOR_TRAIL_TIME));
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::CursorMove { position } => {
                let position = position.as_f32();
                let position = (position - self.texture_target.bottom_left())
                    / self.framebuffer_size.as_f32()
                    * self.texture.size().as_f32();
                self.cursor_pos = self
                    .camera
                    .screen_to_world(self.texture.size().as_f32(), position)
                    .as_r32();
                self.cursor_history.push_back(CursorEntry {
                    pos: self.cursor_pos,
                    time: self.real_time,
                });
            }
            _ => {}
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

            let vertices = self
                .cursor_history
                .iter()
                .map(|entry| {
                    let t = (self.real_time - entry.time - r32(CURSOR_FADE_TIME)).max(Time::ZERO)
                        / r32(CURSOR_TRAIL_TIME - CURSOR_FADE_TIME);
                    let t = crate::util::smoothstep(t).as_f32();

                    let color_a = Color::WHITE;
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
