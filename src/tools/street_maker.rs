use std::fmt::Debug;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use bevy_extern_events::queue_event;

use robotics_lib::energy::Energy;
use robotics_lib::event::events::Event;
use robotics_lib::interface::{destroy, Direction, robot_map, robot_view};
use robotics_lib::runner::{Robot, Runnable, Runner};
use robotics_lib::runner::backpack::BackPack;
use robotics_lib::utils::LibError;
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::world::tile::Content;
use robotics_lib::world::world_generator::*;

use worldgen_unwrap::public::*;

use bessie::bessie::*;
use tRust_us_Path_finding::tools::gps;
use tRust_us_Path_finding::tools::gps::{Command, Goal};

use crate::gui::gui_test::start_gui;
use crate::gui::read_events::{ReadRobotEventType, ReadWorldEventType};
use crate::tools::street_maker::ActionsEncoding::*;

use crate::tools::movement_functions::*;

use crate::gui::gui_test::WORLD_PATH;

#[derive(Clone, Debug, PartialEq)]
pub enum RobotState { Idle, Exploring, PavingRoad }

#[derive(Clone, Debug, PartialEq)]
// First 4 actions are for moving, then robot_view, pick_rock, make_road
pub enum ActionsEncoding { MU, MR, MD, ML, RW, PR(Direction), TR(Direction) }

struct RobotAttributes {
    state: RobotState,
    inventory_full: bool,
    future_actions: Vec<ActionsEncoding>,
    spiral_radius: i32,
    spiral_shift: i32,
    building_1: Option<(i32, i32)>,
    building_2: Option<(i32, i32)>,
    path_first_building_calculated: bool,
    printed_message: bool,
}

impl RobotAttributes {
    fn new() -> Self {
        Self {
            state: RobotState::Idle,
            inventory_full: false,
            future_actions: Vec::new(),
            spiral_radius: 3,
            spiral_shift: 0,
            building_1: None,
            building_2: None,
            path_first_building_calculated: false,
            printed_message: false,
        }
    }
}

pub struct Bot(
    Robot,
    RobotAttributes,
);

// Robot will find two buildings and then connect them to each other using roads (Bessy tool)

