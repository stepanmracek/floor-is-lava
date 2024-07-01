use std::collections::HashMap;

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};

#[derive(Resource)]
pub struct BlockMesh(pub Handle<Mesh>);

#[derive(Resource)]
pub struct BlockMaterials {
    pub gray: HashMap<u8, Handle<StandardMaterial>>,
    pub red: HashMap<u8, Handle<StandardMaterial>>,
    pub blue: HashMap<u8, Handle<StandardMaterial>>,
}

#[derive(Resource, Default)]
pub struct Blocks {
    pub coords: HashMap<(i32, i32), Entity>,
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

    pub fn load(asset_server: &Res<AssetServer>, materials: &mut Assets<StandardMaterial>) -> Self {
        BlockMaterials {
            gray: BlockMaterials::load_color(asset_server, materials, "gray"),
            red: BlockMaterials::load_color(asset_server, materials, "red"),
            blue: BlockMaterials::load_color(asset_server, materials, "blue"),
        }
    }
}

pub fn create_block_mesh() -> Mesh {
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
