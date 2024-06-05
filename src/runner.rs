use std::thread::sleep;
use std::time::Duration;
use bevy::prelude::Component;
use bevy_extern_events::queue_event;
use rand::Rng;
use robotics_lib::energy::Energy;
use robotics_lib::event::events::Event;
use robotics_lib::interface::{destroy, Direction, go, put, robot_map};
use robotics_lib::runner::{Robot, Runnable};
use robotics_lib::runner::backpack::BackPack;
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::world::tile::Content::Rock;
use crate::read_events::{ReadRobotEventType, ReadWorldEventType};
use crate::TICK_DURATION;


use std::collections::HashMap;
use core::mem::Discriminant;
use core::ops::Range;
use std::error::Error;
use another_one_bytes_the_dust_tile_resource_mapper_tool::coordinates::map_coordinate::MapCoordinate;
use another_one_bytes_the_dust_tile_resource_mapper_tool::tool::tile_mapper::TileMapper;
use std::vec::Vec;
use bob_lib::tracker::GoalTracker;
use robotics_lib::world::tile::Content;
use crate::better_gps::{crazy_noisy_bizarre_gps};

// use tRust_us_Path_finding::tools::gps;
// use tRust_us_Path_finding::tools::gps::{Command, Goal, gps};
// use tile_resource_mapper_tool::tool::tile_resource_mapper_tool{ContentQuantity, TileMapper};

#[derive(Debug)]
pub enum RobotState{
    Decision,
    Deciding,
    GoingSpiral,
    GoingToGoal,
    Default
}

pub struct RobotAttributes{
    //state of the robot
    pub state: RobotState,

    // always saved spiral movement
    pub spiral : Vec<Direction>,

    // current vector of direction of the robot
    pub directions: Vec<Direction>,

    //tile-resource-mapper tool mapper
    pub mappertool: TileMapper,

    //tile-resource-mapper tool map
    pub map: Option<HashMap<Discriminant<Content>, Vec<(MapCoordinate, (Option<usize>, Option<Range<usize>>))>>>,

    //goal tracker
    pub tracker: GoalTracker,

    //previous direction
    pub prev_dir: Direction,

}
impl RobotAttributes{
    pub fn new() -> RobotAttributes{

        let mut state = RobotState::GoingSpiral;
        // creates a vector of directions
        let mut mov = 200;
        let spiral = spiral_directions(mov);
        let mut directions = spiral_directions(mov);

        let mut mappertool = TileMapper {};
        let mut map = None;

        let mut tracker = GoalTracker::new();
        let mut prev_dir = Direction::Right;

        // return
        RobotAttributes{state, spiral, directions, mappertool, map, tracker, prev_dir}
    }
}
pub struct MyRobot(pub Robot, pub RobotAttributes);

