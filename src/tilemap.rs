use bevy::app::{App, Plugin};
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::square_grid::neighbors::Neighbors;
use crate::loading::TextureAssets;

use bevy_ecs_tilemap::prelude::*;

use crate::GameState;
use crate::player::Player;

pub struct TilePlugin;

#[derive(Component)]
pub struct Tilemap;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for crate::tilemap::TilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(TilemapPlugin)
            .add_systems(OnEnter(GameState::Playing), crate::tilemap::spawn_tilemap)
            // .add_systems(Update, crate::tilemap::update.run_if(in_state(GameState::Playing)))
        ;
    }
}

#[derive(Component)]
pub struct LastUpdate(f64);

#[derive(Component)]
pub struct HouseID(u32);

#[derive(Component, Default)]
pub struct Pop {
    money: u64,
    race: String,
    sex: Sex,
}

#[derive(Default)]
enum Sex {
    male,
    female,
    #[default]
    nonbinary
}

#[derive(Bundle, Default)]
struct PopBundle {
    pub pop: Pop,
    pub tile_bundle: TileBundle,
}

#[derive(Component)]
pub struct Building;

fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut tile_storage_query: Query<(&TileStorage, &TilemapSize, &mut LastUpdate)>,
    tile_query: Query<(Entity, &TilePos, &TileVisible)>,
) {
    let current_time = time.elapsed_seconds_f64();
    let (tile_storage, map_size, mut last_update) = tile_storage_query.single_mut();
    if current_time - last_update.0 > 0.1 {
        for (entity, position, visibility) in tile_query.iter() {
            let neighbor_count =
                Neighbors::get_square_neighboring_positions(position, map_size, true)
                    .entities(tile_storage)
                    .iter()
                    .filter(|neighbor| {
                        let tile_component =
                            tile_query.get_component::<TileVisible>(**neighbor).unwrap();
                        tile_component.0
                    })
                    .count();

            let was_alive = visibility.0;

            let is_alive = match (was_alive, neighbor_count) {
                (true, x) if x < 2 => false,
                (true, 2) | (true, 3) => true,
                (true, x) if x > 3 => false,
                (false, 3) => true,
                (otherwise, _) => otherwise,
            };

            if is_alive && !was_alive {
                commands.entity(entity).insert(TileVisible(true));
            } else if !is_alive && was_alive {
                commands.entity(entity).insert(TileVisible(false));
            }
        }
        last_update.0 = current_time;
    }
}

fn spawn_tilemap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    #[cfg(all(not(feature = "atlas"), feature = "render"))] array_texture_loader: Res<
        ArrayTextureLoader,
    >,
) {
    let tile_image: Handle<Image> = asset_server.load("textures/tiles.png");
    // the color black Image::from_color(Color::rgb(0.0, 0.0, 0.0));
    // let tile_image = Handle::

    let map_size = TilemapSize { x: 32, y: 32 };

// Create a tilemap entity a little early.
// We want this entity early because we need to tell each tile which tilemap entity
// it is associated with. This is done with the TilemapId component on each tile.
// Eventually, we will insert the `TilemapBundle` bundle on the entity, which
// will contain various necessary components, such as `TileStorage`.
    let tilemap_entity = commands.spawn_empty().id();

// To begin creating the map we will need a `TileStorage` component.
// This component is a grid of tile entities and is used to help keep track of individual
// tiles in the world. If you have multiple layers of tiles you would have a tilemap entity
// per layer, each with their own `TileStorage` component.
    let mut tile_storage = TileStorage::empty(map_size);

// Spawn the elements of the tilemap.
// Alternatively, you can use helpers::filling::fill_tilemap.
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_bundle = TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                ..Default::default()
            };
            // just one dude at middle, else don't put in a tile type
            let tile_entity = if x == map_size.x / 2 && y == map_size.y / 2 {
                commands
                    .spawn(
                        PopBundle {
                            tile_bundle: TileBundle {
                                texture_index: TileTextureIndex(1),
                                ..tile_bundle
                            },
                            ..Default::default()
                        }
                    )
                    .id()
            } else {
                commands.spawn(tile_bundle)
                    .id()
            };
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();
    let transform = get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0);

    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(tile_image),
            tile_size,
            transform,
            ..Default::default()
        },
        LastUpdate(0.0),
    ));

// Add atlas to array texture loader so it's preprocessed before we need to use it.
// Only used when the atlas feature is off and we are using array textures.
    #[cfg(all(not(feature = "atlas"), feature = "render"))]
    {
        array_texture_loader.add(TilemapArrayTexture {
            texture: TilemapTexture::Single(asset_server.load("tiles.png")),
            tile_size,
            ..Default::default()
        });
    }
}