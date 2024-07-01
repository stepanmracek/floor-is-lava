use bevy::prelude::*;

pub mod components;
pub mod resources;
pub mod systems;

pub struct BlocksPlugin;

impl Plugin for BlocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::blocks_init);
        app.add_systems(Update, (systems::block_in_lava, systems::block_generator));
    }
}
