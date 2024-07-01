use interpolation::Ease;
use rand::Rng;

use crate::block;
use crate::game;
use crate::player::components::*;
use crate::player::resources::*;
use crate::player::*;
use crate::utils;

pub fn players_init(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(PlayerAnimations {
        idle: asset_server.load("models/megarex/scene.gltf#Animation0"),
        run: asset_server.load("models/megarex/scene.gltf#Animation1"),
        win: asset_server.load("models/megarex/scene.gltf#Animation3"),
        falling: asset_server.load("models/megarex/scene.gltf#Animation0"), // same as idle
        death: asset_server.load("models/megarex/scene.gltf#Animation7"),
        jump: asset_server.load("models/megarex/scene.gltf#Animation8"),
    });

    for (player, x) in [
        (Player::Blue, BLUE_PLAYER_START_POS_X + BLUE_PLAYER_X_OFFSET),
        (Player::Red, RED_PLAYER_START_POS_X + RED_PLAYER_X_OFFSET),
    ] {
        let scene = match player {
            Player::Blue => asset_server.load("models/megarex/blue.gltf#Scene0"),
            Player::Red => asset_server.load("models/megarex/red.gltf#Scene0"),
        };
        info!("Scene {}", scene.id());

        let ai = player == Player::Red;
        let mut entity_commands = commands.spawn((
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
                score: Score(0),
            },
            Idle,
        ));
        if ai {
            entity_commands.insert(AI);
        }

        let player_id = entity_commands.id();
        info!("Spawned player {:?}", player_id);
    }
}

