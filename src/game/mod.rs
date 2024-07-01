use bevy::prelude::*;

pub struct GamePlugin;

pub mod components;
pub mod systems;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::setup);
        app.add_systems(
            Update,
            (
                systems::raising_lava,
                systems::camera_follow,
                systems::show_score,
            ),
        );
    }
}
