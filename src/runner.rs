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
use crate::read_events::{ReadRobotEventType, ReadWorldEventType};
use crate::TICK_DURATION;


use another_one_bytes_the_dust_tile_resource_mapper_tool::tool::tile_mapper::TileMapper;
use crate::better_gps::crazy_noisy_bizarre_gps;
use robotics_lib::world::tile::{Content, TileType, Tile};


pub struct MyRobot(pub Robot, pub RobotAttributes);

/*  Things the robot can do :
    destroy()
    put()
    look_at_the_sky()
    go()
    craft()


    tools I can use :
    Journey Journal

    I want the robot to:
    - explore the world
    - collect needed resources (keep track of missing materials)
    - draw the picture (need schematics builder and area calculation)

*/

pub struct RobotAttributes{
    pub directions : Vec<Direction>,                //directions the robot has to take
    pub robot_status: RobotStatus,                  //enum describing state of robot
    pub drawings : Vec<Box<dyn Drawable>>,          //the collection of drawings that contain all the information regarding what the robot has to paint
    pub robot_mission : TodaysMission,              //the enum variable used to save the robot's intent
}

//enum value used to determine in process_tick what the robot has to do when performing specific actions
//example: if the robot is moving towards a tile and his objective is to GetMaterial,
// then his last move will be a get() call instead of a go()
#[derive(Debug, PartialEq)]
enum TodaysMission {
    Testing,
    DrawPixel,
    GetMaterial,
    SetupPlot,
}

//enum value that indicates the current state of the robot, the intention was to give the robot a sort of state machine vibe
//in order to separate the various tasks required to implement the complete AI
#[derive(Debug, PartialEq)]
enum RobotStatus {
    Testing,
    StartOfRound,
    DecideWhatToDo,
    Explore,
    FollowDirections,
    SetupPlot,
    DrawPicture,
    CollectResources,
}

impl RobotAttributes{
    pub fn new() -> RobotAttributes{
        let mut vecdir = spiral_directions_gab_2(50);
        let drawings_vec = vec![];
        RobotAttributes{
            directions : vecdir,
            robot_status: RobotStatus::StartOfRound,
            drawings : drawings_vec,
            robot_mission : TodaysMission::SetupPlot,
        }
    }
}

