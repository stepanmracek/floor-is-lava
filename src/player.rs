use bevy::prelude::*;
use interpolation::{self, Ease};

use crate::utils;

pub struct PlayersPlugin;

impl Plugin for PlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, players_init);
        app.add_systems(Update, (idle_init, control, movement));
    }
}

fn players_init(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(PlayerAnimations {
        idle: asset_server.load("models/megarex/scene.gltf#Animation0"),
        run: asset_server.load("models/megarex/scene.gltf#Animation1"),
        win: asset_server.load("models/megarex/scene.gltf#Animation3"),
        death: asset_server.load("models/megarex/scene.gltf#Animation7"),
        jump: asset_server.load("models/megarex/scene.gltf#Animation8"),
    });

    for (player, x) in [(Player::Arrows, 1.2), (Player::Wasd, -1.2)] {
        let scene = match player {
            Player::Arrows => asset_server.load("models/megarex/blue.gltf#Scene0"),
            Player::Wasd => asset_server.load("models/megarex/red.gltf#Scene0"),
        };
        info!("Scene {}", scene.id());
        let player_id = commands
            .spawn((
                PlayerBundle {
                    player,
                    speed: Speed(2.0),
                    scene: SceneBundle {
                        scene,
                        transform: Transform {
                            translation: (x, 3.5, -3.0).into(),
                            rotation: Quat::IDENTITY,
                            scale: (0.2, 0.2, 0.2).into(),
                        },
                        ..Default::default()
                    },
                },
                Idle,
            ))
            .id();
        info!("Spawned player {:?}", player_id);
    }
}

#[derive(Resource)]
struct PlayerAnimations {
    idle: Handle<AnimationClip>,
    run: Handle<AnimationClip>,
    win: Handle<AnimationClip>,
    death: Handle<AnimationClip>,
    jump: Handle<AnimationClip>,
}

fn idle_init(
    mut commands: Commands,
    animations: Res<PlayerAnimations>,
    parent_query: Query<&Parent>,
    mut animation_players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut animation_player) in &mut animation_players.iter_mut() {
        let top_parent = utils::get_top_parent(entity, &parent_query);
        animation_player.play(animations.idle.clone_weak()).repeat();

        commands
            .entity(top_parent)
            .insert(AnimationPlayerEntity(entity));
        info!(
            "{:?} initialized with animation_player: {:?}",
            top_parent, entity
        );
    }
}

#[derive(Component, Debug)]
pub enum Player {
    Wasd,
    Arrows,
}

#[derive(Component)]
struct Speed(f32);

#[derive(Component)]
struct Idle;

#[derive(Component)]
struct Moving {
    start_time: f32,
    source: Vec3,
    target: Vec3,
}

#[derive(Component)]
struct AnimationPlayerEntity(Entity);

#[derive(Bundle)]
struct PlayerBundle {
    scene: SceneBundle,
    player: Player,
    speed: Speed,
}

enum Direction {
    Right,
    Left,
    Up,
    Down,
}

fn control(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &Player,
            &AnimationPlayerEntity,
            &Speed,
        ),
        (With<Player>, With<Idle>),
    >,
    mut animation_player: Query<&mut AnimationPlayer>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    animations: Res<PlayerAnimations>,
) {
    for (entity, mut transform, player, animation_player_entity, speed) in query.iter_mut() {
        let direction = match player {
            Player::Arrows => {
                if keys.pressed(KeyCode::ArrowRight) {
                    Some(Direction::Right)
                } else if keys.pressed(KeyCode::ArrowLeft) {
                    Some(Direction::Left)
                } else if keys.pressed(KeyCode::ArrowUp) {
                    Some(Direction::Up)
                } else if keys.pressed(KeyCode::ArrowDown) {
                    Some(Direction::Down)
                } else {
                    None
                }
            }
            Player::Wasd => {
                if keys.pressed(KeyCode::KeyD) {
                    Some(Direction::Right)
                } else if keys.pressed(KeyCode::KeyA) {
                    Some(Direction::Left)
                } else if keys.pressed(KeyCode::KeyW) {
                    Some(Direction::Up)
                } else if keys.pressed(KeyCode::KeyS) {
                    Some(Direction::Down)
                } else {
                    None
                }
            }
        };

        match direction {
            Some(Direction::Right) => {
                transform.look_to(-Vec3::X, Vec3::Y);
                commands.entity(entity).remove::<Idle>().insert(Moving {
                    source: transform.translation,
                    target: transform.translation + Vec3::X,
                    start_time: time.elapsed_seconds(),
                });
            }
            Some(Direction::Left) => {
                transform.look_to(Vec3::X, Vec3::Y);
                commands.entity(entity).remove::<Idle>().insert(Moving {
                    source: transform.translation,
                    target: transform.translation - Vec3::X,
                    start_time: time.elapsed_seconds(),
                });
            }
            Some(Direction::Up) => {
                transform.look_to(Vec3::Z, Vec3::Y);
                commands.entity(entity).remove::<Idle>().insert(Moving {
                    source: transform.translation,
                    target: transform.translation + Vec3::Y - Vec3::Z,
                    start_time: time.elapsed_seconds(),
                });
            }
            Some(Direction::Down) => {
                transform.look_to(-Vec3::Z, Vec3::Y);
                commands.entity(entity).remove::<Idle>().insert(Moving {
                    source: transform.translation,
                    target: transform.translation - Vec3::Y + Vec3::Z,
                    start_time: time.elapsed_seconds(),
                });
            }
            None => {}
        };
        if direction.is_some() {
            let mut animation_player = animation_player.get_mut(animation_player_entity.0).unwrap();
            animation_player.play(animations.jump.clone_weak());
            animation_player.set_speed(speed.0);
        }
    }
}

fn movement(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Speed,
        &mut Transform,
        &Moving,
        &AnimationPlayerEntity,
        &Player,
    )>,
    mut animation_player: Query<&mut AnimationPlayer>,
    time: Res<Time>,
    animations: Res<PlayerAnimations>,
) {
    for (entity, speed, mut transform, moving, animation_player_entity, player) in query.iter_mut()
    {
        let moving_duration = time.elapsed_seconds() - moving.start_time;
        let moving_progress = moving_duration * speed.0;

        if moving_progress >= 1.0 {
            transform.translation = moving.target;

            commands.entity(entity).remove::<Moving>().insert(Idle);
            let mut animation_player = animation_player.get_mut(animation_player_entity.0).unwrap();
            animation_player.play(animations.idle.clone_weak()).repeat();
            let x = transform.translation.x.round() as i32;
            let y = transform.translation.y.round() as i32;

            info!("{:?} -> ({};{})", player, x, y);
        } else {
            let s = moving_progress.cubic_in_out();
            transform.translation = moving.source.lerp(moving.target, s);
        }
    }
}
