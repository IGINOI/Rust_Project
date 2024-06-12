// use std::fmt::Debug;
// use std::hash::Hash;
// use std::path::PathBuf;
// use std::thread;
// use std::time::Duration;
// use bessie::bessie::RpmError;
// use bevy_extern_events::queue_event;
// use rand::Rng;
// use rand::thread_rng;
//
// use robotics_lib::energy::Energy;
// use robotics_lib::event::events::Event;
// use robotics_lib::interface::{destroy, Direction, robot_map, robot_view};
// use robotics_lib::runner::{Robot, Runnable, Runner};
// use robotics_lib::runner::backpack::BackPack;
// use robotics_lib::utils::LibError;
// use robotics_lib::world::coordinates::Coordinate;
// use robotics_lib::world::tile::Content;
// use robotics_lib::world::world_generator::*;
//
// use worldgen_unwrap::public::*;
//
// // use bessie::bessie::*;
// use tRust_us_Path_finding::tools::gps;
// use tRust_us_Path_finding::tools::gps::{Command, Goal};
//
// use crate::gui::gui_test::start_gui;
// use crate::gui::read_events::ReadEventType;
// use crate::tools::world_gen::ActionsEncoding::*;
//
// use crate::tools::movement_functions::*;
//
// use crate::gui::gui_test::WORLD_PATH;
//
// #[derive(Clone, Debug, PartialEq)]
// pub enum RobotState { Idle, Exploring, PickingRocks, Trailing }
//
// #[derive(Clone, Debug, PartialEq)]
// // First 4 actions are for moving, then robot_view, pick_rock, make_road, first exploration, connected two buildings
// pub enum ActionsEncoding { MU, MR, MD, ML, RW, PR(Direction), TR(Direction), INIT, SCB((i32, i32)), ECB((i32, i32)) }
//
// #[derive(Clone)]
// struct RobotAttributes {
//     state: RobotState,
//     init: bool,
//     future_actions: Vec<ActionsEncoding>,
//     spiral_radius: i32,
//     spiral_shift: i32,
//     discovered_rocks: Vec<(i32, i32, bool)>, // Boolean in this case represents the unreachable rocks
//     discovered_buildings: Vec<(i32, i32)>,
// }
//
// impl RobotAttributes {
//     fn new() -> Self {
//         Self {
//             state: RobotState::Idle,
//             init: true,
//             future_actions: Vec::new(),
//             spiral_radius: 1,
//             spiral_shift: 0,
//             discovered_rocks: Vec::new(),
//             discovered_buildings: Vec::new(),
//         }
//     }
// }
// pub struct Bot(
//     Robot,
//     RobotAttributes,
// );
//
// // Robot will find two buildings and then connect them to each other using roads (Bessy tool)
//
//
// impl Runnable for Bot {
//     fn process_tick(&mut self, world: &mut robotics_lib::world::World) {
//         // println!("Robot state: {:?}", self.1.state);
//         // if let Some(index) = self.1.future_actions.iter().position(|x| x == &INIT) {
//         //     println!("Index of INIT: {}", index);
//         // }
//
//         // Statement for exploring state
//         if self.1.state == RobotState::Idle ||
//             (self.1.state == RobotState::Exploring && self.1.future_actions.len() == 0)
//         {
//             // Set generic spiraling pattern if no other actions to do
//             // MR MD ML MU
//             let mut direction_vector = vec![MU, MR, MD, ML];
//             let shift = thread_rng().gen_range(0..(direction_vector.len() + 1));
//             direction_vector.rotate_left(shift);
//
//             for _ in 0..6 {
//                 for t in vec![MU, MR, MD, ML] {
//                     let spiral_length = self.1.spiral_radius / 2 + if self.1.spiral_radius % 2 != 0 { 1 } else { 0 };
//                     for _ in 0..spiral_length {
//                         for _ in 0..3 {
//                             self.1.future_actions.push(t.clone());
//                             self.1.future_actions.push(RW);
//                         }
//                     }
//
//                     self.1.spiral_radius += 1;
//                 }
//             }
//             self.1.future_actions.push(INIT);
//             self.1.state = RobotState::Exploring;
//         }
//
//         // Actions for moving robot (includes robot_view in order to not introduce another state)
//         else if self.1.state == RobotState::Exploring {
//             match self.1.future_actions[0].clone() {
//                 RW => {
//                     let r_view = robot_view(self, world);
//
//                     // Picking rocks if any are present
//                     let mut rocks_to_pick: Vec<(i32, i32)> = Vec::new();
//
//                     // Adding buildings currently in (robot) view to the persistent list of discovered buildings
//                     // Adding rocks currently in (robot) view to a temporary list of discovered rocks (will be picked up a little later)
//                     for row_index in 0..r_view.len() {
//                         for column_index in 0..r_view[row_index].len() {
//                             let current_element = r_view[row_index][column_index].clone();
//
//                             if let Some(c) = current_element {
//                                 3
//                                 let content_coords = ((robot_coords.0 + row_index - 1) as i32, (robot_coords.1 + column_index - 1) as i32);
//
//                                 match c.content {
//                                     Content::Building => {
//                                         if !self.1.discovered_buildings.contains(&(content_coords.0, content_coords.1)) && self.1.discovered_buildings.len() < 2 {
//                                             println!("Building found at coords {:?}", content_coords);
//                                             self.1.discovered_buildings.push((content_coords.0, content_coords.1));
//                                         }
//                                     }
//                                     Content::Rock(n) => {
//                                         if n > 0 {
//                                             // println!("Rock found at coords {:?}", content_coords);
//                                             rocks_to_pick.push(content_coords);
//                                         }
//                                     }
//                                     _ => {}
//                                 }
//                             }
//                         }
//                     }
//                     if !self.1.init {
//                         let mut trailing:Vec<bool> = Vec::new();
//                         let mut number_of_rocks = 0;
//                         for i in self.0.backpack.get_contents() {
//                             match i.0 {
//                                 Content::Rock(_) => { number_of_rocks += 1; }
//                                 _ => {}
//                             }
//                         }
//
//                         if self.1.discovered_buildings.len() == 2 {
//                             let first_building = self.1.discovered_buildings[0];
//                             let second_building
//                         }
//                         // if self.1.discovered_buildings.len() > 2 && rocks_to_pick.len() == 0 || number_of_rocks != 0 {
//                         //     println!("About to connect two buildings");
//                         //     // Checking if two buildings are discovered, if so start connecting them (if not connected already)
//                         //
//                         //     let mut uub_present = false;
//                         //
//                         //     for b in &self.1.discovered_buildings {
//                         //         for bb in &self.1.discovered_buildings {
//                         //             if !self.1.connected_buildings.contains(
//                         //                 &(
//                         //                     (self.1.discovered_buildings[0].0, self.1.discovered_buildings[0].1),
//                         //                     (self.1.discovered_buildings[1].0, self.1.discovered_buildings[1].1))) {
//                         //                 uub_present = true;
//                         //             }
//                         //         }
//                         //     }
//                         //
//                         //     if uub_present {
//                         //         self.1.state = RobotState::Trailing;
//                         //         self.1.future_actions.clear();
//                         //     }
//                         //
//                         //     // Find path from one building to the other and then connect them
//                         // }
//                         if rocks_to_pick.len() > 0 &&
//                             trailing.len() < 2 &&
//                             !self.1.future_actions.contains(&PR(Direction::Up)) &&
//                             !self.1.future_actions.contains(&PR(Direction::Down)) &&
//                             !self.1.future_actions.contains(&PR(Direction::Left)) &&
//                             !self.1.future_actions.contains(&PR(Direction::Right))
//                         {
//                             self.1.state = RobotState::PickingRocks;
//
//                             for i in &rocks_to_pick {
//                                 self.1.discovered_rocks.push((i.0, i.1, true));
//                                 self.1.future_actions.clear();
//                             }
//                         }
//
//                     }
//                 }
//
//                 INIT => {
//                     self.1.init = false
//                 }
//
//                 PR(d) => {
//                     // Stuff to pick rock
//                     let r = destroy(self, world, d);
//                     match r {
//                         Ok(_) => { println!("Picked up rock successfully"); }
//                         Err(e) => {
//                             match e {
//                                 LibError::NotEnoughSpace(_) => {
//                                     println!("Content not picked up, not enough space in backpack");
//                                     self.1.future_actions.remove(0);
//                                 }
//
//                                 _ => { println!("Generic error while picking up rock {:?}", e); }
//                             }
//                         }
//                     }
//                 }
//
//                 TR(d) => {
//                     // Stuff to make road
//                     println!("Making road here");
//                     let r = bessie::bessie::road_paving_machine(
//                         self,
//                         world,
//                         d.clone(),
//                         bessie::bessie::State::MakeRoad
//                     );
//                     match r {
//                         Ok(_) => {}
//                         Err(e) => {
//                             match e {
//                                 RpmError::MustDestroyContentFirst => {
//                                     let _ = bessie::bessie::road_paving_machine(
//                                         self,
//                                         world,
//                                         d.clone(),
//                                         bessie::bessie::State::GetStones
//                                     );
//                                 }
//                                 _ => {
//                                     println!("Unable to make road, resetting movements");
//                                     self.1.future_actions.clear()
//                                 }
//                             }
//
//                         }
//                     }
//                 }
//
//                 SCB(end_coords) => {
//                     let path_to_building = gps::gps(
//                         self,
//                         Goal::Coordinates(end_coords.0 as usize, end_coords.1 as usize),
//                         world,
//                         None
//                     );
//                     match path_to_building {
//                         None => {}
//                         Some(c) => {
//                             let mut p = c.0.clone();
//                             p.reverse();
//                             for c in &p {
//                                 match c {
//                                     Command::Control(d) => {
//                                         match d {
//                                             Direction::Up => {
//                                                 self.1.future_actions.insert(0, TR(d.clone()));
//                                                 self.1.future_actions.insert(0, MU);
//                                             }
//                                             Direction::Down => {
//                                                 self.1.future_actions.insert(0, TR(d.clone()));
//                                                 self.1.future_actions.insert(0, MD);
//                                             }
//                                             Direction::Left => {
//                                                 self.1.future_actions.insert(0, TR(d.clone()));
//                                                 self.1.future_actions.insert(0, ML);
//                                             }
//                                             Direction::Right => {
//                                                 self.1.future_actions.insert(0, TR(d.clone()));
//                                                 self.1.future_actions.insert(0, MR);
//                                             }
//                                         }
//                                     }
//                                     _ => {}
//                                 }
//                             }
//                         }
//                     }
//                 }
//
//                 ECB(start_coords) => {
//                     let r_view = robot_view(self, world);
//                     match r_view[1][1].clone() {
//                         Some(tile) => {
//                             match tile.content {
//                                 Content::Building => {
//                                     self.1.connected_buildings.push((start_coords, (self.get_coordinate().get_row() as i32, self.get_coordinate().get_col() as i32)))
//                                 }
//                                 _ => {}
//                             }
//                         }
//                         _ => {}
//                     }
//                 }
//
//                 // If action is of movement keep spiraling (going in by an extra 1 if movement is not possible)
//                 // The buildings that could be discovered before the next `robot_view` aren't lost just found a bit later
//                 t => {
//                     match t {
//                         MU => {
//                             if move_with_backoff(self, world, &Direction::Up, &Direction::Right, &Direction::Left, &Direction::Down, false) {
//                                 self.1.spiral_shift += 1;
//                             } else {
//                                 self.1.spiral_shift = 0;
//                             }
//                         }
//                         MR => {
//                             if move_with_backoff(self, world, &Direction::Right, &Direction::Down, &Direction::Up, &Direction::Left, false) {
//                                 self.1.spiral_shift += 1;
//                             } else {
//                                 self.1.spiral_shift = 0;
//                             }
//                         }
//                         MD => {
//                             if move_with_backoff(self, world, &Direction::Down, &Direction::Left, &Direction::Right, &Direction::Up, false) {
//                                 self.1.spiral_shift += 1;
//                             } else {
//                                 self.1.spiral_shift = 0;
//                             }
//                         }
//                         ML => {
//                             if move_with_backoff(self, world, &Direction::Left, &Direction::Up, &Direction::Down, &Direction::Right, false) {
//                                 self.1.spiral_shift += 1;
//                             } else {
//                                 self.1.spiral_shift = 0;
//                             }
//                         }
//                         _ => {}
//                     }
//
//
//                     // Resetting spiraling radius if 3 or more shifts in a row
//                     if self.1.spiral_shift >= 3 || self.1.spiral_radius >= 5 {
//                         self.1.spiral_radius = 1;
//                         self.1.spiral_shift = 0;
//                     }
//                 }
//             }
//
//             // Remove completed action (if list hasn't been cleared)
//             if self.1.future_actions.len() > 0 { self.1.future_actions.remove(0);}
//         } else if self.1.state == RobotState::PickingRocks {
//             // Code to calculate moves to:
//             // - Get to rocks
//             for rock in self.1.discovered_rocks.clone() {
//                 if rock.2 {
//                     let path_to_rock = gps::gps(
//                         self,
//                         Goal::Coordinates(rock.0 as usize, rock.1 as usize),
//                         world,
//                         None
//                     );
//
//                     match path_to_rock {
//                         // If no path to rock (for whatever reason) mark as unreachable
//                         None => {
//                             if let Some(mut tuple) = self.1.discovered_rocks.get_mut(0) {
//                                 tuple.2 = false;
//                             }
//                         }
//
//                         Some(p) => {
//                             let mut p_clone = p.0.clone();
//                             let mut breaking_action = None;
//
//                             if p_clone.len() > 0 {
//                                 if let Command::Control(d) = p_clone[p_clone.len() - 1].clone() {
//                                     breaking_action = Some(PR(d));
//                                 }
//
//                                 p_clone.reverse();
//                                 p_clone.remove(0);
//                                 for c in &p_clone {
//                                     match c {
//                                         Command::Control(d) => {
//                                             match d {
//                                                 Direction::Up => { self.1.future_actions.insert(0, MD) }
//                                                 Direction::Down => { self.1.future_actions.insert(0, MU) }
//                                                 Direction::Left => { self.1.future_actions.insert(0, MR) }
//                                                 Direction::Right => { self.1.future_actions.insert(0, ML) }
//                                             }
//                                         }
//                                         _ => {}
//                                     }
//                                 }
//                                 match breaking_action {
//                                     Some(a) => {
//                                         self.1.future_actions.insert(0, a);
//                                     }
//                                     None => {}
//                                 }
//
//
//                                 p_clone.reverse();
//
//                                 if p.1 > 0 {
//                                     for c in &p_clone {
//                                         match c {
//                                             Command::Control(d) => {
//                                                 match d {
//                                                     Direction::Up => { self.1.future_actions.insert(0, MU) }
//                                                     Direction::Down => { self.1.future_actions.insert(0, MD) }
//                                                     Direction::Left => { self.1.future_actions.insert(0, ML) }
//                                                     Direction::Right => { self.1.future_actions.insert(0, MR) }
//                                                 }
//                                             }
//                                             _ => {}
//                                         }
//                                     }
//
//                                     // Creating reverse actions to get back to starting point
//                                     p_clone.reverse();
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//             self.1.state = RobotState::Exploring;
//
//         } else if self.1.state == RobotState::Trailing {
//             println!("Trailing state, finding path to nearest unconnected building");
//             let mut first_building:Option<(i32, i32)> = None;
//             let mut second_building:Option<(i32, i32)> = None;
//
//             let mut path_first_building = None;
//             let mut path_second_building = None;
//
//             for b in &self.1.discovered_buildings {
//                 for bb in &self.1.discovered_buildings {
//                     if !self.1.connected_buildings.contains(&((b.0, b.1), (bb.0, bb.1))) {
//                         first_building = Some((b.0, b.1));
//                         second_building = Some((bb.0, bb.1));
//                         path_first_building = gps::gps(
//                             self,
//                             Goal::Coordinates(first_building.clone().unwrap().0 as usize, first_building.clone().unwrap().1 as usize),
//                             world,
//                             None
//                         );
//                         path_second_building = gps::gps(
//                             self,
//                             Goal::Coordinates(second_building.clone().unwrap().0 as usize, second_building.clone().unwrap().1 as usize),
//                             world,
//                             None
//                         );
//
//                         if path_first_building.is_some() && path_second_building.is_some() {
//                             break;
//                         }
//                     }
//                 }
//             }
//
//             let mut p1 = path_first_building.unwrap().0;
//             let mut p2 = path_second_building.unwrap().0;
//             let mut p:Vec<Command> = Vec::new();
//             if p1.len() >= p2.len() {
//                 self.1.future_actions.insert(0, SCB(second_building.clone().unwrap()));
//                 p = p1.clone();
//             } else {
//                 self.1.future_actions.insert(0, SCB(first_building.clone().unwrap()));
//                 p = p2.clone();
//             }
//
//             p.reverse();
//             for c in p {
//                 match c {
//                     Command::Control(d) => {
//                         match d {
//                             Direction::Up => { self.1.future_actions.insert(0, MU); }
//                             Direction::Down => { self.1.future_actions.insert(0, MD); }
//                             Direction::Left => { self.1.future_actions.insert(0, ML); }
//                             Direction::Right => { self.1.future_actions.insert(0, MR); }
//                         }
//                     }
//                     _ => {}
//                 }
//             }
//
//             // if let Some(first_unconnected_building) = find_unconnected_building(
//             //     (self.get_coordinate().get_row() as i32, self.get_coordinate().get_col() as i32),
//             //     self.1.discovered_buildings.clone()
//             // ) {
//             //     println!("found a building to connect");
//             //     self.1.discovered_buildings[first_unconnected_building as usize].2 = false;
//             //     if let Some(path_to_building) = gps::gps(
//             //         self,
//             //         Goal::Coordinates(self.1.discovered_buildings[first_unconnected_building as usize].0 as usize, self.1.discovered_buildings[first_unconnected_building as usize].1 as usize),
//             //         world,
//             //         None
//             //     ) {
//             //         let mut path_clone = path_to_building.0.clone();
//             //         path_clone.reverse();
//             //         self.1.future_actions.insert(0, CB((self.get_coordinate().get_row() as i32, self.get_coordinate().get_col() as i32)));
//             //         for c in path_clone {
//             //             match c {
//             //                 Command::Control(d) => {
//             //                     match d {
//             //                         Direction::Up => {
//             //                             self.1.future_actions.insert(0, MU);
//             //                             self.1.future_actions.insert(0, TR(d));
//             //                         }
//             //                         Direction::Down => {
//             //                             self.1.future_actions.insert(0, MD);
//             //                             self.1.future_actions.insert(0, TR(d));
//             //                         }
//             //                         Direction::Left => {
//             //                             self.1.future_actions.insert(0, ML);
//             //                             self.1.future_actions.insert(0, TR(d));
//             //                         }
//             //                         Direction::Right => {
//             //                             self.1.future_actions.insert(0, MR);
//             //                             self.1.future_actions.insert(0, TR(d));
//             //                         }
//             //                     }
//             //                 }
//             //                 _ => {}
//             //             }
//             //         }
//             //     } else {
//             //         println!("Did not find a path to the building");
//             //         self.1.discovered_buildings[first_unconnected_building as usize].2 = false;
//             //     }
//             // }
//             self.1.state = RobotState::Exploring;
//         }
//         queue_event(ReadEventType::LittleMapUpdate(robot_map(world).unwrap()));
//     }
//
//     fn handle_event(&mut self, event: Event) {
//         match event{
//             Event::Ready => {
//                 println!("We are super ready");
//             }
//             Event::Terminated => {
//                 println!("We are fucking done");
//             }
//             Event::TimeChanged(new_conditions) => {
//                 queue_event(ReadEventType::TimeChanged(new_conditions));
//             }
//             Event::DayChanged(new_conditions) => {
//                 queue_event(ReadEventType::TimeChanged(new_conditions));
//             }
//             Event::EnergyRecharged(energy_recharged) => {
//                 queue_event(ReadEventType::EnergyRecharged((energy_recharged, self.get_energy().get_energy_level())));
//             }
//             Event::EnergyConsumed(energy_consumed) => {
//                 queue_event(ReadEventType::EnergyConsumed(energy_consumed));
//             }
//             Event::Moved(_tile, position) => {
//                 queue_event(ReadEventType::RobotMoved((position.0,position.1)));
//             }
//             Event::TileContentUpdated(tile, position) => {
//                 queue_event(ReadEventType::UpdatedTile((tile, (position.0, position.1))));
//             }
//             Event::AddedToBackpack(_content, _quantity) => {
//                 let mut vec_content = vec![];
//                 for tile_content in self.get_backpack().get_contents(){
//                     if *tile_content.1 != 0{
//                         for _ in 0..*tile_content.1{
//                             vec_content.push(tile_content.0.clone());
//                         }
//                     }
//                 }
//                 println!("{:?}",vec_content);
//                 println!("{:?}",self.0.backpack);
//                 queue_event(ReadEventType::AddBackpack(vec_content));
//             }
//
//             Event::RemovedFromBackpack(_content, _quantity) => {
//                 let mut vec_content = vec![];
//                 for tile_content in self.get_backpack().get_contents(){
//                     if *tile_content.1 != 0{
//                         for _ in 0..*tile_content.1{
//                             vec_content.push(tile_content.0.clone());
//                         }
//                     }
//                 }
//                 println!("{:?}",vec_content);
//                 println!("{:?}",self.0.backpack);
//                 queue_event(ReadEventType::RemoveBackpack(vec_content));
//             }
//         }
//     }
//     fn get_energy(&self) -> &Energy { &self.0.energy }
//     fn get_energy_mut(&mut self) -> &mut Energy { &mut self.0.energy }
//     fn get_coordinate(&self) -> &Coordinate { &self.0.coordinate }
//     fn get_coordinate_mut(&mut self) -> &mut Coordinate { &mut self.0.coordinate }
//     fn get_backpack(&self) -> &BackPack { &self.0.backpack }
//     fn get_backpack_mut(&mut self) -> &mut BackPack { &mut self.0.backpack }
// }
//
// pub fn world_test() {
//     let mut a = WorldgeneratorUnwrap::init(false, Some(PathBuf::from(WORLD_PATH)));
//     // let mut a = WorldgeneratorUnwrap::init(true, None);
//     let world = a.gen();
//
//     thread::spawn(||{
//         thread::sleep(Duration::from_millis(10000));
//         let mut runner = Runner::new(Box::new(Bot(Robot::new(), RobotAttributes::new())), &mut WorldgeneratorUnwrap::init(false, Some(PathBuf::from(WORLD_PATH)))).unwrap();
//         loop {
//             let _ = runner.game_tick();
//             thread::sleep(Duration::from_millis(125));
//         }
//     });
//
//
//     start_gui();
// }