mod physics;
mod tiles;
use bevy::prelude::*;
use physics::{MovementSpeed, Velocity};
use tiles::{PlanetLocation, TransformLock};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(tiles::TilingPlugin)
        .add_plugin(shader_background::BackgroundPlugin)
        .add_plugin(physics::PhysicsPlugin)
        .add_startup_system(setup)
        .add_system(update_player_velocity)
        .add_system(tick_animation_timers)
        .add_system(sprites_face_velocity.after(tick_animation_timers))
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component, Deref, DerefMut)]
struct AnimationFrameTimer(Timer);

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
        .insert(TransformLock)
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
            transform: Transform::default().with_scale(Vec3::splat(4.)).with_translation(Vec3::new(0., 0., 10.)),
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

/// len([North, Northeast, ...])
const NUM_CARDINAL_DIRECTIONS: usize = 8;

fn tick_animation_timers(
    time: Res<Time>,
    mut q: Query<(
        &mut AnimationFrameTimer,
        &Handle<TextureAtlas>,
        &mut TextureAtlasSprite,
    )>,
    atlases: Res<Assets<TextureAtlas>>,
) {
    for (mut frame_timer, atlas_handle, mut sprite) in q.iter_mut() {
        if frame_timer.tick(time.delta()).just_finished() {
            let total_frames_per_row =
                atlases.get(atlas_handle).unwrap().textures.len() / NUM_CARDINAL_DIRECTIONS;
            assert!(total_frames_per_row > 1, "Non-animated (i.e. directional only) sprites shouldn't have an AnimationFrametimer component");
            let walking_frames_per_row = total_frames_per_row - 1;
            let current_col = sprite.index % total_frames_per_row;
            let current_row = sprite.index / total_frames_per_row;

            sprite.index = 1
                + (current_row * total_frames_per_row)
                + ((current_col + 1) % walking_frames_per_row);
        }
    }
}

/// Orients sprites with spritesheets so that they face their velocity's cardinal direction
fn sprites_face_velocity(
    mut q: Query<(&Velocity, &Handle<TextureAtlas>, &mut TextureAtlasSprite)>,
    atlases: Res<Assets<TextureAtlas>>,
) {
    for (&Velocity(vel), atlas_handle, mut sprite) in q.iter_mut() {
        let num_sprites = atlases.get(atlas_handle).unwrap().textures.len();
        let total_frames_per_row = num_sprites / NUM_CARDINAL_DIRECTIONS;
        let num_rows = num_sprites / total_frames_per_row;
        let current_row = sprite.index / total_frames_per_row;

        let mut target_row;
        if vel == Vec2::ZERO {
            sprite.index = current_row * total_frames_per_row;
            continue;
        } else if vel.y > 0. {
            if vel.x == 0. || vel.x == -0. {
                target_row = 6;
            } else if vel.x < 0. {
                target_row = 7;
            } else {
                target_row = 5;
            }
        } else if vel.y < 0. {
            if vel.x == 0. || vel.x == -0. {
                target_row = 2;
            } else if vel.x < 0. {
                target_row = 1;
            } else {
                target_row = 3;
            }
        } else if vel.x < 0. {
            target_row = 0;
        } else {
            target_row = 4;
        }
        target_row %= num_rows as i32;

        if current_row != target_row as usize {
            if total_frames_per_row == 1 {
                sprite.index = target_row as usize;
            } else {
                sprite.index = target_row as usize * total_frames_per_row + 1;
            }
        }
    }
}
