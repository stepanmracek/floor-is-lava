use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use rand::prelude::*;
use std::collections::HashMap;

use crate::Lava;

pub struct BlocksPlugin;

impl Plugin for BlocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, blocks_init);
        app.add_systems(Update, block_in_lava);
    }
}

#[derive(Resource)]
struct BlockTextures {
    gray: HashMap<u8, Handle<Image>>,
    red: HashMap<u8, Handle<Image>>,
    blue: HashMap<u8, Handle<Image>>,
}

#[derive(Component, Debug)]
enum BlockColor {
    Gray,
    Red,
    Blue,
}

#[derive(Component, Debug)]
struct BlockPosition {
    x: i32,
    y: i32,
}

#[derive(Component, Debug)]
struct BlockValue(u8);

#[derive(Bundle)]
struct BlockBundle {
    color: BlockColor,
    value: BlockValue,
    position: BlockPosition,
    pbr: PbrBundle,
}

#[derive(Resource, Default)]
struct Blocks {
    coords: HashMap<(i32, i32), Entity>,
}

fn blocks_init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut blocks = Blocks::default();
    let block_textures = BlockTextures::load(&asset_server);
    let mut rng = rand::thread_rng();
    for y in 0..=20 {
        for x in -3..=3 {
            if rng.gen::<f32>() < 0.7 || (x == 1 && y == 3) || (x == -1 && y == 3) {
                let block_val = rng.gen_range(1u8..=9u8);
                BlockBundle::spawn(
                    block_val,
                    x,
                    y,
                    &block_textures,
                    &mut meshes,
                    &mut materials,
                    &mut commands,
                    &mut blocks,
                );
            }
        }
    }
    commands.insert_resource(blocks);
}

fn block_in_lava(
    mut commands: Commands,
    mut blocks: ResMut<Blocks>,
    lava: Query<&Transform, With<Lava>>,
    blocks_query: Query<(Entity, &BlockValue, &BlockColor, &Transform, &BlockPosition)>,
) {
    if let Ok(lava) = lava.get_single() {
        for (entity, value, color, transform, position) in blocks_query.iter() {
            if transform.translation.y < lava.translation.y - 0.5 {
                debug!("Removing {color:?} cube at {transform:?} worth of {value:?}");
                blocks.coords.remove(&(position.x, position.y));
                commands.entity(entity).despawn();
            }
        }
    }
}

impl BlockTextures {
    fn load_color(asset_server: &Res<AssetServer>, color: &str) -> HashMap<u8, Handle<Image>> {
        (1..=9)
            .map(|val| {
                (
                    val,
                    asset_server.load(format!("textures/block-{val}-{color}.png")),
                )
            })
            .collect()
    }
    fn load(asset_server: &Res<AssetServer>) -> Self {
        BlockTextures {
            gray: BlockTextures::load_color(asset_server, "gray"),
            red: BlockTextures::load_color(asset_server, "red"),
            blue: BlockTextures::load_color(asset_server, "blue"),
        }
    }
}

impl BlockBundle {
    fn spawn(
        value: u8,
        x: i32,
        y: i32,
        block_textures: &BlockTextures,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        commands: &mut Commands,
        blocks: &mut Blocks,
    ) {
        let entity = commands
            .spawn(BlockBundle {
                color: BlockColor::Gray,
                value: BlockValue(value),
                position: BlockPosition { x, y },
                pbr: PbrBundle {
                    mesh: meshes.add(create_block_mesh()),
                    material: materials.add(StandardMaterial {
                        base_color_texture: Some(block_textures.gray[&value].clone()),
                        ..default()
                    }),
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
