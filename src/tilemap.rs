// src/tilemap.rs

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::Rng;

use crate::GameState;

pub struct TilePlugin;

#[derive(Component)]
pub struct Tilemap;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin)
            .init_resource::<GameClock>()
            .init_resource::<JobMarket>()
            .init_resource::<HousingMarket>()
            .add_systems(OnEnter(GameState::Playing), spawn_tilemap)
            .add_systems(Update, (
                update_fixed_time,
                update_pop_visuals,
                handle_speed_input,
            ).run_if(in_state(GameState::Playing)))
            .add_systems(FixedUpdate, (
                update_game_clock,
                update_pops,
                move_pops,
                manage_markets,
                assign_jobs_and_housing,
            ).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct LastUpdate(f64);

#[derive(Component)]
pub struct HouseID(u32);

#[derive(Component, Default)]
pub struct Pop {
    pub(crate) money: i32,
    pub(crate) hunger: u32,
    pub(crate) energy: u32,
    pub(crate) job: Option<Job>,
    pub(crate) home: Option<Entity>,
    position: TilePos,
    destination: Option<TilePos>,
    pub state: PopState,
}

#[derive(Default, Eq, PartialEq, Copy, Clone, Debug, Hash, Reflect)]
pub enum PopState {
    #[default]
    Idle,
    Working,
    Eating,
    Sleeping,
}

#[derive(Component)]
pub struct Job {
    workplace: Entity,
    salary: f32,
    position: TilePos,
}

#[derive(Component)]
pub struct Workplace {
    pub(crate) capacity: u32,
    employees: Vec<Entity>,
    position: TilePos,
}

#[derive(Component)]
pub struct Restaurant {
    pub capacity: u32,
    pub position: TilePos,
}

#[derive(Component)]
pub struct House {
    pub(crate) capacity: u32,
    residents: Vec<Entity>,
    position: TilePos,
}


use bevy::prelude::*;

#[derive(Resource)]
pub struct GameClock {
    pub current_tick: u64,
    pub ticks_per_hour: u64,
    pub hours_per_day: u64,
    pub speed: u32,
    pub paused: bool,
}

impl Default for GameClock {
    fn default() -> Self {
        Self {
            current_tick: 0,
            ticks_per_hour: 60, // 1 tick per minute
            hours_per_day: 24,
            speed: 1,
            paused: false,
        }
    }
}

impl GameClock {
    pub fn tick(&mut self) {
        if !self.paused {
            self.current_tick += 1;
        }
    }

    pub fn day(&self) -> u64 {
        self.current_tick / (self.ticks_per_hour * self.hours_per_day) + 1
    }

    pub fn hour(&self) -> f64 {
        (self.current_tick % (self.ticks_per_hour * self.hours_per_day)) as f64
            / self.ticks_per_hour as f64
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn ticks_per_second(&self) -> f64 {
        self.ticks_per_hour as f64 * self.speed as f64 / 60.0
    }
}

#[derive(Default)]
enum Sex {
    Male,
    Female,
    #[default]
    NonBinary,
}

#[derive(Bundle, Default)]
struct PopBundle {
    pub pop: Pop,
    pub tile_bundle: TileBundle,
}

fn update_game_clock(mut game_clock: ResMut<GameClock>) {
    game_clock.tick();
}

fn update_fixed_time(
    game_clock: Res<GameClock>,
    mut fixed_time: ResMut<Time<Fixed>>,
) {
    if !game_clock.paused {
        fixed_time.set_timestep_hz(game_clock.ticks_per_second());
    } else {
        fixed_time.set_timestep_hz(0.0);
    }
}

#[derive(Component)]
pub struct Building;
// src/tilemap.rs (continued)

use bevy::math::Vec2;


// Modify the move_pops function
fn move_pops(
    mut pop_query: Query<(&mut Pop, &mut TilePos)>,
    tilemap_query: Query<&TilemapSize>,
) {
    let map_size = tilemap_query.single();

    for (mut pop, mut tile_pos) in pop_query.iter_mut() {
        if let Some(destination) = pop.destination {
            let current_pos = Vec2::new(tile_pos.x as f32, tile_pos.y as f32);
            let dest_pos = Vec2::new(destination.x as f32, destination.y as f32);
            let diff = dest_pos - current_pos;

            // Decide whether to move in X or Y direction
            let (new_x, new_y) = if diff.x.abs() >= diff.y.abs() {
                // Move in X direction
                (
                    (current_pos.x + diff.x.signum()).clamp(0.0, (map_size.x - 1) as f32) as u32,
                    current_pos.y as u32
                )
            } else {
                // Move in Y direction
                (
                    current_pos.x as u32,
                    (current_pos.y + diff.y.signum()).clamp(0.0, (map_size.y - 1) as f32) as u32
                )
            };

            let new_tile_pos = TilePos::new(new_x, new_y);

            // Only update if the new position is different
            if new_tile_pos != *tile_pos {
                *tile_pos = new_tile_pos;
                pop.position = *tile_pos;
            }

            // Check if the pop has reached the destination
            if *tile_pos == destination {
                pop.destination = None;
            }
        }
    }
}

fn spawn_tilemap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let map_size = TilemapSize { x: 32, y: 32 };
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    let tile_image: Handle<Image> = asset_server.load("textures/tiles.png");

    let mut rng = rand::thread_rng();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = match rng.gen_range(0..100) {
                0..=69 => spawn_empty_tile(&mut commands, tile_pos, tilemap_entity), // 70% empty
                70..=84 => spawn_pop(&mut commands, tile_pos, tilemap_entity),       // 15% pop
                85..=94 => spawn_house(&mut commands, tile_pos, tilemap_entity),     // 10% house
                _ => spawn_workplace(&mut commands, tile_pos, tilemap_entity),       // 5% workplace
            };
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let grid_size = tile_size.into();
    let map_type = TilemapType::default();
    let transform = get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0);

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(tile_image),
        tile_size,
        transform,
        ..default()
    });
}

