mod assets;
mod game;
mod model;
mod prelude;
mod render;
mod util;

use prelude::*;

#[derive(clap::Parser)]
struct Opts {
    #[clap(flatten)]
    geng: geng::CliArgs,
}

fn main() {
    logger::init();
    geng::setup_panic_handler();

    let opts: Opts = clap::Parser::parse();

    let mut options = geng::ContextOptions::default();
    options.window.title = "Geng Game".to_string();
    options.window.antialias = false;
    // options.fixed_delta_time = 1.0 / FIXED_FPS;
    options.with_cli(&opts.geng);

    Geng::run_with(&options, move |geng| async move {
        let manager = geng.asset_manager();
        let assets_path = run_dir().join("assets");

        let assets = <Assets as geng::asset::Load>::load(manager, &assets_path, &())
            .await
            .unwrap();
        let assets = Rc::new(assets);

        let config =
            <Config as geng::asset::Load>::load(manager, &assets_path.join("config.ron"), &())
                .await
                .unwrap();

        let state = game::State::new(&geng, &assets, config);
        geng.run_state(state).await;
    });
}
