mod tiles;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(tiles::TilingPlugin)
        .add_plugin(shader_background::BackgroundPlugin)
        .add_startup_system(setup)
        .add_system(move_player.at_start())
        .add_system(center_camera.at_start().after(move_player))
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component, Deref, DerefMut)]
struct MovementSpeed(f32);

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(Camera2dBundle { ..default() });
    commands
        .spawn(SpriteBundle {
            texture: assets.load("dude.png"),
            transform: Transform::default().with_scale(Vec3::splat(0.6)),
            ..default()
        })
        .insert(Player)
        .insert(MovementSpeed(6.0));
}

fn move_player(
    mut q: Query<(&mut Transform, &MovementSpeed), With<Player>>,
    map: Query<&tiles::PlanetMap>,
    keyboard: Res<Input<KeyCode>>,
) {
    let (mut player_transform, &MovementSpeed(player_speed)) = q.single_mut();
    let mut vel = Vec3::ZERO;
    if keyboard.pressed(KeyCode::W) {
        vel.y += 1.;
    }
    if keyboard.pressed(KeyCode::A) {
        vel.x -= 1.;
    }
    if keyboard.pressed(KeyCode::S) {
        vel.y -= 1.;
    }
    if keyboard.pressed(KeyCode::D) {
        vel.x += 1.;
    }
    let map = map.single();
    player_transform.translation += vel.normalize_or_zero() * player_speed;
    player_transform.translation = player_transform
        .translation
        .xy()
        .clamp(
            Vec2::ZERO,
            Vec2::splat(tiles::TILE_SIZE * map.size() as f32),
        )
        .extend(player_transform.translation.z);
}

/// Centers the camera on the player
fn center_camera(
    center_to: Query<&Transform, (With<Player>, Without<Camera>)>,
    mut camera_transform: Query<&mut Transform, With<Camera>>,
) {
    camera_transform.single_mut().translation = center_to.single().translation;
}
