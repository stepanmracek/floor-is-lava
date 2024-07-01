use bevy::prelude::*;

mod block;
mod game;
mod player;
mod utils;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            player::PlayersPlugin,
            block::BlocksPlugin,
            game::GamePlugin,
        ))
        .run();
}