fn spawn_empty_tile(commands: &mut Commands, tile_pos: TilePos, tilemap_entity: Entity) -> Entity {
    commands
        .spawn(TileBundle {
            position: tile_pos,
            tilemap_id: TilemapId(tilemap_entity),
            texture_index: TileTextureIndex(0), // Assuming 0 is the empty tile texture
            ..default()
        })
        .id()
}

fn spawn_pop(commands: &mut Commands, tile_pos: TilePos, tilemap_entity: Entity) -> Entity {
    commands
        .spawn((
            Pop {
                money: 100,
                hunger: 0,
                energy: 100,
                job: None,
                home: None,
                position: tile_pos,
                destination: None,
                state: PopState::Idle,
            },
            TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index: TileTextureIndex(1), // Assuming 1 is the pop texture
                ..default()
            },
        ))
        .id()
}

fn spawn_house(commands: &mut Commands, tile_pos: TilePos, tilemap_entity: Entity) -> Entity {
    commands
        .spawn((
            House {
                capacity: 4,
                residents: Vec::new(),
                position: tile_pos,
            },
            TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index: TileTextureIndex(2), // Assuming 2 is the house texture
                ..default()
            },
        ))
        .id()
}

fn spawn_workplace(commands: &mut Commands, tile_pos: TilePos, tilemap_entity: Entity) -> Entity {
    commands
        .spawn((
            Workplace {
                capacity: 10,
                employees: Vec::new(),
                position: tile_pos,
            },
            TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index: TileTextureIndex(3), // Assuming 3 is the workplace texture
                ..default()
            },
        ))
        .id()
}

fn find_nearest_restaurant(
    pop_position: &TilePos,
    restaurant_query: &Query<&Restaurant>,
) -> Option<TilePos> {
    restaurant_query
        .iter()
        .min_by_key(|restaurant| {
            let dx = restaurant.position.x as i32 - pop_position.x as i32;
            let dy = restaurant.position.y as i32 - pop_position.y as i32;
            dx * dx + dy * dy
        })
        .map(|restaurant| restaurant.position)
}

fn spawn_restaurant(commands: &mut Commands, tile_pos: TilePos, tilemap_entity: Entity) -> Entity {
    commands
        .spawn((
            Restaurant {
                capacity: 20,
                position: tile_pos,
            },
            TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index: TileTextureIndex(4), // Assuming 4 is the restaurant texture
                ..default()
            },
        ))
        .id()
}


