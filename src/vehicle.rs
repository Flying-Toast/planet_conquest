use crate::{
    physics::{MovementSpeed, Velocity},
    tiles::{LocationTable, PlanetLocation},
    CameraFollow, Controllable, Player,
};
use bevy::prelude::*;

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(enter_or_exit_vehicles)
            .add_system(move_player_with_vehicle);
    }
}

#[derive(Bundle, Default)]
pub struct VehicleBundle {
    pub sprite_sheet: SpriteSheetBundle,
    pub movement_speed: MovementSpeed,
    pub velocity: Velocity,
    pub vehicle: Vehicle,
    pub location: PlanetLocation,
}

/// Marks an entity as being driveable
#[derive(Component, Default)]
pub struct Vehicle;

fn enter_or_exit_vehicles(
    player_query: Query<(Entity, &PlanetLocation, Option<&Controllable>), With<Player>>,
    controllable_vehicle_query: Query<Entity, (With<Controllable>, With<Vehicle>, Without<Player>)>,
    keyboard: Res<Input<KeyCode>>,
    location_table: Query<&LocationTable>,
    mut vehicle_query: Query<&PlanetLocation, With<Vehicle>>,
    mut commands: Commands,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    let location_table = location_table.single();
    let (player_entity, player_location, player_controllable) = player_query.single();

    if player_controllable.is_some() {
        // Player is the current Controllable - not in a vehicle
        for vehicle_ent in location_table.ents_in_tile_and_neighbors(player_location.tile, 2) {
            if let Ok(vehicle_loc) = vehicle_query.get_mut(vehicle_ent) {
                commands
                    .entity(player_entity)
                    .remove::<CameraFollow>()
                    .insert(Visibility::INVISIBLE)
                    .insert(vehicle_loc.clone())
                    .remove::<Controllable>();
                commands
                    .entity(vehicle_ent)
                    .insert(CameraFollow)
                    .insert(Controllable);
                break;
            }
        }
    } else {
        // Player is not Controllable - already in a vehicle
        commands
            .entity(player_entity)
            .insert(Visibility::VISIBLE)
            .insert(Controllable)
            .insert(CameraFollow);
        commands
            .entity(controllable_vehicle_query.single())
            .remove::<CameraFollow>()
            .remove::<Controllable>()
            .insert(<Velocity as Default>::default());
    }
}

fn move_player_with_vehicle(
    mut player_query: Query<&mut PlanetLocation, (With<Player>, Without<Controllable>)>,
    vehicle_query: Query<&PlanetLocation, (With<Controllable>, With<Vehicle>)>,
) {
    if let (Ok(mut player_loc), Ok(vehicle_loc)) =
        (player_query.get_single_mut(), vehicle_query.get_single())
    {
        *player_loc = vehicle_loc.clone();
    }
}
