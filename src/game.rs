use crate::prelude::*;

pub struct State {
    pub geng: Geng,
    pub assets: Rc<Assets>,

    pub framebuffer_size: vec2<usize>,
    pub texture: ugli::Texture,
    pub texture_target: Aabb2<f32>,

    pub model: Model,
}

impl State {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: Config) -> Self {
        geng.window().lock_cursor();

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

            model: Model::new(geng, config),
        }
    }
}

impl geng::State for State {
    fn update(&mut self, delta_time: f64) {
        let delta_time = r32(delta_time as f32);
        self.model.update(delta_time);
    }

    fn handle_event(&mut self, event: geng::Event) {
        // Cursor state
        if geng_utils::key::is_event_press(&event, &self.model.config.controls.attack) {
            self.model.handle_event(Event::Charge(WeaponIntent::Attack));
        } else if geng_utils::key::is_event_release(&event, &self.model.config.controls.attack) {
            if let CursorState::Attack = self.model.player.cursor.state {
                self.model.handle_event(Event::Release);
            }
        } else if geng_utils::key::is_event_press(&event, &self.model.config.controls.defend) {
            self.model.handle_event(Event::Charge(WeaponIntent::Defend));
        } else if geng_utils::key::is_event_release(&event, &self.model.config.controls.defend) {
            if let CursorState::Defend = self.model.player.cursor.state {
                self.model.handle_event(Event::Release);
            }
        }

        if let geng::Event::RawMouseMove { delta } = event {
            let delta = delta.as_r32();
            // Convert to world coordinates
            let delta = delta
                * r32(self.model.camera.fov / self.framebuffer_size.y as f32)
                * self.model.config.cursor.sensitivity;
            self.model.handle_event(Event::CursorMove { delta });
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();
        ugli::clear(framebuffer, Some(Color::BLACK), None, None);

        let screen = Aabb2::ZERO.extend_positive(framebuffer.size().as_f32());
        self.texture_target =
            geng_utils::layout::fit_aabb(self.texture.size().as_f32(), screen, vec2(0.5, 0.5));

        {
            let model = &self.model;

            let framebuffer =
                &mut geng_utils::texture::attach_texture(&mut self.texture, self.geng.ugli());
            ugli::clear(framebuffer, Some(Color::BLACK), None, None);

            let camera = &model.camera;

            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Ellipse::circle(model.player.position.as_f32(), 0.5, Color::WHITE),
            );

            let offset = model.player.weapon.position;
            let sword_pos = model.player.position + offset;
            let sword_pos = geng_utils::pixel::pixel_perfect_aabb(
                sword_pos.as_f32(),
                vec2(0.5, 0.5),
                self.assets.sword.size(),
                camera,
                framebuffer.size().as_f32(),
            );

            let mut angle = offset.as_f32().arg();
            if let WeaponAction::Swing(WeaponSwing {
                intent: WeaponIntent::Defend,
                ..
            })
            | WeaponAction::Charging {
                intent: WeaponIntent::Defend,
                ..
            } = &model.player.weapon.action
            {
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

            // Cursor trail
            let vertices = model
                .player
                .cursor
                .history
                .iter()
                .map(|entry| {
                    let t = (model.real_time - entry.time - model.config.cursor.fade_time)
                        .max(Time::ZERO)
                        / (model.config.cursor.trail_time - model.config.cursor.fade_time);
                    let t = crate::util::smoothstep(t).as_f32();

                    let mut color_a = match entry.state {
                        CursorState::Idle => Rgba::TRANSPARENT_BLACK, // self.config.palette.idle,
                        CursorState::Attack => model.config.palette.attack,
                        CursorState::Defend => model.config.palette.defend,
                    };
                    color_a.a *= 0.5;
                    let color_b = Color { a: 0.0, ..color_a };

                    draw2d::ColoredVertex {
                        a_pos: (model.player.position + entry.relative_pos).as_f32(),
                        a_color: Color::lerp(color_a, color_b, t),
                    }
                })
                .collect();
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Chain::new_gradient(vertices, 0.07, 0),
            );

            // Weapon trail
            let vertices = model
                .player
                .weapon
                .history
                .iter()
                .map(|entry| {
                    let t = (model.real_time - entry.time - model.config.cursor.fade_time)
                        .max(Time::ZERO)
                        / (model.config.cursor.trail_time - model.config.cursor.fade_time);
                    let t = crate::util::smoothstep(t).as_f32();

                    let color_a = match entry.state {
                        CursorState::Idle => model.config.palette.idle,
                        CursorState::Attack => model.config.palette.attack,
                        CursorState::Defend => model.config.palette.defend,
                    };
                    let color_b = Color { a: 0.0, ..color_a };

                    draw2d::ColoredVertex {
                        a_pos: entry.world_pos.as_f32(),
                        a_color: Color::lerp(color_a, color_b, t),
                    }
                })
                .collect();
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Chain::new_gradient(vertices, 0.1, 0),
            );

            // Cursor
            let cursor_pos = model.player.cursor.pos + model.player.position;
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Ellipse::circle(cursor_pos.as_f32(), 0.15, Rgba::BLACK),
            );
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Ellipse::circle(cursor_pos.as_f32(), 0.1, Rgba::WHITE),
            );

            for text in &model.floating_texts {
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

            // if let Some(action) = &self.player.weapon.action {
            //     let chain = action.arc.map(R32::as_f32).chain(50);
            //     let chain = draw2d::Chain::new(chain, 0.05, Rgba::BLUE, 0);
            //     self.geng.draw2d().draw2d(framebuffer, &self.camera, &chain);
            // }
        }

        let mut draw = geng_utils::texture::DrawTexture::new(&self.texture);
        draw.target = self.texture_target;
        draw.draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);
    }
}
