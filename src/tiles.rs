use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use std::collections::HashMap;

pub struct TilingPlugin;

impl Plugin for TilingPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_earth)
            .add_system(fill_location_table.at_start())
            .add_system(load_unload_tiles)
            .add_system(retransform_tiles.after(load_unload_tiles))
            .add_system(update_planet_locations)
            .add_system(propagate_planet_location_to_transform.after(update_planet_locations));
    }
}

/// Width/height of a map tile.
pub const TILE_SIZE: f32 = 64.;
/// Width/height of the entire planet's map, in tiles.
const MAP_SIZE: usize = 100;

#[derive(Copy, Clone, Debug)]
enum TileKind {
    Grass,
    WorldBorder,
}

impl TileKind {
    fn get_texture(&self, assets: &AssetServer) -> Handle<Image> {
        match self {
            Self::Grass => assets.load("grass.png"),
            Self::WorldBorder => assets.load("world_border.png"),
        }
    }
}

/// A location of something, within a tile of a planet
#[derive(Component, Debug)]
pub struct PlanetLocation {
    pub tile: (usize, usize),
    pub subtile: Vec2,
}

impl PlanetLocation {
    fn to_full_location(&self) -> Vec2 {
        (self.subtile + Vec2::new(self.tile.0 as f32, self.tile.1 as f32)) * TILE_SIZE
    }
}

impl Default for PlanetLocation {
    fn default() -> Self {
        Self {
            tile: (0, 0),
            subtile: Vec2::ZERO,
        }
    }
}

/// The entity with this component will have its transform stay centered in the screen.
/// Only one entity can have this component at a time.
/// It needs to also have a PlanetLocation.
#[derive(Component)]
pub struct CameraFollow;

/// Tiles of a planet
#[derive(Component)]
struct PlanetMap {
    tiles: [[(TileKind, Option<Entity>); MAP_SIZE]; MAP_SIZE],
}

impl PlanetMap {
    fn size(&self) -> usize {
        MAP_SIZE
    }

    /// Put `None` in the entity id spot of the given tile
    fn delete_tile_ent_at(&mut self, x: usize, y: usize) {
        self.tiles[y][x].1 = None;
    }

    fn tile_has_entity(&self, x: usize, y: usize) -> bool {
        self.tiles[y][x].1.is_some()
    }

    fn put_tile_entity(&mut self, x: usize, y: usize, ent: Entity) {
        assert!(self.tiles[y][x].1.is_none());
        self.tiles[y][x].1 = Some(ent);
    }

    fn new_earth() -> Self {
        let mut tiles = [[(TileKind::Grass, None); MAP_SIZE]; MAP_SIZE];
        tiles[0] = [(TileKind::WorldBorder, None); MAP_SIZE];
        tiles[tiles.len() - 1] = [(TileKind::WorldBorder, None); MAP_SIZE];
        for mut row in tiles.iter_mut() {
            row[0].0 = TileKind::WorldBorder;
            row[row.len() - 1].0 = TileKind::WorldBorder;
        }

        Self { tiles }
    }
}

/// Marker component for map tile sprites
#[derive(Component)]
struct MapTile(usize, usize);

/// An efficient way to get all the entities whose PlanetLocation is in a given tile.
#[derive(Component)]
pub struct LocationTable {
    map: HashMap<(usize, usize), Vec<Entity>>,
}

impl LocationTable {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn ents_in_tile(&self, tile_loc: (usize, usize)) -> &[Entity] {
        self.map.get(&tile_loc).map(Vec::as_slice).unwrap_or(&[])
    }
}

fn fill_location_table(
    mut table: Query<&mut LocationTable>,
    ents: Query<(Entity, &PlanetLocation)>,
) {
    let table = &mut table.single_mut().map;
    table.clear();

    for (entity, loc) in ents.iter() {
        table.entry(loc.tile).or_insert(Vec::new()).push(entity);
    }
}

