use bevy::prelude::*;

use crate::block::resources::*;

#[derive(Component, Debug)]
pub struct BlockOwner(pub Option<Entity>);

#[derive(Component, Debug)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Debug)]
pub struct BlockValue(pub u8);

#[derive(Bundle)]
pub struct BlockBundle {
    pub owner: BlockOwner,
    pub value: BlockValue,
    pub position: BlockPosition,
    pub pbr: PbrBundle,
}

impl BlockBundle {
    pub fn spawn(
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
