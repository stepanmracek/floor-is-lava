use bevy::prelude::*;

#[derive(Resource)]
pub struct PlayerAnimations {
    pub idle: Handle<AnimationClip>,
    pub run: Handle<AnimationClip>,
    pub win: Handle<AnimationClip>,
    pub falling: Handle<AnimationClip>,
    pub death: Handle<AnimationClip>,
    pub jump: Handle<AnimationClip>,
}