impl Runnable for MyRobot {
    fn process_tick(&mut self, world: &mut robotics_lib::world::World){
        //I DO NOT HAVE A REAL LOGIC HERE. THE PLAYER SIMPLY MOVES AROUND CASUALLY TRYING TO PUT STREET OR DESTROY CONTENTS IN ORDER TO SEE THE GUI IN ACTION

        match self.1.robot_status {
            //testing phase where I can put anything I like without touching the rest of the logic
            RobotStatus::Testing => {
                println!("testing");
                let spiral_directions = spiral_directions_gab_2(20);
                self.1.directions = spiral_directions;
                self.1.robot_status = RobotStatus::FollowDirections;
            },
            //state I made to patch that dumb error of map not being read
            RobotStatus::StartOfRound => {
                go(self, world, Direction::Left);
                go(self, world, Direction::Right);
                self.1.directions = spiral_directions_gab_2(200);
                self.1.robot_status = RobotStatus::FollowDirections;
            },
            //where I go if I dont know the state I currently am in
            RobotStatus::DecideWhatToDo => {
                println!("decide what to do");
                match &self.1.robot_mission {
                    TodaysMission::SetupPlot => {
                        println!("setup plot");
                        self.1.robot_status = RobotStatus::SetupPlot;
                    },
                    TodaysMission::DrawPixel => {
                        println!("draw pixel");
                        self.1.robot_status = RobotStatus::DrawPicture;
                    },
                    TodaysMission::GetMaterial => {
                        println!("get material");
                        self.1.robot_status = RobotStatus::CollectResources;
                    },
                    _ => {

                    }
                }
            },
            //just simple case where I compute the path to let the robot explore (a spiral movement)
            RobotStatus::Explore => {
                println!("explore");
                self.1.directions = spiral_directions_gab_2(100);
                self.1.robot_status = RobotStatus::FollowDirections;
            },
            RobotStatus::FollowDirections => {
                println!("follow directions");
                //dedicated robot status in which the robot just has to follow the directions of its private attribute directions:
                //check to see if the directions are finished
                if !self.1.directions.is_empty() {
                    //extraction of the first direction on top of the vector
                    let direction = self.1.directions.pop().unwrap();
                    //second check to see if the direction taken is the last direction the robot has to follow
                    if self.1.directions.is_empty() {
                        //needed to check the last direction because based on the robot's mission it could need not to move but to call get() or destroy() in that direction
                        match &self.1.robot_mission {
                            TodaysMission::DrawPixel => {
                                println!("draw pixel");
                                //in case its mission is DrawPixel, the robot has to extract the blueprint in progress and determine the type of material needed for the painting, so that it
                                //can place the right material
                                let last_index = self.1.drawings.len() - 1;
                                let pixel_coords = self.1.drawings.get_mut(last_index).unwrap().get_pixel().unwrap();
                                let random_direction = choose_random_direction();
                                match self.1.drawings.get(last_index).unwrap().drawing_type() {
                                    DrawingType::Coin => {
                                        println!("coin");
                                        if !is_robot_on_next_pixel(where_am_i(self, world).1, pixel_coords) {
                                            match put(self, world, Content::Coin(0), 1, direction) {
                                                //if the put function gives back an ok, the material has been correctly placed
                                                Ok(_) => {
                                                    self.1.drawings.get_mut(last_index).unwrap().increase_pixel_index();
                                                    self.1.robot_status = RobotStatus::DecideWhatToDo;
                                                },
                                                //if the put function gives back an Err, it means the backpack is empty (all other cases are covered by circumstances or other checks)
                                                Err(_) => {
                                                    println!("no coins in backpack");
                                                    self.1.robot_status = RobotStatus::DecideWhatToDo;
                                                    self.1.robot_mission = TodaysMission::GetMaterial;
                                                }
                                            }
                                        } else {
                                            go(self, world, random_direction.clone());
                                        }
                                    },
                                    DrawingType::Rock => {
                                        println!("rock");
                                        if !is_robot_on_next_pixel(where_am_i(self, world).1, pixel_coords) {
                                            match put(self, world, Content::Rock(0), 1, direction) {
                                                //if the put function gives back an ok, the material has been correctly placed
                                                Ok(_) => {
                                                    self.1.drawings.get_mut(last_index).unwrap().increase_pixel_index();
                                                    self.1.robot_status = RobotStatus::DecideWhatToDo;
                                                },
                                                //if the put function gives back an Err, it means the backpack is empty (all other cases are covered by circumstances or other checks)
                                                Err(_) => {
                                                    println!("no rocks in backpack");
                                                    self.1.robot_status = RobotStatus::DecideWhatToDo;
                                                    self.1.robot_mission = TodaysMission::GetMaterial;
                                                }
                                            }
                                        } else {
                                            go(self, world, random_direction.clone());
                                        }
                                    },
                                    DrawingType::Tree => {
                                        println!("tree");
                                        if !is_robot_on_next_pixel(where_am_i(self, world).1, pixel_coords) {
                                            match put(self, world, Content::Tree(0), 1, direction) {
                                                //if the put function gives back an ok, the material has been correctly placed
                                                Ok(_) => {
                                                    self.1.drawings.get_mut(last_index).unwrap().increase_pixel_index();
                                                    self.1.robot_status = RobotStatus::DecideWhatToDo;
                                                },
                                                //if the put function gives back an Err, it means the backpack is empty (all other cases are covered by circumstances or other checks)
                                                Err(_) => {
                                                    println!("no trees in backpack");
                                                    self.1.robot_status = RobotStatus::DecideWhatToDo;
                                                    self.1.robot_mission = TodaysMission::GetMaterial;
                                                }
                                            }
                                        } else {
                                            go(self, world, random_direction.clone());
                                        }
                                    },
                                    DrawingType::JollyBlock => {
                                        println!("jollyblock");
                                        if !is_robot_on_next_pixel(where_am_i(self, world).1, pixel_coords) {
                                            match put(self, world, Content::JollyBlock(0), 1, direction) {
                                                //if the put function gives back an ok, the material has been correctly placed
                                                Ok(_) => {
                                                    self.1.drawings.get_mut(last_index).unwrap().increase_pixel_index();
                                                    self.1.robot_status = RobotStatus::DecideWhatToDo;
                                                },
                                                //if the put function gives back an Err, it means the backpack is empty (all other cases are covered by circumstances or other checks)
                                                Err(_) => {
                                                    println!("no jollyblocks in backpack");
                                                    self.1.robot_status = RobotStatus::DecideWhatToDo;
                                                    self.1.robot_mission = TodaysMission::GetMaterial;
                                                }
                                            }
                                        } else {
                                            go(self, world, random_direction.clone());
                                        }
                                    },
                                }
                            },
                            TodaysMission::GetMaterial => {
                                println!("get material");
                                //in case of mission GetMaterial, it means the robot has to collect the item it has been walking towards, we then match the destroy() output
                                match destroy(self, world, direction){
                                    //in case it is ok, it means the material has been successfully collected and that the robot can go and keep drawing
                                    Ok(quantity) => {
                                        self.1.robot_status = RobotStatus::DrawPicture;
                                        self.1.robot_mission = TodaysMission::DrawPixel;
                                    },
                                    //in case of an Err I didnt quite think of a good alternative, but I guess I can just call back the collectresources state and hope for the best
                                    //it shouldn't happen anyways
                                    Err(_) => {
                                        self.1.robot_status = RobotStatus::CollectResources;
                                        self.1.robot_mission = TodaysMission::GetMaterial;
                                    }
                                }
                            },
                            //this is part of the old logic, it should change soon
                            TodaysMission::SetupPlot => {
                                println!("setup plot");
                                go(self, world, direction);
                                self.1.robot_status = RobotStatus::DecideWhatToDo;
                            },
                            //simple testing to get some outputs
                            TodaysMission::Testing => {
                                println!("testing");
                                let mapper = TileMapper{};
                                let coords = mapper.find_most_loaded(world, self, Content::Coin(2).to_default());
                                println!("{:?}", coords);
                                self.1.robot_status = RobotStatus::Testing;
                            },
                            _ => {}
                        }
                    }else {
                        //if the vec of directions is not empty after popping the direction, it means the robot has to walk with 100% certainty
                        go(self, world, direction);
                    }
                }else{
                    //in case it goes in this state with an empty directions vector, it just has to decide what to do
                    self.1.robot_status = RobotStatus::DecideWhatToDo;
                }
            },
            RobotStatus::SetupPlot => {
                println!("setup plot");
                //initialisation of a couple of variables, used later for a specific purpose
                let map = robot_map(world);
                //drawing_type and plot_size is generated randomly
                let drawing_type = get_random_drawing_type();
                let (mut plot_size, mut drawing_size) = get_random_plot_size(drawing_type);
                println!("drawing type : {:?}, \nplot size: {:?}", drawing_type, plot_size);
                //extraction of candidate for new plot (with new function)
                let maybe_plot = find_plot_from_map(map, plot_size, drawing_type, &self.1.drawings);

                //check to either create a new drawing and beginning the drawing process, or to tell the robot to explore some more
                match maybe_plot {
                    Ok(new_plot) => {
                        //if the plot is found, match the drawing type extracted before (hopefully) at random and put the correct drawing type based on the extraction
                        match drawing_type {
                            DrawingType::Coin => {
                                println!("coin");
                                let mut new_drawing = Coin::new(
                                    new_plot,
                                    Blueprint::new(),
                                    drawing_size,
                                );
                                new_drawing.set_blueprint();
                                self.1.drawings.push(Box::new(new_drawing.clone()));
                                println!("{:?}", new_drawing);
                                //since the robot successfully found a plot, it starts to draw the picture
                                self.1.robot_status = RobotStatus::DrawPicture;
                                self.1.robot_mission = TodaysMission::DrawPixel;
                            },
                            DrawingType::Rock => {
                                println!("rock");
                                let mut new_drawing = Rock::new(
                                    new_plot,
                                    Blueprint::new(),
                                    drawing_size,
                                );
                                new_drawing.set_blueprint();
                                self.1.drawings.push(Box::new(new_drawing.clone()));
                                println!("{:?}", new_drawing);
                                self.1.robot_status = RobotStatus::DrawPicture;
                                self.1.robot_mission = TodaysMission::DrawPixel;
                            },
                            DrawingType::Tree => {
                                println!("tree");
                                let mut new_drawing = Tree::new(
                                    new_plot,
                                    Blueprint::new(),
                                    drawing_size,
                                );
                                new_drawing.set_blueprint();
                                self.1.drawings.push(Box::new(new_drawing.clone()));
                                println!("{:?}", new_drawing);
                                self.1.robot_status = RobotStatus::DrawPicture;
                                self.1.robot_mission = TodaysMission::DrawPixel;
                            },
                            DrawingType::JollyBlock => {
                                println!("jollyblock");
                                let mut new_drawing = JollyBlock::new(
                                    new_plot,
                                    Blueprint::new(),
                                    drawing_size,
                                );
                                new_drawing.set_blueprint();
                                self.1.drawings.push(Box::new(new_drawing.clone()));
                                println!("{:?}", new_drawing);
                                self.1.robot_status = RobotStatus::DrawPicture;
                                self.1.robot_mission = TodaysMission::DrawPixel;
                            }
                        }
                    },
                    Err(_) => {
                        //since the robot did not find any available plot, it should explore some more and retry his/her luck next time (hopefully it will find something!)
                        self.1.robot_status = RobotStatus::Explore;
                        self.1.robot_mission = TodaysMission::SetupPlot;
                    }
                }
            },
            RobotStatus::DrawPicture => {
                println!("draw picture");
                //case in which the robot has to draw the picture
                //extract last index to get the drawing currently being worked on
                let last_index = self.1.drawings.len() - 1;
                //check to make sure the last drawing isn't done (otherwise the robot needs to setup another plot before drawing)
                if self.1.drawings.get(last_index).unwrap().is_done() == true {
                    self.1.robot_status = RobotStatus::SetupPlot;
                }else {
                    //extraction of current drawing
                    if let Some(drawing) = self.1.drawings.get_mut(last_index) {
                        //if there IS a drawing to work on, get next pixel to fill and go draw it
                        let next_pixel_coords = drawing.get_pixel();

                        match next_pixel_coords {
                            Some(coords) => {
                                //check whether robot is on the next pixel to draw (found a bug involving the robot not moving in that position, rare but possible)
                                if !is_robot_on_next_pixel(where_am_i(self, world).1, coords) {
                                    //if not on the next pixel, compute directions and setup a new mission
                                    let mut maybe_pixel_directions = crazy_noisy_bizarre_gps(self, coords, world);
                                    if let Some(pixel_directions) = maybe_pixel_directions {
                                        //if path found, push directions to pixel in directions, setup new status and mission for robot
                                        self.1.directions = pixel_directions;
                                        self.1.robot_status = RobotStatus::FollowDirections;
                                        self.1.robot_mission = TodaysMission::DrawPixel;
                                    } else {
                                        //if path not found, it means it should go and explore a bit more of the world (should not happen under any circumstance)
                                        self.1.robot_status = RobotStatus::Explore;
                                    }
                                } else {
                                    go(self, world, choose_random_direction());
                                }
                            },
                            None => {
                                //if coords to next pixel are not found, it means the drawing is done!
                                //so the robot should mark the drawing as done and go setup the plot for the next one!
                                self.1.drawings.get_mut(last_index).unwrap().mark_as_done();
                                self.1.robot_status = RobotStatus::SetupPlot;
                                self.1.robot_mission = TodaysMission::SetupPlot;
                            }
                        }
                    } else {
                        //if the robot does not get any drawing, it means it has to setup a plot (this particular case should not happen as the robot starts by setting up a plot)
                        self.1.robot_status = RobotStatus::SetupPlot;
                        self.1.robot_mission = TodaysMission::SetupPlot;
                    }
                }
                for i in self.1.drawings.iter().enumerate() {
                    println!("drawing {} : {:?}", i.0, i.1.get_plot());
                }
            },
            RobotStatus::CollectResources => {
                println!("collect resources");
                //needed next line of code for find_most_loaded()
                let mapper = TileMapper{};
                let mut closest_resource;

                //calculate last index for the drawing type and then extract the drawing type from the drawing the robot is currently painting
                let last_index = self.1.drawings.len() - 1;
                let drawing_type = self.1.drawings.get(last_index).unwrap().drawing_type();
                match drawing_type {
                    DrawingType::Coin => {
                        println!("coin");
                        closest_resource = mapper.find_most_loaded(world, self, Content::Coin(2).to_default());
                    },
                    DrawingType::Rock => {
                        println!("rock");
                        closest_resource = mapper.find_most_loaded(world, self, Content::Rock(2).to_default());
                    },
                    DrawingType::Tree => {
                        println!("tree");
                        closest_resource = mapper.find_most_loaded(world, self, Content::Tree(2).to_default());
                    },
                    DrawingType::JollyBlock => {
                        println!("jollyblock");
                        closest_resource = mapper.find_most_loaded(world, self, Content::JollyBlock(2).to_default());
                    }
                }

                match closest_resource {
                    Ok(coords) => {
                        println!("coord found");
                        //if coords are ok, flip them (they're mapped the opposite in the output of find_most_loaded())
                        let flipped_coords : (usize, usize) = (coords.get_height(), coords.get_width());

                        //needed check to avoid collecting resources from completed paintings (with material amount equal to 1)
                        if is_tile_gettable(robot_map(world), flipped_coords, drawing_type) {
                            println!("tile  is gettable");
                            //if the tile is gettable, compute the directions to reach the tile
                            let mut maybe_resource_directions = crazy_noisy_bizarre_gps(self, flipped_coords, world);
                            if let Some(resource_directions) = maybe_resource_directions {
                                //if the function finds a path to the tile, push it onto directions and make the robot go in the FollowDirections status with
                                //mission GetMaterial to collect the resource there
                                self.1.directions = resource_directions;
                                self.1.robot_status = RobotStatus::FollowDirections;
                                self.1.robot_mission = TodaysMission::GetMaterial;
                            } else {
                                //if a path is not found, it should explore more of the world and then try again to collect resources
                                self.1.robot_status = RobotStatus::DecideWhatToDo;
                                self.1.robot_mission = TodaysMission::GetMaterial;
                            }
                        } else {
                            //if the tile isn't gettable, explore the world some more to find some more materials
                            self.1.robot_status = RobotStatus::Explore;
                            self.1.robot_mission = TodaysMission::GetMaterial;
                        }
                    },
                    Err(_) => {
                        println!("coord not found");
                        //if a coord is not found (should be basically impossible) it should explore some more
                        self.1.robot_status = RobotStatus::Explore;
                        self.1.robot_mission = TodaysMission::GetMaterial;
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

fn choose_random_direction() -> Direction{
    let n = rand::thread_rng().gen_range(0..4);
    match n {
        0 => Direction::Right,
        1 => Direction::Left,
        2 => Direction::Up,
        _ => Direction::Down
    }
}

fn choose_multiple_random_directions(quantity : usize) -> Vec<Direction> {
    let mut vec_out : Vec<Direction> = vec![];

    for _ in 0..quantity {
        vec_out.push(choose_random_direction());
    }

    vec_out
}

fn spiral_directions_gab(n: usize) -> Vec<Direction> {
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

fn spiral_directions_gab_2(n : usize) -> Vec<Direction> {
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



#[derive(Debug, Clone)]
struct Plot{
    top_left : (usize, usize),
    bottom_right : (usize, usize),
}

impl Plot {
    fn new() -> Self {
        Self{
            top_left : (0,0),
            bottom_right : (0,0),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum DrawingType {
    Coin,
    Rock,
    Tree,
    JollyBlock,
}
#[derive(Debug, PartialEq, Clone, Copy)]
enum DrawingSize {
    Small,
    Medium,
    Large,
}

trait Drawable{
    fn get_plot(&self) -> &Plot;
    fn set_blueprint(&mut self);
    fn get_pixel(&mut self) -> Option<(usize, usize)>;
    fn increase_pixel_index(&mut self);
    fn is_done(&self) -> bool;
    fn mark_as_done(&mut self);
    fn drawing_type(&self) -> DrawingType;
}
#[derive(Debug, Clone)]
struct Blueprint {
    pixels : Vec<(usize, usize)>,
    pixel_index : usize,
}

impl Blueprint {
    fn new() -> Self{
        Self {
            pixels : Vec::new(),
            pixel_index : 0usize,
        }
    }
}
#[derive(Debug, Clone)]
struct Coin {
    plot : Plot,
    blueprint : Blueprint,
    done : bool,
    drawing_type : DrawingType,
    drawing_size : DrawingSize,
}

impl Coin {
    fn new(plot : Plot, blueprint: Blueprint, drawing_size : DrawingSize) -> Self {
        Self{
            plot,
            blueprint,
            done : false,
            drawing_type : DrawingType::Coin,
            drawing_size
        }
    }
}

impl Drawable for Coin{
    fn get_plot(&self) -> &Plot{
        &self.plot
    }

    fn set_blueprint(&mut self) {
        let drawing_size = self.drawing_size.clone();
        println!("drawing size : {:?}", drawing_size);
        let mut input_pixels: Vec<(usize, usize)> = Vec::new();
        match drawing_size {
            DrawingSize::Small => {
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 1));
                self.blueprint.pixels = input_pixels;
            },
            DrawingSize::Medium => {
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 6, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 6, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 6, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 7, self.plot.top_left.1 + 2));
                self.blueprint.pixels = input_pixels;
            },
            DrawingSize::Large => {
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 5));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 6));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 6));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1 + 6));
                input_pixels.push((self.plot.top_left.0 + 6, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 6, self.plot.top_left.1 + 6));
                input_pixels.push((self.plot.top_left.0 + 7, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 7, self.plot.top_left.1 + 5));
                input_pixels.push((self.plot.top_left.0 + 8, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 8, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 8, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 9, self.plot.top_left.1 + 3));
                self.blueprint.pixels = input_pixels;
            }
            _ => {
                self.blueprint.pixels = input_pixels;
            }
        }
    }

    fn get_pixel(&mut self) -> Option<(usize, usize)> {
        let pixel_index = self.blueprint.pixel_index;
        if pixel_index == self.blueprint.pixels.len() {
            None
        } else {
            Some(*self.blueprint.pixels.get(pixel_index).unwrap())
        }
    }

    fn increase_pixel_index(&mut self) {
        self.blueprint.pixel_index += 1;
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn mark_as_done(&mut self) {
        self.done = true;
    }

    fn drawing_type(&self) -> DrawingType {
        self.drawing_type.clone()
    }
}

