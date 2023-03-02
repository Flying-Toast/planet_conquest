use bevy::math::Vec3Swizzles;
use bevy::prelude::*;

pub struct TilingPlugin;

impl Plugin for TilingPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_earth)
            .add_system(load_unload_tiles.at_start());
    }
}

/// Width/height of a map tile.
pub const TILE_SIZE: f32 = 64.;
/// Width/height of the entire planet's map, in tiles.
const MAP_SIZE: usize = 10;

#[derive(Copy, Clone)]
enum TileKind {
    Grass,
}

impl TileKind {
    fn get_texture(&self, assets: &AssetServer) -> Handle<Image> {
        match self {
            Self::Grass => assets.load("grass.png"),
        }
    }
}

/// Tiles of a planet
#[derive(Component)]
pub struct PlanetMap {
    tiles: [[(TileKind, Option<Entity>); MAP_SIZE]; MAP_SIZE],
}

impl PlanetMap {
    pub fn size(&self) -> usize {
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
        Self {
            tiles: [[(TileKind::Grass, None); MAP_SIZE]; MAP_SIZE],
        }
    }
}

/// Marker component for map tile sprites
#[derive(Component)]
struct MapTile(usize, usize);

/// Spawns sprites for tiles within render distance, and deletes offscreen sprites
fn load_unload_tiles(
    camera_query: Query<(&Transform, &Camera)>,
    existing_tiles: Query<(Entity, &Transform, &MapTile)>,
    mut map: Query<&mut PlanetMap>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    let (cam_transform, camera) = camera_query.single();
    let viewport_size = camera.logical_viewport_size().unwrap();
    let camera_rect = Rect::from_center_size(cam_transform.translation.xy(), viewport_size);

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
    let min_onscreen = (camera_rect.min / TILE_SIZE).ceil();
    let max_onscreen = (camera_rect.max / TILE_SIZE).ceil();

    for y in f32::max(min_onscreen.y - 1., 0.) as usize..=max_onscreen.y as usize {
        if y >= MAP_SIZE {
            break;
        }
        for x in f32::max(min_onscreen.x - 1., 0.) as usize..=max_onscreen.x as usize {
            if x >= MAP_SIZE {
                break;
            }
            if !map.tile_has_entity(x, y) {
                let tile_ent = commands
                    .spawn(SpriteBundle {
                        texture: map.tiles[y][x].0.get_texture(&*assets),
                        transform: Transform::default().with_translation(Vec3::new(
                            x as f32 * TILE_SIZE,
                            y as f32 * TILE_SIZE,
                            0.,
                        )),
                        ..default()
                    })
                    .insert(MapTile(x, y))
                    .id();
                map.put_tile_entity(x, y, tile_ent);
            }
        }
    }
}

fn spawn_earth(mut commands: Commands) {
    commands.spawn(PlanetMap::new_earth());
}
