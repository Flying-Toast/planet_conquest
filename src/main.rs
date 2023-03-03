mod animation;
mod physics;
mod tiles;
use animation::AnimationFrameTimer;
use bevy::prelude::*;
use physics::{MovementSpeed, Velocity};
use tiles::{CameraFollow, PlanetLocation};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(tiles::TilingPlugin)
        .add_plugin(shader_background::BackgroundPlugin)
        .add_plugin(physics::PhysicsPlugin)
        .add_plugin(animation::AnimationPlugin)
        .add_startup_system(setup)
        .add_system(update_player_velocity)
        .run();
}

#[derive(Component)]
struct Player;

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle { ..default() });

    let texture_handle = assets.load("dude.png");
    let atlas = TextureAtlas::from_grid(texture_handle, Vec2::splat(16.), 5, 8, None, None);
    let atlas_handle = texture_atlases.add(atlas);

    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: atlas_handle,
            transform: Transform::default()
                .with_scale(Vec3::splat(4.))
                .with_translation(Vec3::new(0., 0., 10.)),
            ..default()
        })
        .insert(Player)
        .insert(CameraFollow)
        .insert(<Velocity as Default>::default())
        .insert(MovementSpeed(2.))
        .insert(AnimationFrameTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )))
        .insert(PlanetLocation::default());

    let texture_handle = assets.load("rover.png");
    let atlas = TextureAtlas::from_grid(texture_handle, Vec2::splat(16.), 1, 8, None, None);
    let atlas_handle = texture_atlases.add(atlas);
    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: atlas_handle,
            transform: Transform::default()
                .with_scale(Vec3::splat(4.))
                .with_translation(Vec3::new(0., 0., 10.)),
            ..default()
        })
        .insert(MovementSpeed(6.))
        .insert(<Velocity as Default>::default())
        .insert(PlanetLocation::default());
}

fn update_player_velocity(
    mut q: Query<(&mut Velocity, &MovementSpeed), With<Player>>,
    keyboard: Res<Input<KeyCode>>,
) {
    let (mut player_velocity, &MovementSpeed(player_speed)) = q.single_mut();
    let mut new_velocity = Vec2::ZERO;

    if keyboard.pressed(KeyCode::W) {
        new_velocity.y += 1.;
    }
    if keyboard.pressed(KeyCode::A) {
        new_velocity.x -= 1.;
    }
    if keyboard.pressed(KeyCode::S) {
        new_velocity.y -= 1.;
    }
    if keyboard.pressed(KeyCode::D) {
        new_velocity.x += 1.;
    }

    player_velocity.0 = new_velocity.normalize_or_zero() * player_speed;
}
