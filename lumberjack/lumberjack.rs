use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;
use robotics_lib::interface::{Direction, destroy, robot_view, put, robot_map, where_am_i};
use robotics_lib::runner::Runnable;
use robotics_lib::world::tile::{Tile, Content};
use robotics_lib::world::World;
use robotics_lib::interface::look_at_sky;
use robotics_lib::utils::{calculate_cost_go_with_environment, LibError};
use robotics_lib::world::environmental_conditions::EnvironmentalConditions;
extern crate num;

use num::abs;

type Coords = (usize, usize);

/// This function computes the steps to arrive to an object
/// Takes as inputs: a &impl Runnable (your robot), a &World (which is your world), a bool (with which you can select if you are searching for wood or for a market to sell the wood you previously bought)
/// Gives as output a vector that contains the directions to follow to arrive to the closest object (market or wood)
pub fn find_path(robot: &impl Runnable, world: &World, market: bool) -> Option<Vec<Direction>> {
    let dest = find_closest_object(robot, world, market);
    // Debug: Verifica che il robot e il mondo siano validi
    //println!("Robot position: {:?}", where_am_i(robot, world).1);
    let starting_point = where_am_i(robot, world).1;
    //println!("Destination: {:?}", dest);

    // Creazione della mappa con tutte le tile
    let tiles: HashMap<Coords, Contents> = create_map(world)?;
    //println!("Map created with {} tiles", tiles.len());

    // Creazione delle strutture di supporto per il percorso
    let mut dist: HashMap<Coords, usize> = HashMap::new();
    let mut prev: HashMap<Coords, Option<Coords>> = HashMap::new();
    let start = where_am_i(robot, world).1;

    for &coord in tiles.keys() {
        dist.insert(coord, usize::MAX);
        prev.insert(coord, None);
    }
    dist.insert(start, 0);

    let mut heap = BinaryHeap::new();
    heap.push(Node::new(start, 0, tiles.get(&start)?.elevation));

    while let Some(Node { coords, path, cost, elevation }) = heap.pop() {
        if coords == dest {
            // Debug: Stampa il percorso trovato
            //println!("Path found: {:?}", path);
            return Some(path_to_directions(starting_point, path));
        }

        if cost > *dist.get(&coords).unwrap_or(&usize::MAX) {
            continue;
        }

        let neighbors = get_neighbors(&coords, &tiles);

        for neighbor in neighbors {
            let next = neighbor.coords;
            let new_cost = cost + neighbor.cost + (neighbor.elevation as isize - elevation as isize).abs() as usize;

            if new_cost < *dist.get(&next).unwrap_or(&usize::MAX) {
                dist.insert(next, new_cost);
                prev.insert(next, Some(coords));

                let mut new_path = path.clone();
                new_path.push(next);

                heap.push(Node::create(next, new_path, new_cost, neighbor.elevation));
            }
        }
    }

    // Debug: Stampa un messaggio se non è stato trovato alcun percorso
    println!("No path found");
    None
}

