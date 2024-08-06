use bevy::prelude::*;
use crate::tilemap::{GameClock, House, Pop, Workplace};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(Update, (update_ui, handle_speed_buttons));
    }
}

#[derive(Component)]
struct GameInfoText;

#[derive(Component)]
struct SpeedButton(f32);

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "Game Info",
            TextStyle {
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            }),
        GameInfoText,
    ));
}

// Modify the update_ui function to use the new GameClock
fn update_ui(
    mut text_query: Query<&mut Text, With<GameInfoText>>,
    game_clock: Res<GameClock>,
    pop_query: Query<&Pop>,
    house_query: Query<&House>,
    workplace_query: Query<&Workplace>,
) {
    if let Ok(mut text) = text_query.get_single_mut() {
        let pop_count = pop_query.iter().count();
        let total_money: i32 = pop_query.iter().map(|pop| pop.money).sum();
        let average_money = if pop_count > 0 { total_money as f32 / pop_count as f32 } else { 0.0 };

        let employed_count = pop_query.iter().filter(|pop| pop.job.is_some()).count();
        let homeless_count = pop_query.iter().filter(|pop| pop.home.is_none()).count();

        let total_hunger: u32 = pop_query.iter().map(|pop| pop.hunger).sum();
        let average_hunger = if pop_count > 0 { total_hunger as f32 / pop_count as f32 } else { 0.0 };

        let total_energy: u32 = pop_query.iter().map(|pop| pop.energy).sum();
        let average_energy = if pop_count > 0 { total_energy as f32 / pop_count as f32 } else { 0.0 };

        let house_count = house_query.iter().count();
        let total_house_capacity: u32 = house_query.iter().map(|house| house.capacity).sum();

        let workplace_count = workplace_query.iter().count();
        let total_job_capacity: u32 = workplace_query.iter().map(|workplace| workplace.capacity).sum();

        text.sections[0].value = format!(
            "Day: {}, Time: {:02}:{:02}\nSpeed: {}x({} ticks/sec)\n\\n\n\
            Population: {}\n\
            Employed: {} ({:.1}%)\n\
            Homeless: {} ({:.1}%)\n\
            Average Money: ${:.2}\n\
            Average Hunger: {:.1}/10000\n\
            Average Energy: {:.1}/10000\n\n\
            Houses: {} (Capacity: {})\n\
            Workplaces: {} (Capacity: {})",
            game_clock.day(),
            game_clock.hour(),
            (game_clock.hour().fract() * 60.0) as u32,
            game_clock.speed,
            game_clock.ticks_per_second(),
            pop_count,
            employed_count,
            (employed_count as f32 / pop_count as f32) * 100.0,
            homeless_count,
            (homeless_count as f32 / pop_count as f32) * 100.0,
            average_money,
            average_hunger,
            average_energy,
            house_count,
            total_house_capacity,
            workplace_count,
            total_job_capacity
        );
    }
}

// Modify the handle_speed_buttons function
fn handle_speed_buttons(
    mut interaction_query: Query<
        (&Interaction, &SpeedButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_clock: ResMut<GameClock>,
) {
    for (interaction, speed_button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            game_clock.speed = speed_button.0 as u32;
        }
    }
}