use super::*;

pub struct UtilRender {
    geng: Geng,
    // assets: Rc<Assets>,
}

impl UtilRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            // assets: assets.clone(),
        }
    }
}
