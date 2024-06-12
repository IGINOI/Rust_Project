use std::thread::sleep;
use std::time::Duration;
use bevy::prelude::Component;
use bevy_extern_events::queue_event;
use rand::Rng;
use robotics_lib::energy::Energy;
use robotics_lib::event::events::Event;
use robotics_lib::interface::{destroy, Direction, go, put, robot_map, where_am_i};
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
use std::fs::{File, read_to_string};
use std::io::BufReader;
use std::sync::Mutex;
use std::thread;
use another_one_bytes_the_dust_tile_resource_mapper_tool::coordinates::map_coordinate::MapCoordinate;
use another_one_bytes_the_dust_tile_resource_mapper_tool::tool::tile_mapper::TileMapper;
use std::vec::Vec;
use bob_lib::tracker::{destroy_and_collect_item, Goal, GoalTracker, GoalType, sell_items_in_market, throw_garbage};
use lazy_static::lazy_static;
use robotics_lib::world::tile::Content;
use crate::better_gps::{crazy_noisy_bizarre_gps};

extern crate lazy_static;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::sync::{Arc, mpsc, MutexGuard};
use std::sync::mpsc::{Receiver, RecvError, Sender};



// use tRust_us_Path_finding::tools::gps;
// use tRust_us_Path_finding::tools::gps::{Command, Goal, gps};
// use tile_resource_mapper_tool::tool::tile_resource_mapper_tool{ContentQuantity, TileMapper};

lazy_static!{
    static ref GLOBAL_TRACKER: Mutex<GoalTracker> = Mutex::new(GoalTracker::new());
    static ref GLOBAL_SENDER: Mutex<Option<Sender<()>>> = Mutex::new(None);
}

#[derive(Debug)]
pub enum RobotState{
    Decision,
    GoingSpiral,
    GoingToGoal,
    Default,
    End
}
#[derive(Debug)]
pub enum GoalState{
    PickGarbage,
    ThrowGarbage,
    PickRock,
    SellRock
}

pub struct RobotAttributes{
    //state of the robot
    pub state: RobotState,

    pub goalstate: GoalState,

    // always saved spiral movement
    pub spiral : Vec<Direction>,

    // current vector of direction of the robot
    pub directions: Vec<Direction>,

    //tile-resource-mapper tool mapper
    pub mappertool: TileMapper,

    //tile-resource-mapper tool map
    pub map: Option<HashMap<Discriminant<Content>, Vec<(MapCoordinate, (Option<usize>, Option<Range<usize>>))>>>,

    //goal tracker
    // pub tracker: GoalTracker,

    //previous direction
    pub prev_dir: Direction,

    //recycle or not
    pub recycle: bool,

}
impl RobotAttributes{
    pub fn new() -> RobotAttributes{

        let mut state = RobotState::Default;
        let mut goalstate = GoalState::PickGarbage;
        // creates a vector of directions
        let mov = 150;
        let spiral = spiral_directions(mov);
        let mut directions = spiral_directions(mov);

        let mut mappertool = TileMapper {};
        let mut map = None;

        let mut prev_dir = Direction::Right;

        let mut recycle = false;

        // return
        RobotAttributes{state, goalstate, spiral, directions, mappertool, map, prev_dir, recycle}
    }
}
pub struct MyRobot(pub Robot, pub RobotAttributes);

