use bevy_extern_events::queue_event;
use rand::Rng;
use robotics_lib::energy::Energy;
use robotics_lib::event::events::Event;
use robotics_lib::interface::{go, put, destroy, Direction, robot_map};
use robotics_lib::runner::{Robot, Runnable};
use robotics_lib::runner::backpack::BackPack;
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::world::tile::Content;
use crate::read_events::{ReadRobotEventType, ReadWorldEventType};

use std::vec::Vec;
use another_one_bytes_the_dust_tile_resource_mapper_tool::tool::tile_mapper::TileMapper;
use another_one_bytes_the_dust_tile_resource_mapper_tool::coordinates::map_coordinate::MapCoordinate;
use crate::lumberjack::{crazy_noisy_bizarre_gps};
use std::error::Error;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub enum RobotState{
    Decision,
    Deciding,
    GoingSpiral,
    GoingToTree,
    GoingToSell,
    Default
}

pub struct RobotAttributes{
    //here we have the state of the robot
    pub state: RobotState,

    //always saved spiral movement
    pub spiral : Vec<Direction>,

    // current vector of direction of the robot
    pub directions: Vec<Direction>,

    pub mappertool: TileMapper,

    //previous direction
    pub prev_dir: Direction
}

impl RobotAttributes{
    pub fn new() -> RobotAttributes{

        //I want the robot to start going spiral
        let mut state = RobotState::GoingSpiral;

        //I set the number of movement I want the robot to do
        let mut mov = 50;

        let mut mappertool = TileMapper {};

        //I compute the steps I have to do for spiraling
        let spiral = spiral_directions(mov);
        let mut directions = Vec::new();

        let mut prev_dir = Direction::Right;

        RobotAttributes{state, directions, mappertool, spiral, prev_dir}

    }
}


pub struct MyRobot(pub Robot, pub RobotAttributes);

impl Runnable for MyRobot {
    fn process_tick(&mut self, world: &mut robotics_lib::world::World){
        //println!("THE BACKPPACK CONTAINS: {:?}", self.0.backpack);

        println!("THE TREE IN THE BACKPACK ARE: {:?}", self.0.backpack.get_contents().get(&Content::Tree(0)).unwrap());
        println!("THE COINS IN THE BACKPACK ARE: {:?}", self.0.backpack.get_contents().get(&Content::Coin(0)).unwrap());


        match self.1.state{
            RobotState::Decision =>{
                //If backpack is full go to sell otherwise keep collecting trees
                if *self.0.backpack.get_contents().get(&Content::Coin(0)).unwrap() == 20{
                    println!("I AM SUPER RICH NOW: MISSION ACCOMPLISHED");
                    self.1.state = RobotState::Default;
                }else if *self.0.backpack.get_contents().get(&Content::Tree(0)).unwrap() <= 9 {
                    let closest = self.1.mappertool.find_closest(world, self, Content::Tree(0));
                    let coords_to_closest = to_coords(closest);

                    match coords_to_closest {
                        None => {
                            println!("The function coords_to_closest gi");
                        }
                        Some(coord_to_closest) => {
                            let direction_vector = crazy_noisy_bizarre_gps(self, coord_to_closest, world);
                            match direction_vector {
                                None => {
                                    println!("qualcosa è andato storto 2");
                                }
                                Some(vector_of_directions) => {
                                    self.1.directions = vector_of_directions;
                                    self.1.state = RobotState::GoingToTree;
                                }
                            }
                        }
                    }
                }else{
                    let closest = self.1.mappertool.find_closest(world, self, Content::Market(0));
                    let coords_to_closest = to_coords(closest);

                    match coords_to_closest {
                        None => {
                            println!("The function coords_to_closest gi");
                        }
                        Some(coord_to_closest) => {
                            let direction_vector = crazy_noisy_bizarre_gps(self, coord_to_closest, world);
                            match direction_vector{
                                None => {
                                    println!("qualcosa è andato storto 2");
                                }
                                Some(vector_of_directions) =>{
                                    self.1.directions = vector_of_directions;
                                    self.1.state = RobotState::GoingToSell;
                                }
                            }
                        }
                    }
                }

            }

            RobotState::Deciding =>{

            }

            RobotState::GoingSpiral => {
                if !self.1.spiral.is_empty(){
                    let next_direction = self.1.spiral.pop();
                    let _ = go(self, world, next_direction.unwrap());
                } else {
                    self.1.state = RobotState::Decision;
                }
            }

            RobotState::GoingToTree => {
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

                } else {
                    let momdir = self.1.directions.pop();
                    self.1.prev_dir = momdir.unwrap().clone();
                    let _ = destroy(self, world, self.1.prev_dir.clone());

                    self.1.state = RobotState::Decision;
                }
            }

            RobotState::GoingToSell => {
                if self.1.directions.len() != 1 {
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
                } else {
                    let momdir = self.1.directions.pop();
                    self.1.prev_dir = momdir.unwrap().clone();

                    let _ = put(self, world, Content::Tree(0), 10, self.1.prev_dir.clone());

                    self.1.state = RobotState::Decision;

                }

            }

            RobotState::Default => {}
        }



        //I use this event in order to being able to update the little map with the robot view
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
                println!("ADDED {:?} TO THE BACKPACK", vec_content.clone());
                thread::sleep(Duration::from_secs_f32(0.5));
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

pub fn spiral_directions(n: usize) -> Vec<Direction> {
    let mut directions = Vec::with_capacity(n);
    let mut steps = 3;
    let mut total_directions = 0;
    directions.push(Direction::Right);
    directions.push(Direction::Left);
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

        steps += 3;

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

        steps += 3;
    }
}

pub fn opposite_dir(dir: Direction) -> Direction{
    match dir {
        Direction::Up => {return Direction::Down;}
        Direction::Down => {return Direction::Up;}
        Direction::Left => {return Direction::Right;}
        Direction::Right => {return Direction::Left;}
    }
}