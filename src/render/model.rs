use super::*;

pub struct ModelRender {
    geng: Geng,
    assets: Rc<Assets>,
}

impl ModelRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
        }
    }

    pub fn draw(&self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        let camera = &model.camera;

        self.draw_player(model, &model.player, framebuffer);
        self.draw_cursor(model, &model.player.cursor, framebuffer);

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

    pub fn draw_cursor(&self, model: &Model, cursor: &Cursor, framebuffer: &mut ugli::Framebuffer) {
        let camera = &model.camera;

        // Cursor trail
        let vertices = cursor
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

        // Cursor
        let cursor_pos = cursor.pos + model.player.position;
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
    }

    pub fn draw_player(&self, model: &Model, player: &Player, framebuffer: &mut ugli::Framebuffer) {
        let camera = &model.camera;

        self.geng.draw2d().draw2d(
            framebuffer,
            camera,
            &draw2d::Ellipse::circle(player.position.as_f32(), 0.5, Color::WHITE),
        );

        self.draw_player_weapon(model, player.position, &player.weapon, framebuffer);
    }

    pub fn draw_player_weapon(
        &self,
        model: &Model,
        player_pos: vec2<Coord>,
        weapon: &WeaponControl,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let camera = &model.camera;

        // Position sprite
        let offset = weapon.position;
        let sword_pos = player_pos + offset;
        let sword_pos = geng_utils::pixel::pixel_perfect_aabb(
            sword_pos.as_f32(),
            vec2(0.5, 0.5),
            self.assets.sword.size(),
            camera,
            framebuffer.size().as_f32(),
        );

        // Proper rotation
        let mut angle = offset.as_f32().arg();
        if let WeaponAction::Swing(WeaponSwing {
            intent: WeaponIntent::Defend,
            ..
        })
        | WeaponAction::Charging {
            intent: WeaponIntent::Defend,
            ..
        } = &weapon.action
        {
            angle += Angle::from_degrees(50.0);
        }

        // Render sprite
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

        // Weapon trail
        let vertices = weapon
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
    }
}