pub fn idle_init(
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

#[derive(Clone)]
enum Direction {
    Right,
    Left,
    Up,
    Down,
}

fn player_translation_to_position(translation: &Vec3) -> (i32, i32) {
    let x = translation.x.round() as i32;
    let y = (translation.y - 0.5).round() as i32;
    (x, y)
}

fn get_ai_direction(
    x: i32,
    y: i32,
    player_entity: Entity,
    blocks: &block::resources::Blocks,
    owner_query: &Query<&block::components::BlockOwner>,
) -> Option<Direction> {
    let deltas = [
        (1, 0, Direction::Right),
        (-1, 0, Direction::Left),
        (0, 1, Direction::Up),
        (0, -1, Direction::Down),
    ];

    let mut possible_directions_enemy = vec![];
    let mut possible_directions_own = vec![];
    let mut possible_directions_empty = vec![];
    for (dx, dy, direction) in deltas.iter() {
        if let Some(&block_entity) = blocks.coords.get(&(x + dx, y + dy)) {
            if let Ok(block_owner_component) = owner_query.get(block_entity) {
                let owner = block_owner_component.0;
                if let Some(owner_entity) = owner {
                    if owner_entity != player_entity {
                        possible_directions_enemy.push(direction);
                    } else {
                        possible_directions_own.push(direction);
                    }
                } else {
                    possible_directions_empty.push(direction);
                }
            }
        }
    }

    for possible_directions in [
        possible_directions_enemy,
        possible_directions_empty,
        possible_directions_own,
    ] {
        if !possible_directions.is_empty() {
            let random_index = rand::thread_rng().gen_range(0..possible_directions.len());
            return Some(possible_directions[random_index].clone());
        }
    }

    None
}

pub fn ai_control(
    mut commands: Commands,
    mut ai_player_query: Query<
        (Entity, &mut Transform, &AnimationPlayerEntity, &Speed),
        (With<Idle>, With<AI>),
    >,
    mut animation_player: Query<&mut AnimationPlayer>,
    owner_query: Query<&block::components::BlockOwner>,
    time: Res<Time>,
    blocks: Res<block::resources::Blocks>,
    animations: Res<PlayerAnimations>,
) {
    for (entity, transform, animation_player_entity, speed) in ai_player_query.iter_mut() {
        let (x, y) = player_translation_to_position(&transform.translation);
        let direction = get_ai_direction(x, y, entity, &blocks, &owner_query);
        if let Some(direction) = direction {
            apply_move(
                &direction,
                transform,
                &mut commands,
                entity,
                &time,
                &mut animation_player,
                animation_player_entity,
                &animations,
                speed,
            );
        }
    }
}

pub fn key_control(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &Player,
            &AnimationPlayerEntity,
            &Speed,
        ),
        (With<Idle>, Without<AI>),
    >,
    mut animation_player: Query<&mut AnimationPlayer>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    animations: Res<PlayerAnimations>,
) {
    for (entity, transform, player, animation_player_entity, speed) in query.iter_mut() {
        let direction = match player {
            Player::Blue => {
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
            Player::Red => {
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

        if let Some(direction) = direction {
            apply_move(
                &direction,
                transform,
                &mut commands,
                entity,
                &time,
                &mut animation_player,
                animation_player_entity,
                &animations,
                speed,
            );
        }
    }
}

fn apply_move(
    direction: &Direction,
    mut transform: Mut<Transform>,
    commands: &mut Commands,
    entity: Entity,
    time: &Res<Time>,
    animation_player: &mut Query<&mut AnimationPlayer, ()>,
    animation_player_entity: &AnimationPlayerEntity,
    animations: &Res<PlayerAnimations>,
    speed: &Speed,
) {
    match direction {
        Direction::Right => {
            transform.look_to(-Vec3::X, Vec3::Y);
            commands.entity(entity).remove::<Idle>().insert(Moving {
                source: transform.translation,
                target: transform.translation + Vec3::X,
                start_time: time.elapsed_seconds(),
            });
        }
        Direction::Left => {
            transform.look_to(Vec3::X, Vec3::Y);
            commands.entity(entity).remove::<Idle>().insert(Moving {
                source: transform.translation,
                target: transform.translation - Vec3::X,
                start_time: time.elapsed_seconds(),
            });
        }
        Direction::Up => {
            transform.look_to(Vec3::Z, Vec3::Y);
            commands.entity(entity).remove::<Idle>().insert(Moving {
                source: transform.translation,
                target: transform.translation + Vec3::Y - Vec3::Z,
                start_time: time.elapsed_seconds(),
            });
        }
        Direction::Down => {
            transform.look_to(-Vec3::Z, Vec3::Y);
            commands.entity(entity).remove::<Idle>().insert(Moving {
                source: transform.translation,
                target: transform.translation - Vec3::Y + Vec3::Z,
                start_time: time.elapsed_seconds(),
            });
        }
    };
    let mut animation_player = animation_player.get_mut(animation_player_entity.0).unwrap();
    animation_player.play(animations.jump.clone_weak());
    animation_player.set_speed(speed.0);
}

pub fn moving(
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
    mut owner_query: Query<(
        &block::components::BlockValue,
        &mut block::components::BlockOwner,
    )>,
    time: Res<Time>,
    animations: Res<PlayerAnimations>,
    blocks: Res<block::resources::Blocks>,
    block_materials: Res<block::resources::BlockMaterials>,
) {
    for (player_entity, speed, mut transform, moving, animation_player_entity, player) in
        query.iter_mut()
    {
        let moving_duration = time.elapsed_seconds() - moving.start_time;
        let moving_progress = moving_duration * speed.0;

        if moving_progress >= 1.0 {
            transform.translation = moving.target;

            let (x, y) = player_translation_to_position(&transform.translation);
            let mut animation_player = animation_player.get_mut(animation_player_entity.0).unwrap();
            if let Some(block_entity) = blocks.coords.get(&(x, y)) {
                commands
                    .entity(player_entity)
                    .remove::<Moving>()
                    .insert(Idle);
                animation_player.play(animations.idle.clone_weak()).repeat();

                color_block(
                    &mut owner_query,
                    block_entity,
                    &player_entity,
                    player,
                    x,
                    y,
                    &mut material_query,
                    &block_materials,
                );
            } else {
                info!("{player:?} is falling!");
                commands
                    .entity(player_entity)
                    .remove::<Moving>()
                    .insert(Falling);
                animation_player.play(animations.falling.clone_weak());
            }
        } else {
            let s = moving_progress.cubic_in_out();
            transform.translation = moving.source.lerp(moving.target, s);
        }
    }
}

pub fn color_block(
    owner_query: &mut Query<(
        &block::components::BlockValue,
        &mut block::components::BlockOwner,
    )>,
    block_entity: &Entity,
    player_entity: &Entity,
    player: &Player,
    x: i32,
    y: i32,
    material_query: &mut Query<&mut Handle<StandardMaterial>>,
    block_materials: &Res<block::resources::BlockMaterials>,
) {
    let (block_value, mut block_owner) = owner_query.get_mut(*block_entity).unwrap();
    info!("{player_entity:?} -> ({x};{y}) value: {block_value:?}");
    if let Ok(mut block_material) = material_query.get_mut(*block_entity) {
        debug!("{block_material:?}");
        let new_material = match player {
            Player::Red => block_materials.red[&block_value.0].clone_weak(),
            Player::Blue => block_materials.blue[&block_value.0].clone_weak(),
        };
        *block_material = new_material;
        *block_owner = block::components::BlockOwner(Some(*player_entity));
    }
}

pub fn falling(mut query: Query<&mut Transform, With<Falling>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        transform.translation.y -= 5.0 * time.delta_seconds();
    }
}

pub fn lava_contact(
    mut commands: Commands,
    mut animation_player: Query<&mut AnimationPlayer>,
    query: Query<(Entity, &Transform, &AnimationPlayerEntity, &Player), Without<Dying>>,
    lava: Query<&Transform, With<game::components::Lava>>,
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

pub fn dying(
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
    blocks: Res<block::resources::Blocks>,
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
                Player::Blue => BLUE_PLAYER_X_OFFSET,
                Player::Red => RED_PLAYER_X_OFFSET,
            };
            transform.translation.x = *x as f32 + x_offset;
            transform.translation.y = *y as f32 + 0.5;
            transform.translation.z = -(*y as f32)
        }
    }
}