/// This function finds the closest object to the robot.
/// It takes in input: a &impl Runnable (your robot), a &World (which is your world), a bool (with which you can select if you are searching for wood or for a market to sell the wood you previously bought)
/// It returns the coordinates to the closest content you are searching for.
/// This function is usually automatically called from the find_path function, but you can use it also separately
fn find_closest_object(robot:&impl Runnable, mut world: &World, market: bool) -> (usize, usize) {
    let self_x = robot.get_coordinate().get_row();
    let self_y = robot.get_coordinate().get_col();

    let robot_map = robot_map(&mut world).unwrap();

    let mut smaller_distance = 1000;
    let mut current_distance ;
    let mut resource_coordinates: (usize, usize) = (10000, 10000);
    for x in 0..robot_map.len(){
        for y in 0..robot_map.len(){
            match robot_map[x][y].clone(){
                None => {}
                Some(tile) => {
                    match tile.content {
                        Content::Tree(_) => {
                            if !market {
                                current_distance = abs(self_x as i32 - x as i32) + abs(self_y as i32 - y as i32);
                                if current_distance < smaller_distance {
                                    resource_coordinates = (x, y);
                                    smaller_distance = current_distance;
                                    println!("The smaller distance is: {:?}", current_distance);
                                }
                            }
                        },
                        Content::Market(_) => {
                            if market {
                                current_distance = abs(self_x as i32 - x as i32) + abs(self_y as i32 - y as i32);
                                if current_distance < smaller_distance {
                                    resource_coordinates = (x, y);
                                    smaller_distance = current_distance;
                                    println!("The smaller distance is: {:?}", current_distance);
                                }
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
    }
    println!("{:?}", resource_coordinates);
    return resource_coordinates;
}


/// This function tries to chop wood in all the 4 direction, useful if you are in the middle of a lot of trees.
/// You can use it combined to the previous tool to make the chopping faster.
/// It returns a vector of direction of the braking attempt combined with its result from the 'destroy' interface
pub fn chop_wood(robot: &mut impl Runnable, world: &mut World) -> Vec<(Direction, Result<usize, LibError>)> {
    let r_view = robot_view(robot, world);
    let mut directions_to_remove = Vec::new();

    match &r_view[0][1] {
        Some(tile) => match tile.content {
            Content::Tree(_) => { directions_to_remove.push(Direction::Up) }
            _ => {}
        },
        _ => {}
    }
    match &r_view[2][1] {
        Some(tile) => match tile.content {
            Content::Tree(_) => { directions_to_remove.push(Direction::Down) }
            _ => {}
        },
        _ => {}
    }
    match &r_view[1][0] {
        Some(tile) => match tile.content {
            Content::Tree(_) => { directions_to_remove.push(Direction::Left) }
            _ => {}
        },
        _ => {}
    }
    match &r_view[1][2] {
        Some(tile) => match tile.content {
            Content::Tree(_) => { directions_to_remove.push(Direction::Right) }
            _ => {}
        },
        _ => {}
    }

    let mut action_result: Vec<(Direction, Result<usize, LibError>)> = Vec::new();
    for direction in directions_to_remove {
        action_result.push((direction.clone(), destroy(robot, world, direction.clone())));
    }
    action_result
}

/// This function tries to sell wood in all the 4 direction.
/// You can use it combined to the previous tool to make the selling faster.
/// It returns a vector of direction of the braking attempt combined with its result from the 'put' interface
pub fn sell_wood(robot: &mut impl Runnable, world: &mut World, quantity: usize) -> Vec<(Direction, Result<usize, LibError>)> {
    let r_view = robot_view(robot, world);
    let mut directions_to_remove = Vec::new();

    match &r_view[0][1] {
        Some(tile) => match tile.content {
            Content::Market(_) => { directions_to_remove.push(Direction::Up) }
            _ => {}
        },
        _ => {}
    }
    match &r_view[2][1] {
        Some(tile) => match tile.content {
            Content::Market(_) => { directions_to_remove.push(Direction::Down) }
            _ => {}
        },
        _ => {}
    }
    match &r_view[1][0] {
        Some(tile) => match tile.content {
            Content::Market(_) => { directions_to_remove.push(Direction::Left) }
            _ => {}
        },
        _ => {}
    }
    match &r_view[1][2] {
        Some(tile) => match tile.content {
            Content::Market(_) => { directions_to_remove.push(Direction::Right) }
            _ => {}
        },
        _ => {}
    }

    let content_in = Content::Tree(quantity);
    let mut action_result: Vec<(Direction, Result<usize, LibError>)> = Vec::new();
    for direction in directions_to_remove {
        action_result.push((direction.clone(), put(robot, world, content_in.clone(), quantity.clone(), direction.clone())));
    }
    action_result
}






#[derive(Debug, Clone)]
pub struct Neighbors {
    up: Option<Coords>,
    down: Option<Coords>,
    left: Option<Coords>,
    right: Option<Coords>,
}

impl Neighbors {
    fn new() -> Self {
        Self {
            up: None,
            down: None,
            left: None,
            right: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Contents {
    cost: usize,
    elevation: usize,
    neighbors: Neighbors,
}

impl Contents {
    fn create(cost: usize, elevation: usize, neighbors: Neighbors) -> Self {
        Self { cost, elevation, neighbors }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    coords: Coords,
    path: Vec<Coords>,
    cost: usize,
    elevation: usize,
}

impl Node {
    fn new(coords: Coords, cost: usize, elevation: usize) -> Self {
        Self {
            coords,
            path: vec![],
            cost,
            elevation,
        }
    }

    fn create(coords: Coords, path: Vec<Coords>, cost: usize, elevation: usize) -> Self {
        Self { coords, path, cost, elevation }
    }
}

// Implement the Ord trait for Node to use it in a BinaryHeap
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost).then_with(|| self.coords.cmp(&other.coords))
    }
}

// Implement the PartialOrd trait for Node
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.coords == other.coords
    }
}

impl Eq for Node {}

pub fn get_neighbors(coords: &Coords, map: &HashMap<Coords, Contents>) -> Vec<Node> {
    let mut vec = vec![];
    if let Some(content) = map.get(coords) {
        let neighbors = &content.neighbors;
        let coords_vec = vec![neighbors.up, neighbors.right, neighbors.left, neighbors.down];

        for coord_option in coords_vec {
            if let Some(coord) = coord_option {
                if let Some(content) = map.get(&coord) {
                    vec.push(Node::create(
                        coord,
                        vec![coord],
                        content.cost,
                        content.elevation,
                    ));
                }
            }
        }
    }
    vec
}

pub fn create_map(world: &World) -> Option<HashMap<Coords, Contents>> {
    let mut map = HashMap::new();
    let opt_map = robot_map(world)?;

    for (x, row) in opt_map.iter().enumerate() {
        for (y, tile_option) in row.iter().enumerate() {
            if let Some(tile) = tile_option {
                let coords = (x, y);
                let cost = simple_cost(look_at_sky(world), tile);
                let elevation = tile.elevation;
                let mut neighbors = Neighbors::new();

                if y > 0 {
                    neighbors.left = opt_map.get(y - 1).and_then(|row| row.get(x)).map(|_| (x, y - 1));
                }
                if y < opt_map.len() - 1 {
                    neighbors.right = opt_map.get(y + 1).and_then(|row| row.get(x)).map(|_| (x, y + 1));
                }
                if x > 0 {
                    neighbors.up = opt_map.get(y).and_then(|row| row.get(x - 1)).map(|_| (x - 1, y));
                }
                if x < opt_map.len() - 1 {
                    neighbors.down = opt_map.get(y).and_then(|row| row.get(x + 1)).map(|_| (x + 1, y));
                }

                map.insert(coords, Contents::create(cost, elevation, neighbors));
            }
        }
    }
    Some(map)
}

fn simple_cost(weather_report: EnvironmentalConditions, tile: &Tile) -> usize {
    let mut base_cost = tile.tile_type.properties().cost();
    base_cost = calculate_cost_go_with_environment(base_cost, weather_report, tile.tile_type);
    base_cost
}




pub fn path_to_directions(start: (usize, usize), path: Vec<(usize, usize)>) -> Vec<Direction> {
    let mut directions = vec![];

    // Includi la partenza nel percorso
    let mut full_path = vec![start];
    full_path.extend(path);

    for window in full_path.windows(2) {
        if let [current, next] = window {
            let direction = if next.0 > current.0 {
                Direction::Down
            } else if next.0 < current.0 {
                Direction::Up
            } else if next.1 > current.1 {
                Direction::Right
            } else if next.1 < current.1 {
                Direction::Left
            } else {
                continue; // This case should not happen
            };

            directions.push(direction);
        }
    }

    directions
}


