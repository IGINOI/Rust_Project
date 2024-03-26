use bevy::prelude::*;
use crate::{SQUARE_FRAME_PATH, FRAME_SIZE, BIG_RECTANGLE_FRAME_PATH, MAP_SIZE, LITTLE_RECTANGLE_FRAME_PATH};

pub struct SpawnFramePlugin;

impl Plugin for SpawnFramePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_weather_frame)
            .add_systems(Startup, spawn_time_frame)
            .add_systems(Startup, spawn_energy_frame)
            .add_systems(Startup, spawn_message_frame)
            .add_systems(Startup, spawn_backpack_frame)
            .add_systems(Startup, spawn_map_frame)
        ;
    }
}

//I build a list of component that I will use to recognise different component in the queries
#[derive(Component)]
pub struct BackpackFrame;

#[derive(Component)]
pub struct TimeFrame;
#[derive(Component)]
pub struct WeatherFrame;

#[derive(Component)]
pub struct EnergyFrame;
#[derive(Component)]
pub struct EnergyTotFrame;
#[derive(Component)]
pub struct EnergyRemFrame;
#[derive(Component)]
pub struct EnergyAddFrame;

#[derive(Component)]
pub struct MessageFrame;

#[derive(Component)]
pub struct MapFrame;

fn spawn_backpack_frame(
    mut commands: Commands,
    assets: Res<AssetServer>
)
{
    for i in 0..20{
        let backpack_frame =
            (ImageBundle{
                style: Style{
                    position_type: PositionType::Absolute,
                    align_self: AlignSelf::End,
                    justify_self: JustifySelf::Start,
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    justify_items: JustifyItems::Center,
                    align_items: AlignItems::Center,
                    left: Val::Vw(i as f32 * 5.0),
                    height: Val::Vw(5.0),
                    width: Val::Vw(5.0),
                    ..default()
                },
                image: UiImage{
                    texture: assets.load(SQUARE_FRAME_PATH),
                    ..default()
                },
                ..default()
            },BackpackFrame, Name::new("BackPack_Frame"));
        commands.spawn(backpack_frame);
    }
}

fn spawn_weather_frame(
    mut commands: Commands,
    assets: Res<AssetServer>
)
{
    let weather_frame = (ImageBundle {
        style: Style {
            position_type: PositionType::Relative,
            align_self: AlignSelf::Start,
            justify_self: JustifySelf::End,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            justify_items: JustifyItems::Center,
            align_items: AlignItems::Center,
            height: Val::Vw(FRAME_SIZE),
            width: Val::Vw(FRAME_SIZE),
            ..default()
        },
        image: UiImage {
            texture: assets.load(SQUARE_FRAME_PATH),
            ..default()
        },
        ..default()
    }, WeatherFrame, Name::new("Weather_Frame"));
    commands.spawn(weather_frame);
}

fn spawn_time_frame(
    mut commands: Commands,
    assets: Res<AssetServer>
)
{
    let time_frame = (ImageBundle{
        style: Style{
            position_type: PositionType::Relative,
            align_self: AlignSelf::Start,
            justify_self: JustifySelf::End,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            justify_items: JustifyItems::Center,
            align_items: AlignItems::Center,
            right: Val::Vw(FRAME_SIZE),
            height: Val::Vw(FRAME_SIZE),
            width: Val::Vw(FRAME_SIZE),
            ..default()
        },
        image: UiImage{
            texture: assets.load(SQUARE_FRAME_PATH),
            ..default()
        },
        ..default()
    }, TimeFrame, Name::new("Time_Frame"));
    commands.spawn(time_frame);
}

