use std::path::PathBuf;
use bevy::prelude::*;
use robotics_lib::world::tile::{Content, TileType};
use robotics_lib::world::world_generator::Generator;
use worldgen_unwrap::public::WorldgeneratorUnwrap;
use crate::{WORLD_PATH};

pub struct WorldPlugin;

impl Plugin for WorldPlugin{
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreStartup, (spawn_light, spawn_world));
    }
}

#[derive(Component)]
pub struct Light;
#[derive(Component)]
pub struct TileBlock;
#[derive(Component)]
pub struct ContentBlock;

//Here I spawn the lights for the 3d map
fn spawn_light(
    mut commands: Commands
)
{
    let position = WorldgeneratorUnwrap::init(false, Some(PathBuf::from(WORLD_PATH))).gen().0.len() as f32 /2.0;
    let light = (
        PointLightBundle{
            point_light: PointLight{
                intensity: 300000.0,
                shadows_enabled: true,
                range: 1000.0,
                radius: 100.0,
                color: Color::rgb(0.4, 0.4, 0.4),
                ..default()
            },
            transform: Transform::from_xyz(position, 100.0, position),
            ..default()
        },
        //Light,
        Name::new("Light")
    );
    commands.spawn(light);
}

//Here I spawn all the blocks and the contents of the 3d map
fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>)
{
    let world = WorldgeneratorUnwrap::init(false, Some(PathBuf::from(WORLD_PATH))).gen();
    let world_blocks = world.0;

    let mut raw_position = 0.0;
    for raw in world_blocks {
        let mut column_position = 0.0;
        for col in raw{

            let tile_type = match col.tile_type {
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

            for i in 0..20{
                if col.elevation as f32 - (i as f32) >= 0.0 { //in order to spawn less blocks
                    let colored_block = (
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Cube::new(1.0))),
                            material: materials.add(tile_type.clone().into()),
                            transform: Transform::from_xyz(column_position, col.elevation as f32 - (i as f32), raw_position),
                            ..default()
                        },
                        TileBlock,
                        Name::new("Tile_Block")
                    );
                    commands.spawn(colored_block);
                }
            }

            let content_type = match col.content {
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
            if col.content != Content::None {
                let tile_content = (
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube::new(0.3))),
                        material: materials.add(content_type.clone().into()),
                        transform: Transform::from_xyz(column_position, col.elevation as f32 + 0.36, raw_position),
                        ..default()
                    },
                    ContentBlock,
                    Name::new("Content_Block")
                );
                commands.spawn(tile_content);
            }
            column_position += 1.0;
        }
        raw_position += 1.0;
    }

}

