use bevy::prelude::*;

pub mod components;
pub mod resources;
pub mod systems;

use crate::game;

pub struct PlayersPlugin;

impl Plugin for PlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, systems::players_init);
        app.add_systems(
            Update,
            (
                systems::idle_init,
                systems::key_control,
                systems::ai_control,
                systems::moving,
                systems::falling,
                systems::lava_contact,
                systems::dying,
            )
                .run_if(in_state(game::GameState::InGame)),
        );
    }
}

pub const PLAYER_START_Y: f32 = 3.0;
pub const BLUE_PLAYER_X_OFFSET: f32 = 0.2;
pub const BLUE_PLAYER_START_POS_X: f32 = 1.0;
pub const RED_PLAYER_X_OFFSET: f32 = -0.2;
pub const RED_PLAYER_START_POS_X: f32 = -1.0;
