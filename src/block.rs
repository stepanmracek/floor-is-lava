use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use rand::prelude::*;
use std::collections::HashMap;

use crate::player;
use crate::Lava;

pub struct BlocksPlugin;

impl Plugin for BlocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, blocks_init);
        app.add_systems(Update, (block_in_lava, block_generator));
    }
}

#[derive(Resource)]
struct BlockMesh(Handle<Mesh>);

#[derive(Resource)]
pub struct BlockMaterials {
    gray: HashMap<u8, Handle<StandardMaterial>>,
    pub red: HashMap<u8, Handle<StandardMaterial>>,
    pub blue: HashMap<u8, Handle<StandardMaterial>>,
}

#[derive(Component, Debug)]
pub struct BlockOwner(pub Option<Entity>);

#[derive(Component, Debug)]
struct BlockPosition {
    x: i32,
    y: i32,
}

#[derive(Component, Debug)]
pub struct BlockValue(pub u8);

#[derive(Bundle)]
struct BlockBundle {
    owner: BlockOwner,
    value: BlockValue,
    position: BlockPosition,
    pbr: PbrBundle,
}

#[derive(Resource, Default)]
pub struct Blocks {
    pub coords: HashMap<(i32, i32), Entity>,
}

fn blocks_init(
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

fn block_in_lava(
    mut commands: Commands,
    mut blocks: ResMut<Blocks>,
    mut score_query: Query<&mut player::Score>,
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
                        (*score).0 += value.0 as u32;
                        info!("{owner:?} score: {}", score.0)
                    }
                }
            }
        }
    }
}

fn block_generator(
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

impl BlockMaterials {
    fn load_color(
        asset_server: &Res<AssetServer>,
        materials: &mut Assets<StandardMaterial>,
        color: &str,
    ) -> HashMap<u8, Handle<StandardMaterial>> {
        (1..=9)
            .map(|val| {
                (
                    val,
                    materials.add(StandardMaterial {
                        base_color_texture: Some(
                            asset_server.load(format!("textures/block-{val}-{color}.png")),
                        ),
                        ..default()
                    }),
                )
            })
            .collect()
    }
    fn load(asset_server: &Res<AssetServer>, materials: &mut Assets<StandardMaterial>) -> Self {
        BlockMaterials {
            gray: BlockMaterials::load_color(asset_server, materials, "gray"),
            red: BlockMaterials::load_color(asset_server, materials, "red"),
            blue: BlockMaterials::load_color(asset_server, materials, "blue"),
        }
    }
}

impl BlockBundle {
    fn spawn(
        value: u8,
        x: i32,
        y: i32,
        block_materials: &BlockMaterials,
        mesh: Handle<Mesh>,
        commands: &mut Commands,
        blocks: &mut Blocks,
    ) {
        let material = block_materials.gray[&value].clone_weak();
        let entity = commands
            .spawn(BlockBundle {
                owner: BlockOwner(None),
                value: BlockValue(value),
                position: BlockPosition { x, y },
                pbr: PbrBundle {
                    mesh,
                    material,
                    transform: Transform::from_xyz(x as f32, y as f32, -y as f32),
                    ..default()
                },
            })
            .id();
        blocks.coords.insert((x, y), entity);
    }
}

fn create_block_mesh() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            // top (facing towards +y)
            [-0.4, 0.5, -0.4],
            [0.4, 0.5, -0.4],
            [0.4, 0.5, 0.4],
            [-0.4, 0.5, 0.4],
            // bottom   (-y)
            [-0.4, -0.3, -0.4],
            [0.4, -0.3, -0.4],
            [0.4, -0.3, 0.4],
            [-0.4, -0.3, 0.4],
            // right    (+x)
            [0.4, -0.3, -0.4],
            [0.4, -0.3, 0.4],
            [0.4, 0.5, 0.4],
            [0.4, 0.5, -0.4],
            // left     (-x)
            [-0.4, -0.3, -0.4],
            [-0.4, -0.3, 0.4],
            [-0.4, 0.5, 0.4],
            [-0.4, 0.5, -0.4],
            // back     (+z)
            [-0.4, -0.3, 0.4],
            [-0.4, 0.5, 0.4],
            [0.4, 0.5, 0.4],
            [0.4, -0.3, 0.4],
            // forward  (-z)
            [-0.4, -0.3, -0.4],
            [-0.4, 0.5, -0.4],
            [0.4, 0.5, -0.4],
            [0.4, -0.3, -0.4],
        ],
    )
    // Set-up UV coordinates to point to the upper (V < 0.5), "dirt+grass" part of the texture.
    // Take a look at the custom image (assets/textures/array_texture.png)
    // so the UV coords will make more sense
    // Note: (0.0, 0.0) = Top-Left in UV mapping, (1.0, 1.0) = Bottom-Right in UV mapping
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![
            // Assigning the UV coords for the top side.
            [0.0, 0.5],
            [1.0, 0.5],
            [1.0, 1.0],
            [0.0, 1.0],
            // Assigning the UV coords for the bottom side.
            [0.0, 0.5],
            [1.0, 0.5],
            [1.0, 1.0],
            [0.0, 1.0],
            // Assigning the UV coords for the right side.
            [0.0, 0.5],
            [1.0, 0.5],
            [1.0, 1.0],
            [0.0, 1.0],
            // Assigning the UV coords for the left side.
            [0.0, 0.5],
            [1.0, 0.5],
            [1.0, 1.0],
            [0.0, 1.0],
            // Assigning the UV coords for the back side.
            [0.0, 0.5],
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 0.5],
            // Assigning the UV coords for the forward side.
            [0.0, 0.5],
            [1.0, 0.5],
            [1.0, 1.0],
            [0.0, 1.0],
        ],
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![
            // Normals for the top side (towards +y)
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            // Normals for the bottom side (towards -y)
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            // Normals for the right side (towards +x)
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            // Normals for the left side (towards -x)
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            // Normals for the back side (towards +z)
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            // Normals for the forward side (towards -z)
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
        ],
    )
    .with_inserted_indices(Indices::U32(vec![
        0, 3, 1, 1, 3, 2, // triangles making up the top (+y) facing side.
        4, 5, 7, 5, 6, 7, // bottom (-y)
        8, 11, 9, 9, 11, 10, // right (+x)
        12, 13, 15, 13, 14, 15, // left (-x)
        16, 19, 17, 17, 19, 18, // back (+z)
        20, 21, 23, 21, 22, 23, // forward (-z)
    ]))
}
