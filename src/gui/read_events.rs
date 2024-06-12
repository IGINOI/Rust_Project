use std::path::PathBuf;
use bevy::app::App;
use bevy::prelude::*;
use bevy_extern_events::{ExternEvent, ExternEventsPlugin};
use robotics_lib::world::environmental_conditions::{DayTime, EnvironmentalConditions, WeatherType};
use robotics_lib::world::tile::{Content, Tile, TileType};
use robotics_lib::world::world_generator::Generator;
use worldgen_unwrap::public::WorldgeneratorUnwrap;
use crate::gui::player_gen::{Player2d, Player3d};
use crate::gui::frame_gen::{BackpackFrame, EnergyTotFrame, EnergyAddFrame, EnergyRemFrame, WeatherFrame, TimeFrame, MessageFrame, MapFrame};
use crate::gui::world_gen::{ContentBlock, Light, TileBlock};
use crate::gui::gui_test::{FRAME_SIZE, MAP_SIZE, SQUARE_FRAME_PATH, WORLD_PATH};

pub struct ReadEventPlugin;

impl Plugin for ReadEventPlugin{
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ExternEventsPlugin::<ReadRobotEventType>::default())
            .add_systems(Update, robot_event_system)
            .add_plugins(ExternEventsPlugin::<ReadWorldEventType>::default())
            .add_systems(Update, world_event_system);
    }
}

//Enum containing all the events I need to manage the GUI updates
#[derive(Default, PartialEq, Debug)]
pub enum ReadRobotEventType{
    RobotMoved((usize, usize)),
    EnergyRecharged((usize,usize)),
    EnergyConsumed(usize),
    AddBackpack(Vec<Content>),
    RemoveBackpack(Vec<Content>),
    MessageLogMoved((usize,usize)),
    MessageLogAddedToBackpack((Content, usize)),
    MessageLogRemovedFromBackpack((Content, usize)),
    #[default]
    None,
}