// Modify the update_pops function
fn update_pops(
    game_clock: Res<GameClock>,
    mut pop_query: Query<&mut Pop>,
    house_query: Query<&House>,
    workplace_query: Query<&Workplace>,
    restaurant_query: Query<&Restaurant>,
    tilemap_query: Query<&TilemapSize>,
) {
    let map_size = tilemap_query.single();
    for mut pop in pop_query.iter_mut() {
        // Increase hunger and decrease energy every tick
        pop.hunger = pop.hunger.saturating_add(1);
        pop.energy = pop.energy.saturating_sub(1);

        // Update pop state and destination based on needs and time of day
        if pop.hunger > 7000 && pop.state != PopState::Eating {
            pop.state = PopState::Eating;
            pop.destination = find_nearest_restaurant(&pop.position, &restaurant_query);
        } else if pop.energy < 2000 && pop.state != PopState::Sleeping {
            pop.state = PopState::Sleeping;
            pop.destination = pop.home.and_then(|home| house_query.get(home).ok().map(|h| h.position));
        } else if pop.state != PopState::Working && game_clock.hour() >= 9.0 && game_clock.hour() < 17.0 {
            pop.state = PopState::Working;
            pop.destination = pop.job.as_ref().map(|job| job.position);
        } else if pop.state == PopState::Working && (game_clock.hour() < 9.0 || game_clock.hour() >= 17.0) {
            pop.state = PopState::Idle;
            pop.destination = None;
        }

        // Handle actions based on state
        match pop.state {
            PopState::Eating => {
                if pop.hunger > 0 {
                    pop.hunger = pop.hunger.saturating_sub(20);
                    pop.money = pop.money.saturating_sub(1); // Cost of food
                } else {
                    pop.state = PopState::Idle;
                    pop.destination = None;
                }
            }
            PopState::Sleeping => {
                pop.energy = pop.energy.saturating_add(10);
                if pop.energy >= 10000 {
                    pop.state = PopState::Idle;
                    pop.destination = None;
                }
                // Sleeping consumes less food
                pop.hunger = pop.hunger.saturating_add(1);
            }
            PopState::Working => {
                if let Some(job) = &pop.job {
                    pop.money = pop.money.saturating_add((job.salary / (game_clock.ticks_per_hour * 8) as f32) as i32); // Assuming 8-hour workday
                    pop.energy = pop.energy.saturating_sub(1);
                }
            }
            PopState::Idle => {
                // In idle state, pops might wander or socialize
                if pop.destination.is_none() {
                    // Randomly set a new destination within a certain range
                    let random_offset = TilePos::new(
                        rand::random::<u32>() % 5,
                        rand::random::<u32>() % 5
                    );
                    pop.destination = Some(TilePos::new(
                        pop.position.x.saturating_add(random_offset.x).saturating_sub(2).min(map_size.x - 1),
                        pop.position.y.saturating_add(random_offset.y).saturating_sub(2).min(map_size.y - 1),
                    ));
                }
                // Idle state consumes energy and increases hunger slightly
                pop.energy = pop.energy.saturating_sub(1);
                pop.hunger = pop.hunger.saturating_add(1);
            }
        }

        // Clamp values to ensure they stay within reasonable bounds
        pop.hunger = pop.hunger.min(10000);
        pop.energy = pop.energy.min(10000);
    }
}

use bevy_ecs_tilemap::prelude::*;

fn update_pop_visuals(
    mut commands: Commands,
    mut tilemap_query: Query<(Entity, &mut TileStorage)>,
    pop_query: Query<(Entity, &TilePos), With<Pop>>,
    house_query: Query<(Entity, &TilePos), With<House>>,
    workplace_query: Query<(Entity, &TilePos), With<Workplace>>,
) {
    if let Ok((tilemap_entity, mut tile_storage)) = tilemap_query.get_single_mut() {
        // Reset all tiles to empty
        for x in 0..tile_storage.size.x {
            for y in 0..tile_storage.size.y {
                let tile_pos = TilePos { x, y };
                if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                    commands.entity(tile_entity).insert(TileTextureIndex(0)); // Empty tile texture
                }
            }
        }

        // Helper function to safely update tile texture
        let mut update_tile_texture = |tile_pos: &TilePos, texture_index: u32| {
            if tile_pos.x < tile_storage.size.x && tile_pos.y < tile_storage.size.y {
                if let Some(tile_entity) = tile_storage.get(tile_pos) {
                    commands.entity(tile_entity).insert(TileTextureIndex(texture_index));
                }
            }
        };

        // Set pop positions
        for (_, tile_pos) in pop_query.iter() {
            update_tile_texture(tile_pos, 1);
        }

        // Set house positions
        for (_, tile_pos) in house_query.iter() {
            update_tile_texture(tile_pos, 2);
        }

        // Set workplace positions
        for (_, tile_pos) in workplace_query.iter() {
            update_tile_texture(tile_pos, 3);
        }
    }
}

