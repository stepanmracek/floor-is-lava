use bevy::prelude::*;

pub struct GamePlugin;

pub mod components;
pub mod systems;

#[derive(States, Debug, Default, Hash, PartialEq, Eq, Clone)]
pub enum GameState {
    #[default]
    Start,
    InGame,
    Pause,
    End
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>();
        app.add_systems(Startup, systems::setup);
        app.add_systems(
            Update,
            (
                systems::raising_lava.run_if(in_state(GameState::InGame)),
                systems::maybe_pause.run_if(in_state(GameState::InGame)),
                systems::waiting_for_start.run_if(in_state(GameState::Start)),
                systems::waiting_for_start.run_if(in_state(GameState::Pause)),
                systems::camera_follow,
                systems::show_score,
            ),
        );
        app.add_systems(OnEnter(GameState::Start), systems::start_entered);
        app.add_systems(OnExit(GameState::Start), systems::start_exit);
        app.add_systems(OnEnter(GameState::Pause), systems::pause_entered);
        app.add_systems(OnExit(GameState::Pause), systems::pause_exit);
    }
}