/// Spawns sprites for tiles within render distance, and deletes offscreen sprites
fn load_unload_tiles(
    camera_query: Query<&Camera>,
    existing_tiles: Query<(Entity, &Transform, &MapTile)>,
    mut map: Query<&mut PlanetMap>,
    assets: Res<AssetServer>,
    mut commands: Commands,
    camera_origin: Query<&PlanetLocation, With<CameraFollow>>,
) {
    let origin_loc = camera_origin.single().to_full_location();
    let camera = camera_query.single();
    let viewport_size = camera.logical_viewport_size().unwrap();
    let camera_rect = Rect::from_center_size(Vec2::ZERO, viewport_size);

    let mut map = map.single_mut();
    // despawn offscreen tiles
    for (ent, tile_transform, map_tile) in existing_tiles.iter() {
        let tile_rect =
            Rect::from_center_size(tile_transform.translation.xy(), Vec2::splat(TILE_SIZE));

        if tile_rect.intersect(camera_rect).is_empty() {
            map.delete_tile_ent_at(map_tile.0, map_tile.1);
            commands.entity(ent).despawn();
        }
    }

    // spawn new tiles
    let min_onscreen = ((camera_rect.min + origin_loc) / TILE_SIZE).ceil();
    let max_onscreen = ((camera_rect.max + origin_loc) / TILE_SIZE).ceil();

    for y in f32::max(min_onscreen.y - 1., 0.) as usize..=max_onscreen.y as usize {
        if y >= MAP_SIZE {
            break;
        }
        for x in f32::max(min_onscreen.x - 1., 0.) as usize..=max_onscreen.x as usize {
            if x >= MAP_SIZE {
                break;
            }
            if !map.tile_has_entity(x, y) {
                let camera_relative_translation =
                    get_transform_for_tile(&MapTile(x, y), origin_loc);
                let tile_ent = commands
                    .spawn(SpriteBundle {
                        texture: map.tiles[y][x].0.get_texture(&*assets),
                        transform: Transform::default()
                            .with_translation(camera_relative_translation.extend(0.))
                            .with_scale(Vec3::splat(4.)),
                        ..default()
                    })
                    .insert(MapTile(x, y))
                    .id();
                map.put_tile_entity(x, y, tile_ent);
            }
        }
    }
}

fn get_transform_for_tile(mt: &MapTile, player_origin: Vec2) -> Vec2 {
    let tile_location = Vec2::new(mt.0 as f32, mt.1 as f32) * TILE_SIZE;

    tile_location - player_origin
}

fn retransform_tiles(
    mut tiles: Query<(&mut Transform, &MapTile)>,
    camera_origin: Query<&PlanetLocation, With<CameraFollow>>,
) {
    let camera_origin = camera_origin.single().to_full_location();

    for (mut transform, mt) in tiles.iter_mut() {
        transform.translation = get_transform_for_tile(mt, camera_origin).extend(0.);
    }
}

fn spawn_earth(mut commands: Commands) {
    commands
        .spawn(PlanetMap::new_earth())
        .insert(LocationTable::new());
}

fn update_planet_locations(mut locations: Query<&mut PlanetLocation>, map: Query<&PlanetMap>) {
    let map = map.single();

    for mut pl in locations.iter_mut() {
        let new_subtile = pl.subtile.abs().fract() * pl.subtile.signum();
        let dtile = (pl.subtile.abs() - new_subtile.abs()) * pl.subtile.signum();

        if dtile.x < 0. {
            pl.tile.0 = pl.tile.0.saturating_sub(dtile.x.abs() as usize);
        } else {
            pl.tile.0 += dtile.x.abs() as usize;
        }

        if dtile.y < 0. {
            pl.tile.1 = pl.tile.1.saturating_sub(dtile.y.abs() as usize);
        } else {
            pl.tile.1 += dtile.y.abs() as usize;
        }

        pl.subtile = new_subtile;

        if pl.tile.0 == 0 {
            pl.subtile.x = pl.subtile.x.max(0.);
        } else if pl.to_full_location().x >= (map.size() - 1) as f32 * TILE_SIZE {
            pl.tile.0 = map.size() - 1;
            pl.subtile.x = 0.;
        }

        if pl.tile.1 == 0 {
            pl.subtile.y = pl.subtile.y.max(0.);
        } else if pl.to_full_location().y >= (map.size() - 1) as f32 * TILE_SIZE {
            pl.tile.1 = map.size() - 1;
            pl.subtile.y = 0.;
        }
    }
}

fn propagate_planet_location_to_transform(
    mut q: Query<(&mut Transform, &PlanetLocation), Without<CameraFollow>>,
    camera_origin: Query<&PlanetLocation, With<CameraFollow>>,
) {
    for (mut transform, loc) in q.iter_mut() {
        let transform_xy = loc.to_full_location() - camera_origin.single().to_full_location();
        transform.translation = transform_xy.extend(transform.translation.z);
    }
}