#[derive(Debug, Clone)]
struct Rock {
    plot : Plot,
    blueprint : Blueprint,
    done : bool,
    drawing_type : DrawingType,
    drawing_size : DrawingSize,
}

impl Rock {
    fn new(plot : Plot, blueprint : Blueprint, drawing_size : DrawingSize) -> Self {
        Self {
            plot,
            blueprint,
            done : false,
            drawing_type : DrawingType::Rock,
            drawing_size,
        }
    }
}

impl Drawable for Rock {
    fn get_plot(&self) -> &Plot {&self.plot}

    fn set_blueprint(&mut self) {
        let drawing_size = self.drawing_size.clone();
        println!("drawing size : {:?}", drawing_size);
        let mut input_pixels: Vec<(usize, usize)> = Vec::new();
        match drawing_size {
            DrawingSize::Small => {
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 2));
                self.blueprint.pixels = input_pixels;
            },
            DrawingSize::Medium => {
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 5));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 5));
                self.blueprint.pixels = input_pixels;
            },
            DrawingSize::Large => {
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 5));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 6));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 5));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 5));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 5));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 6));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1 + 5));
                self.blueprint.pixels = input_pixels;
            },
            _ => {
                self.blueprint.pixels = input_pixels;
            },
        }
    }

    fn get_pixel(&mut self) -> Option<(usize, usize)> {
        let pixel_index = self.blueprint.pixel_index;
        if pixel_index == self.blueprint.pixels.len() {
            None
        } else {
            Some(*self.blueprint.pixels.get(pixel_index).unwrap())
        }
    }

    fn increase_pixel_index(&mut self) {
        self.blueprint.pixel_index += 1;
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn mark_as_done(&mut self) {
        self.done = true;
    }

    fn drawing_type(&self) -> DrawingType {
        self.drawing_type.clone()
    }
}