fn assign_jobs_and_housing(
    mut commands: Commands,
    mut pop_query: Query<(Entity, &mut Pop)>,
    mut job_market: ResMut<JobMarket>,
    mut housing_market: ResMut<HousingMarket>,
    mut workplace_query: Query<&mut Workplace>,
    mut house_query: Query<&mut House>,
) {
    for (pop_entity, mut pop) in pop_query.iter_mut() {
        // Assign job if unemployed
        if pop.job.is_none() {
            if let Some((workplace_entity, salary, position)) = job_market.available_jobs.pop() {
                pop.job = Some(Job {
                    workplace: workplace_entity,
                    salary,
                    position,
                });
                // Update workplace
                if let Ok(mut workplace) = workplace_query.get_mut(workplace_entity) {
                    workplace.employees.push(pop_entity);
                }
            }
        }

        // Assign home if homeless
        if pop.home.is_none() {
            if let Some((house_entity, position)) = housing_market.available_houses.pop() {
                pop.home = Some(house_entity);
                pop.destination = Some(position);
                // Update house
                if let Ok(mut house) = house_query.get_mut(house_entity) {
                    house.residents.push(pop_entity);
                }
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct JobMarket {
    pub available_jobs: Vec<(Entity, f32, TilePos)>, // (Workplace, Salary, Position)
}

#[derive(Resource, Default)]
pub struct HousingMarket {
    pub available_houses: Vec<(Entity, TilePos)>, // (House, Position)
}

// src/tilemap.rs

fn manage_markets(
    mut job_market: ResMut<JobMarket>,
    mut housing_market: ResMut<HousingMarket>,
    workplace_query: Query<(Entity, &Workplace)>,
    house_query: Query<(Entity, &House)>,
) {
    job_market.available_jobs.clear();
    for (entity, workplace) in workplace_query.iter() {
        let available_positions = workplace.capacity as i32 - workplace.employees.len() as i32;
        if available_positions > 0 {
            job_market.available_jobs.push((entity, 75.0, workplace.position));
        }
    }

    housing_market.available_houses.clear();
    for (entity, house) in house_query.iter() {
        if house.residents.len() < house.capacity as usize {
            housing_market.available_houses.push((entity, house.position));
        }
    }
}

fn handle_speed_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_clock: ResMut<GameClock>
) {
    let speed = game_clock.speed;
    if keyboard_input.just_pressed(KeyCode::KeyZ) {
        // Decrease speed, but not below 1
        game_clock.speed = speed.saturating_div(2).max(1);
    }
    if keyboard_input.just_pressed(KeyCode::KeyX) {
        // Increase speed, with some upper limit (e.g., 10)
        game_clock.speed = speed.saturating_mul(2).min(512);
    }
    if keyboard_input.just_pressed(KeyCode::Space) {
        game_clock.toggle_pause();
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use bevy::prelude::*;
    use bevy_ecs_tilemap::prelude::*;
    use crate::tilemap::{move_pops, Pop};

    proptest! {
        #[test]
        fn test_pop_movement(
            start_x in 0u32..20,
            start_y in 0u32..20,
            dest_x in 0u32..20,
            dest_y in 0u32..20,
            num_ticks in 1u32..50
        ) {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins)
               .add_systems(Update, move_pops);

            let start_pos = TilePos::new(start_x, start_y);
            let dest_pos = TilePos::new(dest_x, dest_y);

            let pop_entity = app.world_mut().spawn((
                Pop {
                    position: start_pos,
                    destination: Some(dest_pos),
                    ..Default::default()
                },
                start_pos,
            )).id();

            // Run the simulation for the generated number of ticks
            for _ in 0..num_ticks {
                app.update();
            }

            let total_distance = ((dest_x as i32 - start_x as i32).abs() +
                                  (dest_y as i32 - start_y as i32).abs()) as u32;

            let mut query = app.world_mut().query::<(&Pop, &TilePos)>();
            let (pop, tile_pos) = query.get(app.world_mut(), pop_entity).unwrap();

            // Calculate Manhattan distance moved
            let distance_moved = ((tile_pos.x as i32 - start_x as i32).abs() +
                                  (tile_pos.y as i32 - start_y as i32).abs()) as u32;

            // Calculate Manhattan distance to destination
            let distance_to_dest = ((dest_x as i32 - tile_pos.x as i32).abs() +
                                    (dest_y as i32 - tile_pos.y as i32).abs()) as u32;

            // Assert that the pop has moved the correct number of steps
            prop_assert!(distance_moved == num_ticks.min(total_distance),
                "Pop should move one step per tick until reaching the destination. Expected: {}, Actual: {}", num_ticks.min(total_distance), distance_moved);

            // Assert that the pop is either at the destination or on the way
            prop_assert!(distance_to_dest == total_distance.saturating_sub(distance_moved),
                "Pop should be at the correct position on the path to destination");

            // If we've had enough ticks to reach the destination, assert that we're there
            if num_ticks >= total_distance {
                prop_assert_eq!(pop.position, dest_pos, "Pop should be at the destination");
                prop_assert_eq!(pop.destination, None, "Destination should be cleared when reached");
            } else {
                prop_assert_eq!(pop.destination, Some(dest_pos), "Destination should remain if not reached");
            }

            // Assert that the Pop's internal position matches its TilePos
            prop_assert_eq!(pop.position, *tile_pos, "Pop's internal position should match its TilePos");
        }
    }
}