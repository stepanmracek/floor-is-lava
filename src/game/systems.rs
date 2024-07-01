use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use crate::game::components::*;
use crate::game::*;
use crate::player;

pub fn setup(
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

    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 30.0,
                ..default()
            },
        )
        .with_style(Style {
            margin: UiRect::all(Val::Px(5.)),
            ..default()
        }),
        ScoreText,
    ));
}

fn center_text(commands: &mut Commands, value: impl Into<String>, marker: impl Component) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    value,
                    TextStyle {
                        font_size: 30.0,
                        ..default()
                    },
                )
                .with_text_justify(JustifyText::Center),
                marker,
            ));
        });
}

pub fn start_entered(mut commands: Commands) {
    center_text(&mut commands, "Press any key to start", StartText);
}

pub fn start_exit(mut commands: Commands, query: Query<Entity, With<StartText>>) {
    commands.entity(query.single()).despawn();
}

pub fn pause_entered(mut commands: Commands) {
    center_text(
        &mut commands,
        "Game paused,\npress any key to resume",
        PauseText,
    );
}

pub fn pause_exit(mut commands: Commands, query: Query<Entity, With<PauseText>>) {
    commands.entity(query.single()).despawn();
}

pub fn waiting_for_start(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.get_just_pressed().count() > 0 {
        next_state.set(GameState::InGame);
    }
}

pub fn maybe_pause(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Pause) {
        next_state.set(GameState::Pause);
    }
}

pub fn raising_lava(time: Res<Time>, mut lava: Query<&mut Transform, With<Lava>>) {
    let mut lava_transform = lava.single_mut();
    lava_transform.translation.y += 0.5 * time.delta_seconds();
    lava_transform.translation.z -= 0.5 * time.delta_seconds();
}

pub fn show_score(
    mut text_query: Query<&mut Text, With<ScoreText>>,
    score_query: Query<(&player::components::Score, &player::components::Player)>,
) {
    let mut red = 0;
    let mut blue = 0;
    for (score, player) in score_query.iter() {
        match player {
            player::components::Player::Arrows => blue = score.0,
            player::components::Player::Wasd => red = score.0,
        }
    }

    if let Ok(mut text) = text_query.get_single_mut() {
        text.sections[0].value = format!("red: {red}\nblue: {blue}");
    }
}

pub fn camera_follow(
    mut camera_transform: Query<&mut Transform, With<Camera3d>>,
    players_transform: Query<&Transform, (With<player::components::Player>, Without<Camera3d>)>,
    lava_transform: Query<&Transform, (With<Lava>, Without<Camera3d>)>,
    time: Res<Time>,
) {
    if let Ok(lava_transform) = lava_transform.get_single() {
        if let Ok(mut t) = camera_transform.get_single_mut() {
            let players: Vec<_> = players_transform.iter().map(|t| t.translation).collect();
            if players_transform.is_empty() {
                return;
            }
            let center = players.iter().sum::<Vec3>() / players.len() as f32;

            let mut target = *t;
            target.translation.x = -center.x / 2.0;
            target.translation.y = 4.5 + lava_transform.translation.y;
            target.translation.z = 9.0 - lava_transform.translation.y;
            target.look_at(center, Vec3::Y);

            let s = time.delta_seconds() * 2.0;
            t.translation = t.translation.lerp(target.translation, s);
            t.rotation = t.rotation.slerp(target.rotation, s);
        }
    }
}