#[derive(Debug, Clone)]
struct Tree {
    plot : Plot,
    blueprint : Blueprint,
    done : bool,
    drawing_type : DrawingType,
    drawing_size : DrawingSize,
}

impl Tree {
    fn new(plot : Plot, blueprint : Blueprint, drawing_size : DrawingSize) -> Self {
        Self {
            plot,
            blueprint,
            done : false,
            drawing_type : DrawingType::Tree,
            drawing_size
        }
    }
}

impl Drawable for Tree {
    fn get_plot(&self) -> &Plot {
        &self.plot
    }

    fn set_blueprint(&mut self) {
        let drawing_size = self.drawing_size.clone();
        println!("drawing size : {:?}", drawing_size);
        let mut input_pixels: Vec<(usize, usize)> = Vec::new();
        match drawing_size {
            DrawingSize::Small => {
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 2));
                self.blueprint.pixels = input_pixels;
            },
            DrawingSize::Medium => {
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 5));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 5));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 5));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 6, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 6, self.plot.top_left.1 + 3));
                self.blueprint.pixels = input_pixels;
            },
            DrawingSize::Large => {
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 5));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1 + 5));
                input_pixels.push((self.plot.top_left.0 + 6, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 6, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 6, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 6, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 6, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 6, self.plot.top_left.1 + 5));
                input_pixels.push((self.plot.top_left.0 + 6, self.plot.top_left.1 + 6));
                input_pixels.push((self.plot.top_left.0 + 7, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 8, self.plot.top_left.1 + 3));
                self.blueprint.pixels = input_pixels;
            },
            _ => {
                self.blueprint.pixels = input_pixels;
            }
        }
    }

    fn get_pixel(&mut self) -> Option<(usize, usize)> {
        let pixel_index = self.blueprint.pixel_index;
        if pixel_index == self.blueprint.pixels.len() {
            None
        } else {
            Some(*self.blueprint.pixels.get(pixel_index).unwrap())
        }
    }

    fn increase_pixel_index(&mut self) {
        self.blueprint.pixel_index +=1;
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn mark_as_done(&mut self) {
        self.done = true;
    }

    fn drawing_type(&self) -> DrawingType {
        self.drawing_type.clone()
    }
}

