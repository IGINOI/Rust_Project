use std::thread;
use std::thread::sleep;
use std::time::Duration;
use bevy::ui::AlignItems::Default;
use bevy::ui::UiPassNode;
use bevy_extern_events::queue_event;
use OwnerSheeps_Sound_Tool::functions::interface_sound::{craft_with_sound, destroy_with_sound, go_with_sound, put_with_sound};
use OwnerSheeps_Sound_Tool::functions::weather_sounds::{weather_sound, weather_sound_init};
use rand::Rng;
use robotics_lib::energy::Energy;
use robotics_lib::event::events::Event;
use robotics_lib::interface::{destroy, Direction, go, put, robot_map};
use robotics_lib::interface::Direction::{Down, Up, Left, Right};
use robotics_lib::runner::{Robot, Runnable};
use robotics_lib::runner::backpack::BackPack;
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::world::tile::{Content, ContentProps};
use robotics_lib::world::tile::Content::Rock;
use crate::read_events::{ReadRobotEventType, ReadWorldEventType};
use crate::runner::Action::{Destroy, Put, Go, Craft};
use crate::TICK_DURATION;

pub enum Action{
    Destroy,
    Put,
    Go,
    Craft
}
pub struct RobotAttributes{
    actions: Vec<(Action, Direction)>,
    first: bool,
}
impl RobotAttributes{
    pub fn new()-> Self{
        let mut vec = get_actions();
        vec.reverse();
        Self{ actions: vec, first: true }
    }
}
pub struct MyRobot(pub Robot, pub RobotAttributes);

impl Runnable for MyRobot {
    fn process_tick(&mut self, world: &mut robotics_lib::world::World){
        if self.1.first{
            weather_sound_init();
            self.1.first = false;
        }

        let action = self.1.actions.pop();
        match action{
            None => {}
            Some((a,b)) => {
                match a{
                    Destroy => {
                        let _ = destroy_with_sound(self, world, b);
                    }
                    Put => {
                        let map = self.0.backpack.get_contents();
                        let mut content= Content::Rock(0).to_default();
                        for elem in map{
                            if elem.1 != &0{
                                content = elem.0.clone();
                            }
                        }
                        let _ = put_with_sound(self, world, content.clone(), 1, b);
                    }
                    Go => {
                        let _ = go_with_sound(self, world, b);
                    }
                    Craft => {
                        let _ = craft_with_sound(self, Content::JollyBlock(0).to_default());
                    }
                }
            }
        }

        weather_sound(world);
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
                // thread::sleep(Duration::from_secs_f32(0.4));
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


pub fn get_actions() -> Vec<(Action, Direction)>{
    vec![
        (Destroy, Down),
        (Put, Up),
        (Go, Right),
        (Go, Right),
        (Destroy, Down),
        (Put, Up),
        (Go, Right),
        (Go, Right),
        (Destroy, Down),
        (Put, Up),
        (Go, Right),
        (Go, Right),
        (Destroy, Down),
        (Put, Up),
        (Go, Right),
        (Go, Right),
        (Destroy, Down),
        (Put, Up),
        (Go, Right),
        (Go, Right),
        (Destroy, Down),
        (Put, Up),
        (Go, Right),
        (Go, Right),
        (Destroy, Down),
        (Put, Up),
        (Go, Right),
        (Go, Right),
        (Destroy, Down),
        (Put, Up),
        (Go, Right),
        (Put, Up),
        (Go, Right),
        (Destroy, Down),
        (Craft, Up),
        (Put, Up),
        (Go, Down),
        (Go, Down),
        (Go, Down),
        (Go, Down),
        (Go, Down),
        (Go, Down),
        (Go, Down),
        (Go, Down),
        (Go, Down),
        (Go, Down),
        (Go, Down),
        (Go, Down),
        (Go, Down),
        (Go, Down),
        (Go, Down),
        (Go, Down),
        (Destroy, Down),
        (Put, Down),
        (Go, Right),
        (Destroy, Down),
        (Put, Down),
    ]
}