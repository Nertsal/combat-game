mod model;
mod util;

use model::ModelRender;
pub use util::UtilRender;

use crate::prelude::*;

pub struct GameRender {
    geng: Geng,
    render: ModelRender,
    pub framebuffer_size: vec2<usize>,
    pub texture: ugli::Texture,
    pub texture_target: Aabb2<f32>,
}

impl GameRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            // assets: assets.clone(),
            render: ModelRender::new(geng, assets),
            framebuffer_size: vec2(1, 1),
            texture: {
                let mut texture = geng_utils::texture::new_texture(geng.ugli(), vec2(640, 360));
                texture.set_filter(ugli::Filter::Nearest);
                texture
            },
            texture_target: Aabb2::ZERO,
        }
    }

    pub fn draw(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size();

        let screen = Aabb2::ZERO.extend_positive(framebuffer.size().as_f32());
        self.texture_target =
            geng_utils::layout::fit_aabb(self.texture.size().as_f32(), screen, vec2(0.5, 0.5));

        {
            let framebuffer =
                &mut geng_utils::texture::attach_texture(&mut self.texture, self.geng.ugli());
            ugli::clear(framebuffer, Some(Color::BLACK), None, None);
            self.render.draw(model, framebuffer);
        }

        let mut draw = geng_utils::texture::DrawTexture::new(&self.texture);
        draw.target = self.texture_target;
        draw.draw(&geng::PixelPerfectCamera, &self.geng, framebuffer);
    }
}
