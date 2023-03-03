mod tiles;
use bevy::prelude::*;
use tiles::{PlanetLocation, TransformLock};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(tiles::TilingPlugin)
        .add_plugin(shader_background::BackgroundPlugin)
        .add_startup_system(setup)
        .add_system(move_player)
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
        .insert(TransformLock)
        .insert(MovementSpeed(0.1))
        .insert(PlanetLocation::default());
}

fn move_player(
    mut q: Query<(&mut PlanetLocation, &MovementSpeed), With<Player>>,
    keyboard: Res<Input<KeyCode>>,
) {
    let (mut loc, &MovementSpeed(player_speed)) = q.single_mut();
    let mut vel = Vec2::ZERO;
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
    loc.subtile += vel.normalize_or_zero() * player_speed;
}