pub fn robot_event_system(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut native_events: EventReader<ExternEvent<ReadRobotEventType>>,
    mut query_player_3d: Query<(&mut Transform, With<Player3d>)>,
    mut query_player_2d: Query<(Entity, With<Player2d>)>,
    mut query_energy_text_tot: Query<(Entity, With<EnergyTotFrame>)>,
    mut query_energy_text_rem: Query<(Entity, With<EnergyAddFrame>)>,
    mut query_energy_text_add: Query<(Entity, With<EnergyRemFrame>)>,
    mut query_backpack_frames: Query<(Entity, With<BackpackFrame>)>,
    mut query_message_frame: Query<(Entity, With<MessageFrame>)>,
)
{
    for extern_event in native_events.read(){
        match &extern_event.0 {
            ReadRobotEventType::RobotMoved(new_position) => {
                let world = WorldgeneratorUnwrap::init(false, Some(PathBuf::from(WORLD_PATH))).gen();
                let world_size = world.0.len();
                let world_blocks = world.clone().0;

                //I move the 3d player
                for mut player_transform in query_player_3d.iter_mut() {
                    let next_block_height = world_blocks.clone()[new_position.clone().0][new_position.clone().1].elevation as f32;
                    player_transform.0.translation = Vec3::new(new_position.clone().1 as f32, next_block_height + 1.0, new_position.clone().0 as f32);
                }

                //I de-spawn the 2d player
                for (player_2d, _object) in query_player_2d.iter_mut(){
                    commands.entity(player_2d).despawn();
                }

                //I re-spawn it in the new position
                let player_2d = (ImageBundle{
                    style: Style{
                        align_self: AlignSelf::Center,
                        justify_self: JustifySelf::Start,
                        left: Val::Vw((new_position.1 as f32) * (MAP_SIZE /world_size as f32)),
                        top: Val::Vw((MAP_SIZE /(world_size as f32 * 2.0)) - ((world_size as f32 /2.0)* (MAP_SIZE /world_size as f32)) + (new_position.0 as f32*(MAP_SIZE /world_size as f32))),
                        height: Val::Vw(MAP_SIZE/ world_size as f32),
                        width: Val::Vw(MAP_SIZE/world_size as f32),
                        ..default()
                    },
                    image: UiImage{
                        texture: assets.load("frames/orange.png"),
                        ..default()
                    },
                    z_index: ZIndex::Global(1),
                    ..default()
                }, Player2d, Name::new("Player_2d"));
                commands.spawn(player_2d);
            }

            ReadRobotEventType::EnergyRecharged((energy_to_add, tot_energy)) => {

                //In all the energy frame cases I only remove the children of the frames and re-spawn a different writing on it
                for (energy_tot_frame, _object) in query_energy_text_tot.iter_mut() {
                    commands.entity(energy_tot_frame).despawn_descendants();
                    commands.entity(energy_tot_frame).with_children(|parent| {
                        parent.spawn((TextBundle {
                            text: Text::from_section(
                                format!("{}", tot_energy),
                                TextStyle {
                                    font_size: 30.0,
                                    color: Color::BLUE,
                                    ..default()
                                }),
                            ..default()
                        }, Name::new("Energy_Tot_Writing")));
                    });
                }

                for (energy_add_frame, _object) in query_energy_text_rem.iter_mut() {
                    commands.entity(energy_add_frame).despawn_descendants();
                    commands.entity(energy_add_frame).with_children(|parent| {
                        parent.spawn((TextBundle {
                            text: Text::from_section(
                                format!("+{}", energy_to_add),
                                TextStyle {
                                    font_size: 30.0,
                                    color: Color::GREEN,
                                    ..default()
                                }),
                            ..default()
                        }, Name::new("Energy_Add_Writing")));
                    });
                }
            }

            ReadRobotEventType::EnergyConsumed(energy_to_remove) => {
                for (energy_rem_frame, _object) in query_energy_text_add.iter_mut() {
                    commands.entity(energy_rem_frame).despawn_descendants();
                    commands.entity(energy_rem_frame).with_children(|parent| {
                        parent.spawn((TextBundle {
                            text: Text::from_section(
                                format!("-{}", energy_to_remove),
                                TextStyle {
                                    font_size: 30.0,
                                    color: Color::RED,
                                    ..default()
                                }),
                            ..default()
                        }, Name::new("Energy_Rem_Writing")));
                    });
                }
            }

            ReadRobotEventType::AddBackpack(vec_content_or) => {
                //I de-spawn all the backpack frames
                for (backpack_frame_entity, _object) in query_backpack_frames.iter_mut(){
                    commands.entity(backpack_frame_entity).despawn_recursive();
                }
                let mut vec_content = vec_content_or.clone();

                //Here I fill with None-content the backpack-array,
                // so then I can simply spawn all the frames without caring about which has "really" something inside and which has not
                for _i in 0..20{
                    if vec_content.len()<20{
                        vec_content.push(Content::None);
                    }
                }

                //Here I spawn all the 20 frames of the backpack
                for i in 0..20 {
                    let texture_object = match vec_content[i] {
                        Content::Rock(_) => assets.load("contents/rock.png"),
                        Content::Tree(_) => assets.load("contents/tree.png"),
                        Content::Garbage(_) => assets.load("contents/garbage.png"),
                        Content::Fire => assets.load("contents/fire.png"),
                        Content::Coin(_) => assets.load("contents/coin.png"),
                        Content::Bin(_) => assets.load("contents/bin.png"),
                        Content::Crate(_) => assets.load("contents/crate.png"),
                        Content::Bank(_) => assets.load("contents/bank.png"),
                        Content::Water(_) => assets.load("contents/water.png"),
                        Content::Market(_) => assets.load("contents/market.png"),
                        Content::Fish(_) => assets.load("contents/fish.png"),
                        Content::Building => assets.load("contents/building.png"),
                        Content::Bush(_) => assets.load("contents/bush.png"),
                        Content::JollyBlock(_) => assets.load("contents/star.png"),
                        Content::Scarecrow => assets.load("contents/scarecrow.png"),
                        Content::None => assets.load("contents/none.png")
                    };

                    let backpack_frame =
                        (ImageBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                align_self: AlignSelf::End,
                                justify_self: JustifySelf::Start,
                                justify_content: JustifyContent::Center,
                                align_content: AlignContent::Center,
                                justify_items: JustifyItems::Center,
                                align_items: AlignItems::Center,
                                left: Val::Vw(i as f32 * FRAME_SIZE),
                                height: Val::Vw(FRAME_SIZE),
                                width: Val::Vw(FRAME_SIZE),
                                ..default()
                            },
                            image: UiImage {
                                texture: assets.load(SQUARE_FRAME_PATH).clone(),
                                ..default()
                            },
                            ..default()
                        }, BackpackFrame, Name::new("Backpack_Frame"));
                    commands.spawn(backpack_frame).with_children(|parent| {
                        parent.spawn((ImageBundle {
                            style: Style {
                                position_type: PositionType::Relative,
                                align_self: AlignSelf::Center,
                                justify_self: JustifySelf::Center,
                                height: Val::Vw(FRAME_SIZE-1.0),
                                width: Val::Vw(FRAME_SIZE-1.0),
                                ..default()
                            },
                            image: UiImage {
                                texture: texture_object.clone(),
                                ..default()
                            },
                            z_index: ZIndex::Global(1),
                            ..default()
                        }, Name::new("Backpack_Texture")));
                    });
                }
            }

            ReadRobotEventType::RemoveBackpack(vec_content_or) => {
                //Here I de-spawn all the backpack frame and with the same logic as the previous event I respawn them with the updated contents
                for (backpack_frame_entity, _object) in query_backpack_frames.iter_mut(){
                    commands.entity(backpack_frame_entity).despawn_recursive();
                }
                let mut vec_content = vec_content_or.clone();

                for _i in 0..20{
                    if vec_content.len()<20{
                        vec_content.push(Content::None);
                    }
                }

                for i in 0..20 {
                    let texture_object = match vec_content[i] {
                        Content::Rock(_) => assets.load("contents/rock.png"),
                        Content::Tree(_) => assets.load("contents/tree.png"),
                        Content::Garbage(_) => assets.load("contents/garbage.png"),
                        Content::Fire => assets.load("contents/fire.png"),
                        Content::Coin(_) => assets.load("contents/coin.png"),
                        Content::Bin(_) => assets.load("contents/bin.png"),
                        Content::Crate(_) => assets.load("contents/crate.png"),
                        Content::Bank(_) => assets.load("contents/bank.png"),
                        Content::Water(_) => assets.load("contents/water.png"),
                        Content::Market(_) => assets.load("contents/market.png"),
                        Content::Fish(_) => assets.load("contents/fish.png"),
                        Content::Building => assets.load("contents/building.png"),
                        Content::Bush(_) => assets.load("contents/bush.png"),
                        Content::JollyBlock(_) => assets.load("contents/star.png"),
                        Content::Scarecrow => assets.load("contents/scarecrow.png"),
                        Content::None => assets.load("contents/none.png")
                    };

                    let backpack_frame =
                        (ImageBundle {
                            style: Style {
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
                            image: UiImage {
                                texture: assets.load(SQUARE_FRAME_PATH).clone(),
                                ..default()
                            },
                            ..default()
                        }, BackpackFrame, Name::new("Backpack_Frame"));
                    commands.spawn(backpack_frame).with_children(|parent| {
                        parent.spawn((ImageBundle {
                            style: Style {
                                position_type: PositionType::Relative,
                                align_self: AlignSelf::Center,
                                justify_self: JustifySelf::Center,
                                height: Val::Vw(4.0),
                                width: Val::Vw(4.0),
                                ..default()
                            },
                            image: UiImage {
                                texture: texture_object.clone(),
                                ..default()
                            },
                            z_index: ZIndex::Global(1),
                            ..default()
                        }, Name::new("Backpack_Texture")));
                    });
                }
            }

            ReadRobotEventType::MessageLogMoved(position) => {
                //Here I only de-spawn the children since I have no need to re-swan each time the frames
                for (text_frame, _object) in query_message_frame.iter_mut() {
                    commands.entity(text_frame).despawn_descendants();
                    commands.entity(text_frame).with_children(|parent| {
                        parent.spawn((TextBundle {
                            text: Text::from_section(
                                format!("You moved to: {:?}", position),
                                TextStyle {
                                    font_size: 30.0,
                                    color: Color::TEAL,
                                    ..default()
                                }),
                            ..default()
                        }, Name::new("Message_Text")));
                    });
                }
            }

            ReadRobotEventType::MessageLogAddedToBackpack((content, quantity)) => {
                //Here I only de-spawn the children since I have no need to re-swan each time the frames
                for (text_frame, _object) in query_message_frame.iter_mut() {
                    commands.entity(text_frame).despawn_descendants();
                    commands.entity(text_frame).with_children(|parent| {
                        parent.spawn((TextBundle {
                            text: Text::from_section(
                                format!("You added {:?} {:?} to the backpack", quantity, content),
                                TextStyle {
                                    font_size: 20.0,
                                    color: Color::TEAL,
                                    ..default()
                                }),
                            ..default()
                        }, Name::new("Message_Text")));
                    });
                }
            }

            ReadRobotEventType::MessageLogRemovedFromBackpack((content, quantity)) => {
                //Here I only de-spawn the children since I have no need to re-swan each time the frames
                for (text_frame, _object) in query_message_frame.iter_mut() {
                    commands.entity(text_frame).despawn_descendants();
                    commands.entity(text_frame).with_children(|parent| {
                        parent.spawn((TextBundle {
                            text: Text::from_section(
                                format!("You removed {:?} {:?} from the backpack", quantity, content),
                                TextStyle {
                                    font_size: 20.0,
                                    color: Color::TEAL,
                                    ..default()
                                }),
                            ..default()
                        }, Name::new("Message_Text")));
                    });
                }
            }

            ReadRobotEventType::None => {}
        }
    }
}



