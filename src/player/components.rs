use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub enum Player {
    Red,
    Blue,
}

#[derive(Component)]
pub struct Speed(pub f32);

#[derive(Component)]
pub struct Idle;

#[derive(Component)]
pub struct Moving {
    pub start_time: f32,
    pub source: Vec3,
    pub target: Vec3,
}

#[derive(Component)]
pub struct Falling;

#[derive(Component)]
pub struct Dying {
    pub start_time: f32,
}

#[derive(Component)]
pub struct AnimationPlayerEntity(pub Entity);

#[derive(Component)]
pub struct Score(pub u32);

#[derive(Bundle)]
pub struct PlayerBundle {
    pub scene: SceneBundle,
    pub player: Player,
    pub speed: Speed,
    pub score: Score,
}

#[derive(Component)]
pub struct AI;
