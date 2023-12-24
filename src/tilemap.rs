use bevy::app::{App, Plugin};
use bevy::prelude::*;
use crate::loading::TextureAssets;

use bevy_ecs_tilemap::prelude::*;

use crate::GameState;

pub struct TilePlugin;

#[derive(Component)]
pub struct Tilemap;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for crate::tilemap::TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), crate::tilemap::spawn_tilemap);
    }
}

fn spawn_tilemap(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    #[cfg(all(not(feature = "atlas"), feature = "render"))] array_texture_loader: Res<
        ArrayTextureLoader,
    >,
) {
    let tile_image = textures.bevy.clone();
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
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();
    let transform = get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0);

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        // texture: TilemapTexture::Single(tile_image),
        tile_size,
        transform,
        ..Default::default()
    });

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