impl Runnable for Bot {
    fn process_tick(&mut self, world: &mut robotics_lib::world::World) {
        if self.1.future_actions.len() > 0 {
            // println!("Current action: {:?}", self.1.future_actions[0]);
            match self.1.future_actions[0].clone() {
                MU => {
                    if move_with_backoff(self, world, &Direction::Up, &Direction::Right, &Direction::Left, &Direction::Down, false) {
                        self.1.spiral_shift += 1;
                    } else {
                        self.1.spiral_shift = 0;
                    }
                }
                MR => {
                    if move_with_backoff(self, world, &Direction::Right, &Direction::Down, &Direction::Up, &Direction::Left, false) {
                        self.1.spiral_shift += 1;
                    } else {
                        self.1.spiral_shift = 0;
                    }
                }
                MD => {
                    if move_with_backoff(self, world, &Direction::Down, &Direction::Left, &Direction::Right, &Direction::Up, false) {
                        self.1.spiral_shift += 1;
                    } else {
                        self.1.spiral_shift = 0;
                    }
                }
                ML => {
                    if move_with_backoff(self, world, &Direction::Left, &Direction::Up, &Direction::Down, &Direction::Right, false) {
                        self.1.spiral_shift += 1;
                    } else {
                        self.1.spiral_shift = 0;
                    }
                }
                RW => {
                    let r_view = robot_view(self, world);

                    let self_x = self.get_coordinate().get_row() as i32;
                    let self_y = self.get_coordinate().get_col() as i32;

                    for row in 0..3 {
                        for col in 0..3 {
                            match r_view[row][col].clone().unwrap().content {
                                Content::Rock(_) => {
                                    if !self.1.inventory_full {
                                        match row {
                                            0 => {
                                                match col {
                                                    0 => {
                                                        self.1.future_actions.insert(1, MR);
                                                        self.1.future_actions.insert(1, PR(Direction::Up));
                                                        self.1.future_actions.insert(1, ML);
                                                    }
                                                    1 => {
                                                        self.1.future_actions.insert(1, PR(Direction::Up));
                                                    }
                                                    2 => {
                                                        self.1.future_actions.insert(1, ML);
                                                        self.1.future_actions.insert(1, PR(Direction::Up));
                                                        self.1.future_actions.insert(1, MR);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            1 => {
                                                match col {
                                                    0 => {
                                                        self.1.future_actions.insert(1, PR(Direction::Left));
                                                    }
                                                    1 => {}
                                                    2 => {
                                                        self.1.future_actions.insert(1, PR(Direction::Right));
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            2 => {
                                                match col {
                                                    0 => {
                                                        self.1.future_actions.insert(1, MR);
                                                        self.1.future_actions.insert(1, PR(Direction::Down));
                                                        self.1.future_actions.insert(1, ML);
                                                    }
                                                    1 => {
                                                        self.1.future_actions.insert(1, PR(Direction::Down));
                                                    }
                                                    2 => {
                                                        self.1.future_actions.insert(1, ML);
                                                        self.1.future_actions.insert(1, PR(Direction::Down));
                                                        self.1.future_actions.insert(1, MR);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                Content::Building => {
                                    if self.1.building_1.is_none() {
                                        println!("First building found: {:?}", (self_x + row as i32 - 1, self_y + col as i32 - 1));
                                        self.1.building_1 = Some((self_x + row as i32 - 1, self_y + col as i32 - 1));
                                    } else if self.1.building_2.is_none() {
                                        if self.1.building_1.unwrap() != (self_x + row as i32 - 1, self_y + col as i32 - 1) {
                                            println!("Second building found: {:?}", (self_x + row as i32 - 1, self_y + col as i32 - 1));
                                            self.1.building_2 = Some((self_x + row as i32 - 1, self_y + col as i32 - 1));
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    if self.1.inventory_full &&
                        self.1.building_1.is_some() &&
                        self.1.building_2.is_some() {
                        self.1.state = RobotState::PavingRoad;
                        self.1.future_actions.clear();
                    }
                }
                PR(d) => {
                    if !self.1.inventory_full {
                        let r = destroy(self, world, d.clone());
                        match r {
                            Ok(_) => { println!("Picked up rock successfully"); }
                            Err(e) => {
                                match e {
                                    LibError::NotEnoughSpace(_) => {
                                        println!("Content not picked up, not enough space in backpack");
                                        self.1.inventory_full = true;
                                    }

                                    _ => { println!("Generic error while picking up rock {:?}", e); }
                                }
                            }
                        }
                    }
                }
                TR(d) => {
                    // Stuff to make road
                    let r = road_paving_machine(
                        self,
                        world,
                        d.clone(),
                        State::MakeRoad,
                    );
                    match r {
                        Ok(_) => { println!("Paved road successfully"); }
                        Err(e) => {
                            match e {
                                RpmError::NoRockHere => {
                                    self.1.future_actions.clear();
                                    self.1.building_1 = Some((self.0.coordinate.get_row() as i32, self.0.coordinate.get_col() as i32));
                                    println!("Building 1: {:?} --- Building 2: {:?}", self.1.building_1, self.1.building_2);
                                    self.1.path_first_building_calculated = false;
                                    self.1.inventory_full = false;
                                    self.1.spiral_radius = 3;
                                    self.1.spiral_shift = 0;
                                    self.1.state = RobotState::Idle;
                                }
                                _ => { println!("Error while paving road: {:?}", e); }
                            }
                        }
                    }
                }
            }
            if self.1.future_actions.len() > 0 { self.1.future_actions.remove(0); }
        }

        match self.1.state {
            RobotState::Idle => {
                for _ in 0..6 {
                    for t in vec![MR, MD, ML, MU] {
                        let spiral_length = self.1.spiral_radius / 2 + if self.1.spiral_radius % 2 != 0 { 1 } else { 0 };
                        for _ in 0..spiral_length {
                            self.1.future_actions.push(t.clone());
                            self.1.future_actions.push(RW);
                        }

                        self.1.spiral_radius += 3;
                    }
                }
                self.1.future_actions.insert(0, RW);

                self.1.state = RobotState::Exploring;
            }
            RobotState::Exploring => {}
            RobotState::PavingRoad => {
                // println!("Starting to make the road");

                if !self.1.path_first_building_calculated {
                    let mut path = gps::gps(
                        self,
                        Goal::Coordinates(self.1.building_1.unwrap().0 as usize, self.1.building_1.unwrap().1 as usize),
                        world,
                        None,
                    ).unwrap().0;

                    path.reverse();
                    for c in &path {
                        match c {
                            Command::Control(d) => {
                                match d {
                                    Direction::Up => { self.1.future_actions.insert(0, MU); }
                                    Direction::Down => { self.1.future_actions.insert(0, MD); }
                                    Direction::Left => { self.1.future_actions.insert(0, ML); }
                                    Direction::Right => { self.1.future_actions.insert(0, MR); }
                                }
                            }
                            _ => {}
                        }
                    }
                    self.1.path_first_building_calculated = true;
                } else {
                    if self.1.future_actions.len() == 0 {
                        let robot_coordinates = (self.0.coordinate.get_row() as i32, self.0.coordinate.get_col() as i32);
                        if robot_coordinates == self.1.building_1.unwrap() {
                            println!("Arrived at the starting building");

                            let path_to_building = gps::gps(
                                self,
                                Goal::Coordinates(self.1.building_2.unwrap().0 as usize, self.1.building_2.unwrap().1 as usize),
                                world,
                                None,
                            ).unwrap().0;

                            println!("Path to building: {:?}", path_to_building);
                            // path_to_building.reverse();
                            for c in &path_to_building {
                                match c {
                                    Command::Control(d) => {
                                        self.1.future_actions.push(TR(d.clone()));
                                        match d {
                                            Direction::Up => { self.1.future_actions.push(MU); }
                                            Direction::Down => { self.1.future_actions.push(MD); }
                                            Direction::Left => { self.1.future_actions.push(ML); }
                                            Direction::Right => { self.1.future_actions.push(MR); }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        } else if robot_coordinates == self.1.building_2.unwrap() {
                            if !self.1.printed_message {
                                println!("Arrived at destination :)");
                                self.1.printed_message = true;
                            }
                            self.1.future_actions.insert(0, RW);
                        }
                    }
                }
            }
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

pub fn world_test() {
    let mut a = WorldgeneratorUnwrap::init(false, Some(PathBuf::from(WORLD_PATH)));
    // let mut a = WorldgeneratorUnwrap::init(true, None);
    let _ = a.gen();

    thread::spawn(|| {
        thread::sleep(Duration::from_millis(15000));
        let mut runner = Runner::new(Box::new(Bot(Robot::new(), RobotAttributes::new())), &mut WorldgeneratorUnwrap::init(false, Some(PathBuf::from(WORLD_PATH)))).unwrap();
        loop {
            let _ = runner.game_tick();
            thread::sleep(Duration::from_millis(600));
        }
    });


    start_gui();
}