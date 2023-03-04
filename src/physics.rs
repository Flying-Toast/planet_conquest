use crate::tiles::PlanetLocation;
use bevy::prelude::*;

pub struct PhysicsPlugin;

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

#[derive(Component, Deref, DerefMut)]
pub struct MovementSpeed(pub f32);

impl Default for MovementSpeed {
    fn default() -> Self {
        Self(1.)
    }
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_planet_locations_from_velocity);
    }
}

fn move_planet_locations_from_velocity(
    time: Res<Time>,
    mut q: Query<(&mut PlanetLocation, &Velocity)>,
) {
    for (mut loc, vel) in q.iter_mut() {
        loc.subtile += vel.0 * time.delta_seconds();
    }
}