impl Runnable for MyRobot {
    fn process_tick(&mut self, world: &mut robotics_lib::world::World){
        let mut tracker = GLOBAL_TRACKER.lock().unwrap();
        println!("completed quests: {}", tracker.get_completed_number());
        println!("backpack: {:?}", self.0.backpack);
        // println!("{:?}", where_am_i(self, world));
        println!("{:?}", self.1.directions);
        // println!("{:?}", self.1.prev_dir);
        println!("{:?}", self.1.state);
        // the state of the robot
        match self.1.state{
            RobotState::Decision => {
                self.1.map = TileMapper::collection(world);
                // println!("{:?}", self.1.mappertool.find_closest(world, self, Content::Garbage(0)));

                let mut closest;

                match self.1.goalstate {
                    GoalState::PickGarbage => {closest = self.1.mappertool.find_most_loaded(world, self, Content::Garbage(0).to_default());}
                    GoalState::ThrowGarbage => {closest = self.1.mappertool.find_most_loaded(world, self, Content::Bin(0..0).to_default());}
                    GoalState::PickRock => {closest = self.1.mappertool.find_most_loaded(world, self, Content::Rock(0).to_default());}
                    GoalState::SellRock => {closest = self.1.mappertool.find_most_loaded(world, self, Content::Market(0).to_default());}
                }

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
            RobotState::GoingSpiral => {

                if !self.1.directions.is_empty(){
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

                    match self.1.goalstate {
                        GoalState::PickGarbage => {
                            let _ = destroy_and_collect_item(self, world, self.1.prev_dir.clone(), &mut tracker, Some(Content::Garbage(1).to_default()));
                            let map = self.0.backpack.get_contents();
                            match map.get(&Content::Garbage(0).to_default()) {
                                None => {}
                                Some(a) => {
                                    if a >= &5 {self.1.goalstate = GoalState::ThrowGarbage;}
                                }
                            }
                            self.1.state = RobotState::Decision;
                        }
                        GoalState::ThrowGarbage => {
                            let _ = throw_garbage(self, world, Content::Garbage(0), 5, self.1.prev_dir.clone(), &mut tracker);
                            self.1.goalstate = GoalState::PickRock;
                            self.1.state = RobotState::Decision;
                        }
                        GoalState::PickRock => {
                            let _ = destroy_and_collect_item(self, world, self.1.prev_dir.clone(), &mut tracker, Some(Content::Rock(1).to_default()));
                            let map = self.0.backpack.get_contents();
                            match map.get(&Content::Rock(0).to_default()) {
                                None => {}
                                Some(a) => {
                                    if a >= &20 {self.1.goalstate = GoalState::SellRock;}
                                }
                            }
                            self.1.state = RobotState::Decision;

                        }
                        GoalState::SellRock => {
                            let _ = sell_items_in_market(self, world, Content::Rock(0).to_default(), 20, self.1.prev_dir.clone(), &mut tracker);
                            // let _ = put(self, world, Rock(0).to_default(), 20, self.1.prev_dir.clone());
                            self.1.state = RobotState::End;
                        }
                    }
                    tracker.clean_completed_goals();
                }

            }
            RobotState::Default => {
                let _ = go(self, world, Direction::Right);
                let _ = go(self, world, Direction::Left);

                let g1 = Goal::new(
                    "Throw Garbage".to_string(),
                    "Throw Garbage".to_string(),
                    GoalType::ThrowGarbage,
                    None,
                    5
                );
                let g2 = Goal::new(
                    "Collect Garbage".to_string(),
                    "Collect Garbage".to_string(),
                    GoalType::GetItems,
                    Some(Content::Garbage(1).to_default()),
                    5
                );
                let g3 = Goal::new(
                    "Collect Rocks".to_string(),
                    "Collect Rocks".to_string(),
                    GoalType::GetItems,
                    Some(Content::Rock(1).to_default()),
                    20
                );
                let g4 = Goal::new(
                    "Sell Stuff".to_string(),
                    "Sell Stuff".to_string(),
                    GoalType::SellItems,
                    Some(Content::Rock(1).to_default()),
                    20
                );
                tracker.add_goal(g1);
                tracker.add_goal(g2);
                tracker.add_goal(g3);
                tracker.add_goal(g4);

                let sender = start_sound_loop();
                self.1.state = RobotState::GoingSpiral
            }
            RobotState::End => {
                stop_sound();
                println!("this is the end");
                thread::spawn(move || {
                    let (_stream, stream_handle) = OutputStream::try_default().expect("Impossible to open output stream");
                    let sink = Sink::try_new(&stream_handle).expect("Impossible to creat the sink");
                    let file = File::open("src/Victory.mp3").expect("Impossible to open audio file");

                    sink.append(rodio::Decoder::new(std::io::BufReader::new(file)).expect("Impossible to decode audio file"));
                    sink.sleep_until_end();
                    sink.detach();
                });
            }
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
                thread::sleep(Duration::from_secs_f32(0.4));
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
    let mut steps = 3;
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

        steps += 3; // Aumentiamo di due blocchi

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

        steps += 3; // Aumentiamo di due blocchi
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


pub fn opposite_dir(dir: Direction) -> Direction{
    match dir {
        Direction::Up => {return Direction::Down;}
        Direction::Down => {return Direction::Up;}
        Direction::Left => {return Direction::Right;}
        Direction::Right => {return Direction::Left;}
    }
}

fn start_sound_loop() -> Sender<()> {
    let (stop_tx, stop_rx): (Sender<()>, Receiver<()>) = mpsc::channel();

    {
        let mut sender = GLOBAL_SENDER.lock().unwrap();
        *sender = Some(stop_tx.clone());
    }

    thread::spawn(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let file = BufReader::new(File::open("src/Speedrun.mp3").unwrap());
        let source = Decoder::new(file).unwrap();

        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.set_volume(0.5);
        sink.append(source.repeat_infinite());

        loop {
            if stop_rx.try_recv().is_ok() {
                sink.stop();
                break;
            }
        }
    });

    stop_tx
}

fn stop_sound() {
    if let Some(sender) = &*GLOBAL_SENDER.lock().unwrap() {
        sender.send(()).unwrap();
    }
}