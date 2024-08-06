// src/tilemap.rs

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::Rng;

use crate::GameState;

pub struct TilePlugin;

#[derive(Component)]
pub struct Tilemap;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(TilemapPlugin)
            .init_resource::<GameClock>() // Add this line
            .init_resource::<JobMarket>()
            .init_resource::<HousingMarket>()
            .add_systems(OnEnter(GameState::Playing), spawn_tilemap)
            .add_systems(Update, (
                update_game_time,
                update_pops,
                move_pops,
                update_pop_visuals,
                handle_speed_input,
                manage_markets,
                assign_jobs_and_housing,  // Add this line
            ).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct LastUpdate(f64);

#[derive(Component)]
pub struct HouseID(u32);

#[derive(Component, Default)]
pub struct Pop {
    pub(crate) money: f32,
    pub(crate) hunger: f32,
    pub(crate) energy: f32,
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
    current_tick: u64,
    ticks_per_hour: u64,
    hours_per_day: u64,
    speed: u64,
    paused: bool,
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
            self.current_tick += self.speed;
        }
    }

    pub fn day(&self) -> u64 {
        self.current_tick / (self.ticks_per_hour * self.hours_per_day) + 1
    }

    pub fn hour(&self) -> u64 {
        (self.current_tick / self.ticks_per_hour) % self.hours_per_day
    }

    pub fn minute(&self) -> u64 {
        self.current_tick % self.ticks_per_hour
    }

    pub fn set_speed(&mut self, speed: u64) {
        self.speed = speed.max(1);
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
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

fn update_game_clock(time: Res<Time>, mut game_clock: ResMut<GameClock>) {
    let ticks_to_advance = (time.delta_seconds() as f64 * game_clock.speed * game_clock.ticks_per_second as f64) as u64;
    for _ in 0..ticks_to_advance {
        game_clock.tick();
    }
}

#[derive(Component)]
pub struct Building;
// src/tilemap.rs (continued)

use bevy::math::Vec2;

fn move_pops(
    mut pop_query: Query<(&mut Pop, &mut TilePos)>,
    time: Res<Time>,
    game_time: Res<GameTime>,
) {
    let delta = time.delta_seconds() * game_time.speed;
    for (mut pop, mut tile_pos) in pop_query.iter_mut() {
        if let Some(destination) = pop.destination {
            let current_pos = Vec2::new(tile_pos.x as f32, tile_pos.y as f32);
            let dest_pos = Vec2::new(destination.x as f32, destination.y as f32);
            let direction = (dest_pos - current_pos).normalize();

            let movement = direction * delta * 0.1; // Adjust speed as needed
            let new_pos = current_pos + movement;

            *tile_pos = TilePos::new(new_pos.x.round() as u32, new_pos.y.round() as u32);
            pop.position = *tile_pos; // Update pop's internal position

            if current_pos.distance(dest_pos) < 0.1 {
                *tile_pos = destination;
                pop.position = destination;
                pop.destination = None;
            }
        }
    }
}

fn spawn_tilemap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_time: ResMut<GameTime>,
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

    *game_time = GameTime::default();
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
                money: 100.0,
                hunger: 0.0,
                energy: 100.0,
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


fn update_pops(
    game_clock: Res<GameClock>,
    mut pop_query: Query<&mut Pop>,
    house_query: Query<&House>,
    workplace_query: Query<&Workplace>,
    restaurant_query: Query<&Restaurant>,
) {
    let delta_hours = 1.0 / game_clock.ticks_per_second as f64;

    for mut pop in pop_query.iter_mut() {
        // Increase hunger and decrease energy over time
        pop.hunger += 5.0 * delta_hours; // Increase hunger by 5 points per hour
        pop.energy -= 2.5 * delta_hours; // Decrease energy by 2.5 points per hour

        // Update pop state and destination based on needs and time of day
        if pop.hunger > 70.0 && pop.state != PopState::Eating {
            pop.state = PopState::Eating;
            pop.destination = find_nearest_restaurant(&pop.position, &restaurant_query);
        } else if pop.energy < 20.0 && pop.state != PopState::Sleeping {
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
                if pop.hunger > 0.0 {
                    pop.hunger -= 20.0 * delta_hours; // Reduce hunger
                    pop.money -= 5.0 * delta_hours; // Cost of food
                } else {
                    pop.state = PopState::Idle;
                    pop.destination = None;
                }
            }
            PopState::Sleeping => {
                pop.energy += 10.0 * delta_hours; // Recover energy
                if pop.energy >= 100.0 {
                    pop.state = PopState::Idle;
                    pop.destination = None;
                }
                // Sleeping consumes less food
                pop.hunger += 1.0 * delta_hours;
            }
            PopState::Working => {
                if let Some(job) = &pop.job {
                    pop.money += job.salary * delta_hours; // Earn money
                    pop.energy -= 1.0 * delta_hours; // Work consumes some energy
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
                        pop.position.x.saturating_add(random_offset.x).saturating_sub(2),
                        pop.position.y.saturating_add(random_offset.y).saturating_sub(2)
                    ));
                }
                // Idle state consumes energy and increases hunger slightly
                pop.energy -= 0.5 * delta_hours;
                pop.hunger += 2.0 * delta_hours;
            }
        }

        // Clamp values to ensure they stay within reasonable bounds
        pop.hunger = pop.hunger.clamp(0.0, 100.0);
        pop.energy = pop.energy.clamp(0.0, 100.0);
        pop.money = pop.money.max(0.0); // Ensure money doesn't go negative
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

        // Set pop positions
        for (_, tile_pos) in pop_query.iter() {
            if let Some(tile_entity) = tile_storage.get(tile_pos) {
                commands.entity(tile_entity).insert(TileTextureIndex(1)); // Pop texture
            }
        }

        // Set house positions
        for (_, tile_pos) in house_query.iter() {
            if let Some(tile_entity) = tile_storage.get(tile_pos) {
                commands.entity(tile_entity).insert(TileTextureIndex(2)); // House texture
            }
        }

        // Set workplace positions
        for (_, tile_pos) in workplace_query.iter() {
            if let Some(tile_entity) = tile_storage.get(tile_pos) {
                commands.entity(tile_entity).insert(TileTextureIndex(3)); // Workplace texture
            }
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

fn update_game_time(time: Res<Time>, mut game_time: ResMut<GameTime>) {
    game_time.hour += time.delta_seconds() * game_time.speed * 0.5; // 0.5 means 1 real second = 30 in-game minutes at normal speed
    while game_time.hour >= 24.0 {
        game_time.hour -= 24.0;
        game_time.day += 1;
    }
}

fn handle_speed_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_time: ResMut<GameTime>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyZ) {
        // Halve the speed
        game_time.speed = (game_time.speed / 2.0).max(2f32.powf(-6f32));
    }
    if keyboard_input.just_pressed(KeyCode::KeyX) {
        // Double the speed
        game_time.speed = (game_time.speed * 2.0).min(2f32.powf(6f32));
    }
}