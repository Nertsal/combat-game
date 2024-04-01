use crate::{prelude::*, render::GameRender};

pub struct State {
    // geng: Geng,
    // assets: Rc<Assets>,
    render: GameRender,

    pub model: Model,
}

impl State {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: Config) -> Self {
        geng.window().lock_cursor();

        Self {
            // geng: geng.clone(),
            // assets: assets.clone(),
            render: GameRender::new(geng, assets),

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
                * r32(self.model.camera.fov / self.render.framebuffer_size.y as f32)
                * self.model.config.cursor.sensitivity;
            self.model.handle_event(Event::CursorMove { delta });
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Color::BLACK), None, None);
        self.render.draw(&self.model, framebuffer);
    }
}