#[derive(Debug, Clone)]
struct JollyBlock {
    plot : Plot,
    blueprint : Blueprint,
    done : bool,
    drawing_type : DrawingType,
    drawing_size : DrawingSize,
}

impl JollyBlock {
    fn new(plot : Plot, blueprint : Blueprint, drawing_size : DrawingSize) -> Self {
        Self {
            plot,
            blueprint,
            done : false,
            drawing_type : DrawingType::JollyBlock,
            drawing_size,
        }
    }
}

impl Drawable for JollyBlock {
    fn get_plot(&self) -> &Plot {
        &self.plot
    }

    fn set_blueprint(&mut self) {
        let drawing_size = self.drawing_size.clone();
        println!("drawing size : {:?}", drawing_size);
        let mut input_pixels: Vec<(usize, usize)> = Vec::new();
        match drawing_size {
            DrawingSize::Small => {
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 2));
                self.blueprint.pixels = input_pixels;
            },
            DrawingSize::Medium => {
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 4));
                self.blueprint.pixels = input_pixels;
            },
            DrawingSize::Large => {
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 1));
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 1, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 2, self.plot.top_left.1 + 5));
                input_pixels.push((self.plot.top_left.0 + 3, self.plot.top_left.1 + 4));
                input_pixels.push((self.plot.top_left.0 + 4, self.plot.top_left.1 + 3));
                input_pixels.push((self.plot.top_left.0 + 5, self.plot.top_left.1 + 2));
                input_pixels.push((self.plot.top_left.0 + 7, self.plot.top_left.1 + 2));
                self.blueprint.pixels = input_pixels;
            },
            _ => {
                self.blueprint.pixels = input_pixels;
            }
        }
    }

    fn get_pixel(&mut self) -> Option<(usize, usize)> {
        let pixel_index = self.blueprint.pixel_index;
        if pixel_index == self.blueprint.pixels.len() {
            None
        } else {
            Some(*self.blueprint.pixels.get(pixel_index).unwrap())
        }
    }

    fn increase_pixel_index(&mut self) {
        self.blueprint.pixel_index +=1;
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn mark_as_done(&mut self) {
        self.done = true;
    }

    fn drawing_type(&self) -> DrawingType {
        self.drawing_type.clone()
    }
}

