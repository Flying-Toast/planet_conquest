use crate::physics::Velocity;
use bevy::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(tick_animation_timers)
            .add_system(sprites_face_velocity.after(tick_animation_timers));
    }
}

/// len([North, Northeast, ...])
const NUM_CARDINAL_DIRECTIONS: usize = 8;

#[derive(Component, Deref, DerefMut)]
pub struct AnimationFrameTimer(pub Timer);

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
