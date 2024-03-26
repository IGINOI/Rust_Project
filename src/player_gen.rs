use std::path::PathBuf;
use bevy_third_person_camera::*;
use bevy::prelude::*;
use robotics_lib::world::world_generator::Generator;
use worldgen_unwrap::public::WorldgeneratorUnwrap;
use crate::{MAP_SIZE, WORLD_PATH};

#[derive(Component)]
pub struct Player3d;
#[derive(Component)]
pub struct Player2d;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin{
    fn build(&self, app: &mut App){
        app
            .add_systems(Startup, spawn_3d_player)
            .add_systems(Startup, spawn_2d_player);
    }
}

//I spawn the 3d player at his initial position on the map.
fn spawn_3d_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
)
{
    let world = WorldgeneratorUnwrap::init(false, Some(PathBuf::from(WORLD_PATH))).gen();
    let starting_point = world.1;
    let starting_height = world.0.clone()[starting_point.0][starting_point.1].elevation;

    let player_3d = (
        PbrBundle{
            mesh: meshes.add(Mesh::from(shape::Cube::new(1.0))),
            material: materials.add(Color::RED.into()),
            transform: Transform::from_xyz(starting_point.0 as f32 , starting_height as f32 + 1.0, starting_point.1 as f32),
            ..default()
        },
        Player3d,
        //we assign to the camera the target player to follow
        ThirdPersonCameraTarget,
        Name::new("Player_3d")
    );
    commands.spawn(player_3d);
}

//I spawn the 2d player for the robot_map_view
fn spawn_2d_player(
    mut commands: Commands,
    assets: Res<AssetServer>
)
{
    let world = WorldgeneratorUnwrap::init(false, Some(PathBuf::from(WORLD_PATH))).gen();
    let world_size = world.0.len();
    let position = world.1;

    let player_2d = (ImageBundle{
        style: Style{
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Start,
            left: Val::Vw((position.0 as f32) * (MAP_SIZE /world_size as f32)),
            top: Val::Vw((MAP_SIZE /(world_size as f32 * 2.0)) - ((world_size as f32 /2.0)* (MAP_SIZE /world_size as f32)) + (position.1 as f32*(MAP_SIZE /world_size as f32))),
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