fn is_plot_inside_other(
    plot_one_top_left : (usize,usize),
    plot_one_bottom_right : (usize, usize),
    plot_two_top_left : (usize,usize),
    plot_two_bottom_right : (usize, usize)) -> bool {

    let mut output : bool = false;
    if plot_one_top_left.0 >= plot_two_top_left.0 && plot_one_top_left.0 <= plot_two_bottom_right.0 {
        if plot_one_top_left.1 >= plot_two_top_left.1 && plot_one_top_left.1 <= plot_two_bottom_right.1 {
            output = true;
        }
    }
    if plot_one_bottom_right.0 >= plot_two_top_left.0 && plot_one_bottom_right.0 <= plot_two_bottom_right.0 {
        if plot_one_bottom_right.1 >= plot_two_top_left.1 && plot_one_bottom_right.1 <= plot_two_bottom_right.1 {
            output = true;
        }
    }

    return output
}

fn find_plot_from_map(map: Option<Vec<Vec<Option<Tile>>>>, plot_size : (usize, usize), drawing_type : DrawingType, drawings : &Vec<Box<dyn Drawable>>) -> Result<Plot, String>{

    let (plot_height, plot_width) = plot_size;

    //check for the map not to be a None
    if let Some(grid) = map {
        //double for to look for every tile in the map
        for y in 0..grid.len() {
            for x in 0..grid[0].len() {
                //check needed to distinguish explored tiles from non explored tiles (none)
                match &grid[y][x] {
                    Some(Tile) => {
                        //setup needed to eventually insert as top_left and bottom_right for a plot (if found)
                        let start_y = y;
                        let end_y = y + plot_height;
                        let start_x = x;
                        let end_x = x + plot_width;

                        //variable needed to skip entire loops in case a tile does not fit the criteria for top_left of a plot
                        let mut is_plot_still_doable = true;
                        //variable needed to check after the double for loops to decide whether to return a plot (plot has been found) or not
                        let mut plot_ok = true;
                        //double for to look for tiles of possible plot
                        for y_plot in start_y..end_y {
                            for x_plot in start_x..end_x {
                                //check needed to skip loops in case of plot not fitting the criteria
                                if is_plot_still_doable == true {
                                    //self explanatory, the function returns true if the tile is in one of the drawings
                                    if is_tile_in_drawings((y_plot,x_plot), drawings) {
                                        is_plot_still_doable = false;
                                        plot_ok = false;
                                        break
                                    } else {
                                        //start of check for each tile of the candidate plot
                                        match &grid[y_plot][x_plot] {
                                            //if the tile selected is something, start checking if it could do for a plot's tile
                                            Some(tile_plot) => {
                                                //check if selected tile type is walkable essentially (I could need to modify this criteria later but for now it is just that)
                                                match drawing_type {
                                                    DrawingType::Tree => {
                                                        match tile_plot.tile_type {
                                                            TileType::Sand | TileType::Snow => {
                                                                is_plot_still_doable = false;
                                                                plot_ok = false;
                                                            },
                                                            _ => {}
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                                match tile_plot.tile_type {
                                                    TileType::DeepWater | TileType::Lava | TileType::Wall | TileType::Street => {
                                                        is_plot_still_doable = false;
                                                        plot_ok = false;
                                                    },
                                                    _ => {}
                                                }
                                                //check if tile is empty, if not, then discard the plot option (for now, might change later)
                                                match tile_plot.content {
                                                    Content::None => {},
                                                    _ => {
                                                        is_plot_still_doable = false;
                                                        plot_ok = false;
                                                    }
                                                }

                                            },
                                            None => {
                                                is_plot_still_doable = false;
                                                plot_ok = false;
                                            }
                                        }
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                        if plot_ok == true {
                            println!("plot ok");
                            let mut new_plot = Plot{
                                top_left: (start_y, start_x),
                                bottom_right: (end_y - 1,  end_x - 1),
                            };
                            return Ok(new_plot)
                        }
                    },
                    None => {},
                }
            }
        }
    }

    return Err(String::from("couldnt setup the plot"))
}

fn is_tile_in_drawings(tile_coords : (usize, usize), drawings : &Vec<Box<dyn Drawable>>) -> bool {
    let mut out = false;
    for drawing in drawings {
        out = is_tile_inside_plot(tile_coords, drawing.get_plot());
    }
    return out
}

fn is_tile_inside_plot(tile_coords : (usize, usize), plot: &Plot) -> bool {
    let mut out = false;
    if tile_coords.0 >= plot.top_left.0 && tile_coords.0 <= plot.bottom_right.0 {
        if tile_coords.1 >= plot.top_left.1 && tile_coords.1 <= plot.bottom_right.1{
            out = true;
        }
    }
    return out
}

fn is_tile_gettable(map: Option<Vec<Vec<Option<Tile>>>>, tile_pos : (usize, usize), drawing_type : DrawingType) -> bool {
    return if let Some(grid) = map {
        let (y, x) = tile_pos;
        match &grid[y][x] {
            Some(tile) => {
                match &drawing_type {
                    DrawingType::Coin => {
                        match tile.content {
                            Content::Coin(1) | Content::Coin(0) => {
                                false
                            },
                            _ => {
                                true
                            },
                        }
                    },
                    DrawingType::Rock => {
                        match tile.content {
                            Content::Rock(1) | Content::Rock(0) => {
                                false
                            },
                            _ => {
                                true
                            },
                        }
                    }
                    DrawingType::Tree => {
                        match tile.content {
                            Content::Tree(1) | Content::Tree(0) => {
                                false
                            },
                            _ => {
                                true
                            }
                        }
                    }
                    DrawingType::JollyBlock => {
                        match tile.content {
                            Content::JollyBlock(1) | Content::JollyBlock(0) => {
                                false
                            },
                            _ => {
                                true
                            }
                        }
                    }
                }
            }
            None => {
                false
            }
        }
    } else {
        false
    }
}

fn is_robot_on_next_pixel(robot_pos : (usize, usize), pixel_pos : (usize, usize)) -> bool {
    return if robot_pos.0 == pixel_pos.0 {
        if robot_pos.1 == pixel_pos.1 {
            true
        } else {
            false
        }
    } else {
        false
    }
}

fn get_random_drawing_type() -> DrawingType {
    let n = rand::thread_rng().gen_range(0..4);
    return match n {
        0 => DrawingType::Coin,
        1 => DrawingType::Rock,
        2 => DrawingType::Tree,
        3 => DrawingType::JollyBlock,
        _ => DrawingType::JollyBlock,
    }
}

fn get_random_plot_size(drawing_type : DrawingType) -> ((usize, usize), DrawingSize) {
    let n = rand::thread_rng().gen_range(0..=100);
    println!("n = {}", n);
    let mut plot_size = (0,0);
    let mut drawing_size = DrawingSize::Small;
    match drawing_type {
        DrawingType::Coin => {
            match n {
                0..=50 => {
                    plot_size = (3, 4);
                    drawing_size = DrawingSize::Small;
                },
                51..=85 => {
                    plot_size = (5, 8);
                    drawing_size = DrawingSize::Medium;
                },
                _ => {
                    plot_size = (7, 10);
                    drawing_size = DrawingSize::Large;
                },
            }
        },
        DrawingType::Rock => {
            match n {
                0..=40 => {
                    plot_size = (3, 4);
                    drawing_size = DrawingSize::Small;
                },
                41..=75 => {
                    plot_size = (4 , 6);
                    drawing_size = DrawingSize::Medium;
                }
                _ => {
                    plot_size = (6 , 7);
                    drawing_size = DrawingSize::Large;
                }
            }
        },
        DrawingType::Tree => {
            match n {
                0..=55 => {
                    plot_size = (5, 5);
                    drawing_size = DrawingSize::Small;
                },
                56..=85 => {
                    plot_size = (7, 6);
                    drawing_size = DrawingSize::Medium;
                },
                _ => {
                    plot_size = (9, 7);
                    drawing_size = DrawingSize::Large;
                }
            }
        },
        DrawingType::JollyBlock => {
            match n {
                0..=55 => {
                    plot_size = (3, 3);
                    drawing_size = DrawingSize::Small;
                },
                56..=85 => {
                    plot_size = (5, 5);
                    drawing_size = DrawingSize::Medium;
                }
                _ => {
                    plot_size = (6, 8);
                    drawing_size = DrawingSize::Large;
                }
            }
        }
    }
    (plot_size, drawing_size)
}

fn opposite_direction(dir : Direction) -> Direction {
    match dir {
        Direction::Up => Direction::Down,
        Direction::Down => Direction::Up,
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left
    }
}
