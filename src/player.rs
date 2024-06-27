use bevy::prelude::*;
use interpolation::{self, Ease};
use rand::Rng;

use crate::block;
use crate::utils;
use crate::Lava;

pub struct PlayersPlugin;

impl Plugin for PlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, players_init);
        app.add_systems(
            Update,
            (idle_init, control, movement, falling, lava_contact, dying),
        );
    }
}

pub const PLAYER_START_Y: f32 = 3.0;
pub const ARROWS_PLAYER_X_OFFSET: f32 = 0.2;
pub const ARROWS_PLAYER_START_POS_X: f32 = 1.0;
pub const WASD_PLAYER_X_OFFSET: f32 = -0.2;
pub const WASD_PLAYER_START_POS_X: f32 = -1.0;

fn players_init(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(PlayerAnimations {
        idle: asset_server.load("models/megarex/scene.gltf#Animation0"),
        run: asset_server.load("models/megarex/scene.gltf#Animation1"),
        win: asset_server.load("models/megarex/scene.gltf#Animation3"),
        falling: asset_server.load("models/megarex/scene.gltf#Animation0"), // same as idle
        death: asset_server.load("models/megarex/scene.gltf#Animation7"),
        jump: asset_server.load("models/megarex/scene.gltf#Animation8"),
    });

    for (player, x) in [
        (
            Player::Arrows,
            ARROWS_PLAYER_START_POS_X + ARROWS_PLAYER_X_OFFSET,
        ),
        (Player::Wasd, WASD_PLAYER_START_POS_X + WASD_PLAYER_X_OFFSET),
    ] {
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
    falling: Handle<AnimationClip>,
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
struct Falling;

#[derive(Component)]
struct Dying {
    start_time: f32,
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
        With<Idle>,
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
    mut material_query: Query<&mut Handle<StandardMaterial>>,
    mut block_query: Query<(&block::BlockValue, &mut block::BlockColor)>,
    time: Res<Time>,
    animations: Res<PlayerAnimations>,
    blocks: Res<block::Blocks>,
    block_materials: Res<block::BlockMaterials>,
) {
    for (entity, speed, mut transform, moving, animation_player_entity, player) in query.iter_mut()
    {
        let moving_duration = time.elapsed_seconds() - moving.start_time;
        let moving_progress = moving_duration * speed.0;

        if moving_progress >= 1.0 {
            transform.translation = moving.target;

            let x = transform.translation.x.round() as i32;
            let y = (transform.translation.y - 0.5).round() as i32;
            let mut animation_player = animation_player.get_mut(animation_player_entity.0).unwrap();
            if let Some(block_entity) = blocks.coords.get(&(x, y)) {
                commands.entity(entity).remove::<Moving>().insert(Idle);
                animation_player.play(animations.idle.clone_weak()).repeat();

                color_block(
                    &mut block_query,
                    block_entity,
                    player,
                    x,
                    y,
                    &mut material_query,
                    &block_materials,
                );
            } else {
                // TODO: step outside -> fall
                info!("{player:?} is falling!");
                commands.entity(entity).remove::<Moving>().insert(Falling);
                animation_player.play(animations.falling.clone_weak());
            }
        } else {
            let s = moving_progress.cubic_in_out();
            transform.translation = moving.source.lerp(moving.target, s);
        }
    }
}

fn color_block(
    block_query: &mut Query<(&block::BlockValue, &mut block::BlockColor)>,
    block_entity: &Entity,
    player: &Player,
    x: i32,
    y: i32,
    material_query: &mut Query<&mut Handle<StandardMaterial>>,
    block_materials: &Res<block::BlockMaterials>,
) {
    let (block_value, mut block_color) = block_query.get_mut(*block_entity).unwrap();
    info!("{player:?} -> ({x};{y}) value: {block_value:?}");
    if let Ok(mut block_material) = material_query.get_mut(*block_entity) {
        debug!("{block_material:?}");
        let (new_material, new_color) = match player {
            Player::Wasd => (
                block_materials.red[&block_value.0].clone_weak(),
                block::BlockColor::Red,
            ),
            Player::Arrows => (
                block_materials.blue[&block_value.0].clone_weak(),
                block::BlockColor::Blue,
            ),
        };
        *block_material = new_material;
        *block_color = new_color;
    }
}

fn falling(mut query: Query<&mut Transform, With<Falling>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        transform.translation.y -= 5.0 * time.delta_seconds();
    }
}

fn lava_contact(
    mut commands: Commands,
    mut animation_player: Query<&mut AnimationPlayer>,
    query: Query<(Entity, &Transform, &AnimationPlayerEntity, &Player), Without<Dying>>,
    lava: Query<&Transform, With<Lava>>,
    animations: Res<PlayerAnimations>,
    time: Res<Time>,
) {
    if let Ok(lava_transform) = lava.get_single() {
        for (player_entity, player_transform, animation_player_entity, player) in query.iter() {
            if player_transform.translation.y < lava_transform.translation.y {
                info!("{player:?} fell into lava!");
                commands
                    .entity(player_entity)
                    .remove::<(Idle, Falling, Moving)>()
                    .insert(Dying {
                        start_time: time.elapsed_seconds(),
                    });
                let mut animation_player =
                    animation_player.get_mut(animation_player_entity.0).unwrap();
                animation_player.play(animations.death.clone_weak());
            }
        }
    }
}

fn dying(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Transform,
        &AnimationPlayerEntity,
        &Dying,
        &Player,
    )>,
    mut animation_player: Query<&mut AnimationPlayer>,
    animations: Res<PlayerAnimations>,
    time: Res<Time>,
    blocks: Res<block::Blocks>,
) {
    for (entity, mut transform, animation_player_entity, dying, player) in query.iter_mut() {
        if (time.elapsed_seconds() - dying.start_time) > 2.0 {
            let mut animation_player = animation_player.get_mut(animation_player_entity.0).unwrap();
            animation_player.play(animations.idle.clone_weak()).repeat();
            commands.entity(entity).remove::<Dying>().insert(Idle);
            animation_player.play(animations.idle.clone_weak()).repeat();

            // select random block in the top-most row
            let max_y = blocks.coords.iter().map(|((_x, y), _e)| y).max().unwrap();
            let top_row: Vec<_> = blocks.coords.keys().filter(|(_x, y)| y == max_y).collect();
            let (x, y) = top_row[rand::thread_rng().gen_range(0..top_row.len())];

            // re-position player
            let x_offset = match player {
                Player::Arrows => ARROWS_PLAYER_X_OFFSET,
                Player::Wasd => WASD_PLAYER_X_OFFSET,
            };
            transform.translation.x = *x as f32 + x_offset;
            transform.translation.y = *y as f32 + 0.5;
            transform.translation.z = -(*y as f32)
        }
    }
}
