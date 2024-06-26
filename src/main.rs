use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

mod block;
mod player;
mod utils;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::ORANGE_RED,
            brightness: 100.,
        })
        .add_plugins((DefaultPlugins, player::PlayersPlugin, block::BlocksPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (raising_lava, camera_follow))
        .run();
}

#[derive(Component)]
struct Lava;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // lava
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Circle::new(10.0)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("textures/lava.png")),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            transform: Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
            ..default()
        },
        Lava,
    ));

    // light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        directional_light: DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: true,
            color: Color::rgb(1.0, 0.95, 0.9),
            ..default()
        },
        ..default()
    });

    // camera
    commands.spawn(Camera3dBundle { ..default() });
}

fn raising_lava(time: Res<Time>, mut lava: Query<&mut Transform, With<Lava>>) {
    let mut lava_transform = lava.single_mut();
    lava_transform.translation.y += 0.5 * time.delta_seconds();
    lava_transform.translation.z -= 0.5 * time.delta_seconds();
}

fn camera_follow(
    mut camera_transform: Query<&mut Transform, With<Camera3d>>,
    players_transform: Query<&Transform, (With<player::Player>, Without<Camera3d>)>,
    lava_transform: Query<&Transform, (With<Lava>, Without<Camera3d>)>,
) {
    if let Ok(lava_transform) = lava_transform.get_single() {
        if let Ok(mut camera_transform) = camera_transform.get_single_mut() {
            let players_transform: Vec<_> =
                players_transform.iter().map(|t| t.translation).collect();
            if players_transform.is_empty() {
                return;
            }
            let center = players_transform.iter().sum::<Vec3>() / players_transform.len() as f32;
            camera_transform.translation.x = -center.x / 2.0;
            camera_transform.translation.y = 4.5 + lava_transform.translation.y;
            camera_transform.translation.z = 9.0 - lava_transform.translation.y;
            camera_transform.look_at(center, Vec3::Y);
        }
    }
}
