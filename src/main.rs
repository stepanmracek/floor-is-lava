use bevy::prelude::*;

mod block;
mod game;
mod player;
mod utils;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::ORANGE_RED,
            brightness: 100.,
        })
        .add_plugins((
            DefaultPlugins,
            player::PlayersPlugin,
            block::BlocksPlugin,
            game::GamePlugin,
        ))
        .run();
}
