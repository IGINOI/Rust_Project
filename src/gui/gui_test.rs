use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_third_person_camera::*;
use robotics_lib::world::world_generator::Generator;
use worldgen_unwrap::public::WorldgeneratorUnwrap;

use crate::gui::world_gen::WorldPlugin;
use crate::gui::camera::CameraPlugin;
use crate::gui::player_gen::PlayerPlugin;
use crate::gui::read_events::ReadEventPlugin;
use crate::gui::frame_gen::SpawnFramePlugin;
pub const WORLD_PATH: &str = "src/worlds/world_6_45.bin";
pub const TICK_DURATION: f32 = 1.0;
pub const SQUARE_FRAME_PATH: &str = "frames/square_frame.png";
pub const BIG_RECTANGLE_FRAME_PATH: &str = "frames/big_rectangular_frame.png";
pub const LITTLE_RECTANGLE_FRAME_PATH: &str = "frames/little_rectangular_frame.png";
pub const FRAME_SIZE: f32 = 5.0;
pub const MAP_SIZE: f32 = 15.0;

pub fn start_gui()
{
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin{
                    primary_window: Some(
                        Window{
                            title: "Robot".into(),
                            resizable:true,
                            mode: WindowMode::BorderlessFullscreen,
                            ..default()
                        }
                    ),
                    ..default()})
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    filter: "off".into(),
                })
                .build(),
            WorldPlugin,
            CameraPlugin,
            ThirdPersonCameraPlugin,
            PlayerPlugin,
            SpawnFramePlugin,
            ReadEventPlugin,
        ))
        .run();
}

pub fn gen_new_world()
{
    let _ = WorldgeneratorUnwrap::init(true, None).gen();
}