#[derive(Default, PartialEq, Debug)]
pub enum ReadWorldEventType{
    TimeChanged(EnvironmentalConditions),
    WeatherChanged(EnvironmentalConditions),
    UpdatedTile((Tile,(usize,usize))),
    LittleMapUpdate(Vec<Vec<Option<Tile>>>),
    #[default]
    None,
}

pub fn world_event_system(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world_native_events: EventReader<ExternEvent<ReadWorldEventType>>,

    mut query_tiles_block: Query<(&mut Transform, Entity, With<TileBlock>, Without<ContentBlock>)>,
    mut query_content_tile: Query<(&mut Transform, Entity, With<ContentBlock>, Without<TileBlock>)>,
    mut query_light: Query<(Entity, With<Light>)>,
    mut query_time_frame: Query<(Entity, With<TimeFrame>)>,
    mut query_weather_frame: Query<(Entity, With<WeatherFrame>)>,
    mut query_map_frame: Query<(Entity, With<MapFrame>)>
)
{
    for world_extern_event in world_native_events.read(){
        match &world_extern_event.0{

            ReadWorldEventType::WeatherChanged(new_condition) => {

                //I look for the new weather condition and I spawn the new weather frame
                let new_weather_condition = new_condition.get_weather_condition();
                let weather_texture = match new_weather_condition {
                    WeatherType::Sunny => assets.load("time_weather_texture/sunny.png"),
                    WeatherType::Rainy => assets.load("time_weather_texture/rain.png"),
                    WeatherType::Foggy => assets.load("time_weather_texture/fog.png"),
                    WeatherType::TropicalMonsoon => assets.load("time_weather_texture/monsoon.png"),
                    WeatherType::TrentinoSnow => assets.load("time_weather_texture/snow.png")
                };

                for (weather_frame, _object) in query_weather_frame.iter_mut() {
                    commands.entity(weather_frame).despawn_descendants();
                    commands.entity(weather_frame).with_children(|parent| {
                        parent.spawn((ImageBundle {
                            style: Style {
                                position_type: PositionType::Relative,
                                align_self: AlignSelf::Center,
                                justify_self: JustifySelf::Center,
                                height: Val::Vw(FRAME_SIZE - 1.0),
                                width: Val::Vw(FRAME_SIZE - 1.0),
                                ..default()
                            },
                            image: UiImage {
                                texture: weather_texture.clone(),
                                ..default()
                            },
                            z_index: ZIndex::Global(1),
                            ..default()
                        }, Name::new("Weather_Texture")));
                    });

                }
            }

            ReadWorldEventType::TimeChanged(new_condition) => {

                //I look for the new day time
                let new_time_conditions = new_condition.get_time_of_day();
                let texture_time = match new_time_conditions {
                    DayTime::Morning => assets.load("time_weather_texture/morning.png"),
                    DayTime::Afternoon => assets.load("time_weather_texture/evening.png"),
                    DayTime::Night => assets.load("time_weather_texture/night.png")
                };

                //I de-spawn the old image and I spawn the new one
                for (time_frame, _object) in query_time_frame.iter_mut() {
                    commands.entity(time_frame).despawn_descendants();
                    commands.entity(time_frame).with_children(|parent| {
                        parent.spawn((ImageBundle {
                            style: Style {
                                position_type: PositionType::Relative,
                                align_self: AlignSelf::Center,
                                justify_self: JustifySelf::Center,
                                height: Val::Vw(FRAME_SIZE - 1.0),
                                width: Val::Vw(FRAME_SIZE - 1.0),
                                ..default()
                            },
                            image: UiImage {
                                texture: texture_time.clone(),
                                ..default()
                            },
                            z_index: ZIndex::Global(1),
                            ..default()
                        }, Name::new("Time_Texture")));
                    });
                }

                for (light, _object) in query_light.iter_mut(){
                    commands.entity(light).despawn();
                }

                let position = WorldgeneratorUnwrap::init(false, Some(PathBuf::from(WORLD_PATH))).gen().0.len() as f32 /2.0;
                let light_position = match new_time_conditions {
                    DayTime::Morning => (position , 100.0, position),
                    DayTime::Afternoon => (position , 100.0, position),
                    DayTime::Night => (position , 100.0, position)
                };
                let light_color = match new_time_conditions {
                    DayTime::Morning => Color::rgb(1.0, 1.0, 1.0),
                    DayTime::Afternoon => Color::rgb(1.0,  0.6, 0.4),
                    DayTime::Night => Color::rgb(0.3,  0.3,  0.6)
                };

                let light = (
                    PointLightBundle{
                        point_light: PointLight{
                            intensity: 300000.0,
                            shadows_enabled: true,
                            range: 1000.0,
                            radius: 100.0,
                            color: light_color,
                            ..default()
                        },
                        transform: Transform::from_xyz(light_position.0, light_position.1, light_position.2),
                        ..default()
                    },
                    Light,
                    Name::new("Light")
                );
                commands.spawn(light);
            }

            ReadWorldEventType::UpdatedTile((tile_block, position)) => {

                //Here I look for blocks that are turned into street in order to replace them in the 3d map
                if tile_block.tile_type == TileType::Street{
                    for tile_block in query_tiles_block.iter_mut(){
                        if (tile_block.0.translation.z as usize, tile_block.0.translation.x as usize) == *position{
                            commands.entity(tile_block.1).despawn();
                            let tile_block = (
                                PbrBundle {
                                    mesh: meshes.add(Mesh::from(shape::Cube::new(1.0))),
                                    material: materials.add(assets.load("tile_texture/road.png").clone().into()),
                                    transform: Transform::from_xyz(tile_block.0.translation.x, tile_block.0.translation.y, tile_block.0.translation.z),
                                    ..default()
                                },
                                TileBlock,
                                Name::new("Tile_Block")
                            );
                            commands.spawn(tile_block);
                        }
                    }
                }

                //Here I de-spawn the block representing the content tile that correspond to the given position
                for tile in query_content_tile.iter_mut(){
                    if (tile.0.translation.z as usize, tile.0.translation.x as usize) == *position{
                        commands.entity(tile.1).despawn();
                    }
                }

                //If the content of the block is different from None then I spawn the right tile content in the given position,
                // otherwise I do nothing since it means that the content of the tile was simply removed.
                if tile_block.content != Content::None{
                    let content_type = match tile_block.content {
                        Content::Rock(_) => assets.load("contents/rock.png"),
                        Content::Tree(_) => assets.load("contents/tree.png"),
                        Content::Garbage(_) => assets.load("contents/garbage.png"),
                        Content::Fire => assets.load("contents/fire.png"),
                        Content::Coin(_) => assets.load("contents/coin.png"),
                        Content::Bin(_) => assets.load("contents/bin.png"),
                        Content::Crate(_) => assets.load("contents/crate.png"),
                        Content::Bank(_) => assets.load("contents/bank.png"),
                        Content::Water(_) => assets.load("contents/water.png"),
                        Content::Market(_) => assets.load("contents/market.png"),
                        Content::Fish(_) => assets.load("contents/fish.png"),
                        Content::Building => assets.load("contents/building.png"),
                        Content::Bush(_) => assets.load("contents/bush.png"),
                        Content::JollyBlock(_) => assets.load("contents/star.png"),
                        Content::Scarecrow => assets.load("contents/scarecrow.png"),
                        Content::None => assets.load("contents/none.png")
                    };
                    let tile_content = (
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Cube::new(0.3))),
                            material: materials.add(content_type.clone().into()),
                            transform: Transform::from_xyz(position.clone().1 as f32, tile_block.elevation as f32 + 0.36, position.clone().0 as f32),
                            ..default()
                        },
                        ContentBlock,
                        Name::new("Content_Block")
                    );
                    commands.spawn(tile_content);
                }
            }

            ReadWorldEventType::LittleMapUpdate(world) => {
                //Here I de-spawn all the frames of the little map
                for (map_frame, _object1) in query_map_frame.iter_mut(){
                    commands.entity(map_frame).despawn();
                }

                //Here I spawn the updated one
                for row in 0..world.len(){
                    for col in 0..world.len(){
                        match world[row][col].clone() {
                            None => {
                                commands.spawn((
                                    ImageBundle{
                                        style: Style {
                                            align_self: AlignSelf::Center,
                                            justify_self: JustifySelf::Start,
                                            left: Val::Vw((col as f32) * (MAP_SIZE /world.len() as f32)),
                                            top: Val::Vw((MAP_SIZE /(world.len() as f32 * 2.0)) - ((world.len() as f32 /2.0)* (MAP_SIZE /world.len() as f32)) + (row as f32*(MAP_SIZE /world.len() as f32))),
                                            height: Val::Vw(MAP_SIZE /world.len() as f32),
                                            width: Val::Vw(MAP_SIZE /world.len() as f32),
                                            ..default()
                                        },
                                        image: UiImage{
                                            texture: assets.load("tile_texture/none.png").clone(),
                                            ..default()
                                        },
                                        ..default()
                                    },MapFrame, Name::new("MapFrame"))
                                );
                            }
                            Some(tile) => {
                                let tile_type = match tile.tile_type {
                                    TileType::DeepWater => assets.load("tile_texture/deepwater.png"),
                                    TileType::ShallowWater => assets.load("tile_texture/shallow_water.png"),
                                    TileType::Sand => assets.load("tile_texture/sand.png"),
                                    TileType::Grass => assets.load("tile_texture/grass.png"),
                                    TileType::Street => assets.load("tile_texture/road.png"),
                                    TileType::Hill => assets.load("tile_texture/hill.png"),
                                    TileType::Mountain => assets.load("tile_texture/mountain.png"),
                                    TileType::Snow => assets.load("tile_texture/snow.png"),
                                    TileType::Lava => assets.load("tile_texture/lava.png"),
                                    TileType::Teleport(_) => assets.load("tile_texture/teleport.png"),
                                    TileType::Wall => assets.load("tile_texture/wall.png"),
                                };
                                commands.spawn((
                                    ImageBundle{
                                        style: Style {
                                            align_self: AlignSelf::Center,
                                            justify_self: JustifySelf::Start,
                                            left: Val::Vw((col as f32) * (MAP_SIZE /world.len() as f32)),
                                            top: Val::Vw((MAP_SIZE /(world.len() as f32 * 2.0)) - ((world.len() as f32 /2.0)* (MAP_SIZE /world.len() as f32)) + (row as f32*(MAP_SIZE /world.len() as f32))),
                                            height: Val::Vw(MAP_SIZE /world.len() as f32),
                                            width: Val::Vw(MAP_SIZE /world.len() as f32),
                                            ..default()
                                        },
                                        image: UiImage{
                                            texture: tile_type.clone(),
                                            ..default()
                                        },
                                        ..default()
                                    },MapFrame, Name::new("MapFrame"))
                                );
                            }
                        }
                    }
                }
            }

            ReadWorldEventType::None => {}
        }
    }
}