impl Runnable for MyRobot {
    fn process_tick(&mut self, world: &mut robotics_lib::world::World){
        // println!("{:?}", where_am_i(self, world));
        println!("{:?}", self.1.directions);
        // println!("{:?}", self.1.prev_dir);
        println!("{:?}", self.1.state);
        // the state of the robot
        match self.1.state{
            RobotState::Decision => {
                self.1.map = TileMapper::collection(world);
                println!("{:?}", self.1.mappertool.find_closest(world, self, Content::Tree(0)));

                let closest = self.1.mappertool.find_closest(world, self, Content::Tree(0));

                let coords = to_coords(closest);
                match coords {
                    None => {println!("qualcosa è andato storto 1");}
                    Some(a) => {
                        let table = crazy_noisy_bizarre_gps(self, a, world);
                        match table{
                            None => {println!("qualcosa è andato storto 2");}
                            Some(b) => {
                                for elem in &b{
                                    println!("{:?}", elem)
                                }
                                let direction_vec = b;
                                println!("{:?}", direction_vec);
                                self.1.directions = direction_vec;
                                self.1.state = RobotState::GoingToGoal;
                            }
                        }
                    }
                }
            }
            RobotState::Deciding => {


                self.1.state = RobotState::GoingSpiral;
            }
            RobotState::GoingSpiral => {

                if !self.1.directions.is_empty(){
                    // println!("prev tile: {:?}", where_am_i(self, world).1);
                    // goes spiral
                    let momdir = self.1.directions.pop();
                    let _ = go(self, world, momdir.unwrap());
                    // println!("succ tile: {:?}", where_am_i(self, world).1);
                } else {
                    self.1.state = RobotState::Decision;
                    // find the path to the object
                }
            }
            RobotState::GoingToGoal => {
                // println!("ciao");
                if self.1.directions.len() != 1{
                    let momdir = self.1.directions.pop();
                    match momdir {
                        None => {
                            self.1.directions.push(self.1.prev_dir.clone());
                            self.1.directions.push(opposite_dir(self.1.prev_dir.clone()));
                        }
                        Some(a) => {
                            self.1.prev_dir = a.clone();
                            let _ = go(self, world, a);
                        }
                    }

                }else {
                    let momdir = self.1.directions.pop();
                    self.1.prev_dir = momdir.unwrap().clone();
                    let _ = destroy(self, world, self.1.prev_dir.clone());
                    self.1.state = RobotState::Decision;
                }
            }
            //     RobotState::Default => {}

            _ => {}
        }
        queue_event(ReadWorldEventType::LittleMapUpdate(robot_map(world).unwrap()));
    }
    fn handle_event(&mut self, event: Event) {
        //here based on the events triggered by the common crate, i trigger my private events
        match event{
            Event::Ready => {
                println!("We are super ready");
            }

            Event::Terminated => {
                println!("We are done");
            }

            Event::TimeChanged(new_conditions) => {
                queue_event(ReadWorldEventType::TimeChanged(new_conditions.clone()));
                queue_event(ReadWorldEventType::WeatherChanged(new_conditions));
            }

            Event::DayChanged(new_conditions) => {
                queue_event(ReadWorldEventType::TimeChanged(new_conditions.clone()));
                queue_event(ReadWorldEventType::WeatherChanged(new_conditions));
            }

            Event::EnergyRecharged(energy_recharged) => {
                queue_event(ReadRobotEventType::EnergyRecharged((energy_recharged, self.get_energy().get_energy_level())));
            }

            Event::EnergyConsumed(energy_consumed) => {
                queue_event(ReadRobotEventType::EnergyConsumed(energy_consumed));
            }

            Event::Moved(_tile, position) => {
                queue_event(ReadRobotEventType::RobotMoved((position.0,position.1)));
                queue_event(ReadRobotEventType::MessageLogMoved(position));
            }

            Event::TileContentUpdated(tile, position) => {
                queue_event(ReadWorldEventType::UpdatedTile((tile, (position.0, position.1))));
            }

            Event::AddedToBackpack(content, quantity) => {
                //In this case a create a Vec containing the contents of the backpack, that I will pass in a event
                let mut vec_content = vec![];
                for tile_content in self.get_backpack().get_contents() {
                    if *tile_content.1 != 0 {
                        for _ in 0..*tile_content.1 {
                            vec_content.push(tile_content.0.clone());
                        }
                    }
                }
                queue_event(ReadRobotEventType::AddBackpack(vec_content));
                if quantity != 0 {
                queue_event(ReadRobotEventType::MessageLogAddedToBackpack((content, quantity)));
                }
            }

            Event::RemovedFromBackpack(content, quantity) => {
                //Here we have the same reasoning as in the previous event
                let mut vec_content = vec![];
                for tile_content in self.get_backpack().get_contents(){
                    if *tile_content.1 != 0{
                        for _ in 0..*tile_content.1{
                            vec_content.push(tile_content.0.clone());
                        }
                    }
                }
                queue_event(ReadRobotEventType::RemoveBackpack(vec_content));
                queue_event(ReadRobotEventType::MessageLogRemovedFromBackpack((content, quantity)));
            }
        }
    }
    fn get_energy(&self) -> &Energy {
        &self.0.energy
    }
    fn get_energy_mut(&mut self) -> &mut Energy {
        &mut self.0.energy
    }
    fn get_coordinate(&self) -> &Coordinate {
        &self.0.coordinate
    }
    fn get_coordinate_mut(&mut self) -> &mut Coordinate {
        &mut self.0.coordinate
    }
    fn get_backpack(&self) -> &BackPack {
        &self.0.backpack
    }
    fn get_backpack_mut(&mut self) -> &mut BackPack {
        &mut self.0.backpack
    }
}

fn choose_random_direction() -> Direction{
    let n = rand::thread_rng().gen_range(0..4);
    match n {
        0 => Direction::Right,
        1 => Direction::Left,
        2 => Direction::Up,
        _ => Direction::Down
    }
}

pub fn spiral_directions(n: usize) -> Vec<Direction> {
    let mut directions = Vec::with_capacity(n);
    let mut steps = 1;
    let mut total_directions = 0;

    loop {
        for _ in 0..steps {
            if total_directions >= n {
                directions.reverse();
                return directions;
            }
            directions.push(Direction::Right);
            total_directions += 1;
        }

        for _ in 0..steps {
            if total_directions >= n {
                directions.reverse();
                return directions;
            }
            directions.push(Direction::Down);
            total_directions += 1;
        }

        steps += 1;

        for _ in 0..steps {
            if total_directions >= n {
                directions.reverse();
                return directions;
            }
            directions.push(Direction::Left);
            total_directions += 1;
        }

        for _ in 0..steps {
            if total_directions >= n {
                directions.reverse();
                return directions;
            }
            directions.push(Direction::Up);
            total_directions += 1;
        }

        steps += 1;
    }
}

pub fn to_coords(obj: Result<MapCoordinate, Box<dyn Error>>) -> Option<(usize,usize)>{
    match obj {
        Ok(a) => {
            let y = a.get_width();
            let x = a.get_height();
            return Some((x,y));
        }
        Err(b) => {return None;}
    }
}



//
// pub fn gps_to_dir(commands_and_cost: Option<(Vec<Command>, usize)>) -> Vec<Direction>{
//     println!("funzione");
//     let mut vec = vec![];
//     match commands_and_cost {
//
//         None => {
//             println!("none");
//
//         }
//         Some((a, b)) => {
//             println!("some");
//             for x in a {
//                 match x {
//                     Control(b) => {vec.push(b)}
//                     Destroy(_) => {}
//                 }
//             }
//
//             println!("{:?}", vec);
//         }
//     }
//     vec.reverse();
//     vec
// }
//
pub fn opposite_dir(dir: Direction) -> Direction{
    match dir {
        Direction::Up => {return Direction::Down;}
        Direction::Down => {return Direction::Up;}
        Direction::Left => {return Direction::Right;}
        Direction::Right => {return Direction::Left;}
    }
}