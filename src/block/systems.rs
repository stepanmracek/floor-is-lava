use bevy::prelude::*;
use rand::Rng;

use crate::block::components::*;
use crate::block::resources::*;
use crate::player;
use crate::Lava;

pub fn blocks_init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mesh_handle = meshes.add(create_block_mesh());
    let mut blocks = Blocks::default();
    let block_textures = BlockMaterials::load(&asset_server, &mut materials);
    let mut rng = rand::thread_rng();
    let player_start_y = player::PLAYER_START_Y as i32;
    for y in 0..=3 {
        for x in -3..=3 {
            if rng.gen::<f32>() < 0.7
                || (x == player::ARROWS_PLAYER_START_POS_X as i32 && y == player_start_y)
                || (x == player::WASD_PLAYER_START_POS_X as i32 && y == player_start_y)
            {
                let block_val = rng.gen_range(1u8..=9u8);
                BlockBundle::spawn(
                    block_val,
                    x,
                    y,
                    &block_textures,
                    mesh_handle.clone_weak(),
                    &mut commands,
                    &mut blocks,
                );
            }
        }
    }

    commands.insert_resource(block_textures);
    commands.insert_resource(BlockMesh(mesh_handle));
    commands.insert_resource(blocks);
}

pub fn block_in_lava(
    mut commands: Commands,
    mut blocks: ResMut<Blocks>,
    mut score_query: Query<&mut player::components::Score>,
    lava: Query<&Transform, With<Lava>>,
    blocks_query: Query<(Entity, &BlockValue, &BlockOwner, &Transform, &BlockPosition)>,
) {
    if let Ok(lava) = lava.get_single() {
        for (entity, value, owner, transform, position) in blocks_query.iter() {
            if transform.translation.y < lava.translation.y - 0.5 {
                blocks.coords.remove(&(position.x, position.y));
                commands.entity(entity).despawn();
                if let Some(owner) = owner.0 {
                    debug!(
                        "Removing cube at {transform:?} belonging to {owner:?} worth of {value:?}"
                    );
                    if let Ok(mut score) = score_query.get_mut(owner) {
                        score.0 += value.0 as u32;
                        info!("{owner:?} score: {}", score.0)
                    }
                }
            }
        }
    }
}

pub fn block_generator(
    mut commands: Commands,
    mut blocks: ResMut<Blocks>,
    mesh: Res<BlockMesh>,
    block_materials: Res<BlockMaterials>,
    lava: Query<&Transform, With<Lava>>,
) {
    if let Ok(lava) = lava.get_single() {
        let max_y = blocks.coords.iter().map(|((_x, y), _ent)| y).max();
        if let Some(&max_y) = max_y {
            if ((max_y - 5) as f32) < lava.translation.y {
                let mut rng = rand::thread_rng();
                for x in -3..=3 {
                    if rng.gen::<f32>() < 0.7 {
                        let block_val = rng.gen_range(1u8..=9u8);
                        BlockBundle::spawn(
                            block_val,
                            x,
                            max_y + 1,
                            &block_materials,
                            mesh.0.clone_weak(),
                            &mut commands,
                            &mut blocks,
                        );
                    }
                }
            }
        }
    }
}