fn spawn_energy_frame(
    mut commands: Commands,
    assets: Res<AssetServer>
)
{
    let energy_frame = (ImageBundle{
        style: Style{
            align_self: AlignSelf::Start,
            justify_self: JustifySelf::Start,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            justify_items: JustifyItems::Center,
            align_items: AlignItems::Center,
            height: Val::Vw(FRAME_SIZE),
            width: Val::Vw(FRAME_SIZE * 3.0),
            ..default()
        },
        image: UiImage{
            texture: assets.load(BIG_RECTANGLE_FRAME_PATH),
            ..default()
        },
        ..default()
    }, EnergyFrame, Name::new("Energy_Frame"));

    commands.spawn(energy_frame).with_children(|parent|{
        parent.spawn((TextBundle{
            text: Text::from_section(
                format!("ENERGY"),
                TextStyle{
                    font_size: 50.0,
                    color: Color::TEAL,
                    ..default()
                }),
            ..default()

        }, Name::new("Energy_Writing")));
    });

    let energy_tot_frame = (ImageBundle{
        style: Style{
            align_self: AlignSelf::Start,
            justify_self: JustifySelf::Start,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            justify_items: JustifyItems::Center,
            align_items: AlignItems::Center,
            top: Val::Vw(FRAME_SIZE),
            height: Val::Vw(FRAME_SIZE),
            width: Val::Vw(FRAME_SIZE),
            ..default()
        },
        image: UiImage{
            texture: assets.load(SQUARE_FRAME_PATH),
            ..default()
        },
        ..default()
    }, EnergyTotFrame, Name::new("Energy_Tot_Frame"));
    commands.spawn(energy_tot_frame);

    let energy_rem_frame = (ImageBundle{
        style: Style{
            align_self: AlignSelf::Start,
            justify_self: JustifySelf::Start,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            justify_items: JustifyItems::Center,
            align_items: AlignItems::Center,
            left: Val::Vw(FRAME_SIZE),
            top: Val::Vw(FRAME_SIZE),
            height: Val::Vw(FRAME_SIZE),
            width: Val::Vw(FRAME_SIZE),
            ..default()
        },
        image: UiImage{
            texture: assets.load(SQUARE_FRAME_PATH),
            ..default()
        },
        ..default()
    }, EnergyRemFrame, Name::new("Energy_Rem_Frame"));
    commands.spawn(energy_rem_frame);

    let energy_add_frame = (ImageBundle{
        style: Style{
            align_self: AlignSelf::Start,
            justify_self: JustifySelf::Start,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            justify_items: JustifyItems::Center,
            align_items: AlignItems::Center,
            left: Val::Vw(FRAME_SIZE*2.0),
            top: Val::Vw(FRAME_SIZE),
            height: Val::Vw(FRAME_SIZE),
            width: Val::Vw(FRAME_SIZE),
            ..default()
        },
        image: UiImage{
            texture: assets.load(SQUARE_FRAME_PATH),
            ..default()
        },
        ..default()
    }, EnergyAddFrame, Name::new("Energy_Add_Frame"));
    commands.spawn(energy_add_frame);

}

fn spawn_message_frame(
    mut commands: Commands,
    assets: Res<AssetServer>
)
{
    let message_frame = (ImageBundle{
        style: Style{
            align_self: AlignSelf::Start,
            justify_self: JustifySelf::Center,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            justify_items: JustifyItems::Center,
            align_items: AlignItems::Center,
            height: Val::Vw(FRAME_SIZE/2.0),
            width: Val::Vw(FRAME_SIZE*6.0),
            ..default()
        },
        image: UiImage{
            texture: assets.load(LITTLE_RECTANGLE_FRAME_PATH),
            ..default()
        },
        ..default()
    }, MessageFrame, Name::new("Message_Frame"));
    commands.spawn(message_frame);
}

fn spawn_map_frame(
    mut commands: Commands,
    assets: Res<AssetServer>
)
{
    let map_frame = (ImageBundle{
        style: Style{
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Start,
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            justify_items: JustifyItems::Center,
            align_items: AlignItems::Center,
            top: Val::Vw(0.0 - MAP_SIZE/2.0 - 2.5),
            height: Val::Vw(MAP_SIZE/3.0),
            width: Val::Vw(MAP_SIZE),
            ..default()
        },
        image: UiImage{
            texture: assets.load(BIG_RECTANGLE_FRAME_PATH),
            ..default()
        },
        ..default()
    }, Name::new("Map_Frame"));

    commands.spawn(map_frame).with_children(|parent|{
        parent.spawn((TextBundle{
            text: Text::from_section(
                format!("ROBOT VIEW"),
                TextStyle{
                    font_size: 40.0,
                    color: Color::TEAL,
                    ..default()
                }),
            ..default()

        }, Name::new("Map_write")));
    });

    let initial_map_frame = (ImageBundle{
        style: Style{
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Start,
            height: Val::Vw(MAP_SIZE),
            width: Val::Vw(MAP_SIZE),
            ..default()
        },
        image: UiImage{
            texture: assets.load("tile_texture/none.png"),
            ..default()
        },
        ..default()
    }, Name::new("Initial_Map_Frame"));
    commands.spawn(initial_map_frame);
}
