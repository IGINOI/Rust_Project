use std::io;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_third_person_camera::*;
use robotics_lib::runner::{Robot, Runner};
use robotics_lib::world::world_generator::Generator;
use worldgen_unwrap::public::WorldgeneratorUnwrap;

use world_gen::WorldPlugin;
use camera::CameraPlugin;
use player_gen::PlayerPlugin;
use read_events::ReadEventPlugin;
use frame_gen::SpawnFramePlugin;
use runner::MyRobot;
use crate::runner::RobotAttributes;

mod world_gen;
mod camera;
mod player_gen;
mod read_events;
mod frame_gen;
mod runner;

//definition of some constants usefull to parametrize some values
pub const WORLD_PATH: &str = "assets/worlds/world_21_c.bin";
pub const TICK_DURATION: f32 = 1.0;
pub const SQUARE_FRAME_PATH: &str = "frames/square_frame.png";
pub const BIG_RECTANGLE_FRAME_PATH: &str = "frames/big_rectangular_frame.png";
pub const LITTLE_RECTANGLE_FRAME_PATH: &str = "frames/little_rectangular_frame.png";
pub const FRAME_SIZE: f32 = 5.0;
pub const MAP_SIZE: f32 = 15.0;

fn main()
{
    //Decide whether to start the GUI or the World_Generator to create a new world
    println!("Do you want to start the simulation (1) or do you want to build a new world (2)?");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).expect("Failed to read the line");

    if user_input.trim() == String::from("1"){
        //Spawn the thread in which there will be the Runner logic
        thread::spawn(||{
            let mut runner = Runner::new(Box::new(MyRobot(Robot::new(), RobotAttributes::new())), &mut WorldgeneratorUnwrap::init(false, Some(PathBuf::from(WORLD_PATH)))).unwrap();
            let mut tick_number = 0;
            thread::sleep(Duration::from_secs(5));
            loop{
                tick_number += 1;
                println!("tick {}",tick_number);
                runner.game_tick().expect("Tick error");
                thread::sleep(Duration::from_secs_f32(TICK_DURATION));
            }
        });
        //Start the GUI outside the loop
        start_gui();
    }
    else if user_input.trim() == String::from("2"){
        gen_new_world();
    }
}


fn start_gui()
{
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin{
                    primary_window: Some(
                        Window{
                            title: "Robot".into(),
                            resizable:true,
                            mode: WindowMode::Fullscreen,
                            ..default()
                        }
                    ),
                    ..default()})
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