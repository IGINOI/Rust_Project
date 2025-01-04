# Rust GUI with Bevy
## What is this repository about
This repository contains the project for an university course; to make it simple, it is a sort of minecraft game played by the computer.

I want to point out that the repository branches, do not have a good form and are not well maintained due to my past low experience on GitHub.

The master branch contains my contribution (the GUI) to a university project. In order to use it you can follow the directives below.

The other branches contains my collegues contribution (the logic with which the computer plays). These branches are a rudimentary merge (that I attempted) with my collegues' works. The credits for these codes go to: @pedwoo @nicocecchin @Gabbo717. All the branches can be download and they can potentially run by themselves, note that they already contain the GUI. 

Hovewer, to make it work, you also need the common crate, which I don't know if, by the time you are reading this, it will still be available online (University Policy, sorry).

## How to use it
To use my GUI you simply have to:
- download the repository and add all the files in you project.
- add the function `queue_event(ReadEventType::LittleMapUpdate(robot_map(world).unwrap()));` to the `process_tick` function. You can workout whether to use mine or yours; in case you can find it in src/runner.rs.
- add the event calling `command queue_event(ReadEventType::LittleMapUpdate(robot_map(world).unwrap()));` to make the little map work.

Here you have an example:

```rust
impl Runnable for MyRobot {
    fn process_tick(&mut self, world: &mut robotics_lib::world::World){
        //Here you have to put your code: it should be a function shut the one below that returns per tick (optimal for the GUI).
        my_function_computing_moves(self, world);


        //If you have a command per tick you have to put the following line of code at the end of the process tick,
        //otherwise if you have multiple commands per tick you have to put this line of code after every commands that could update the map (go, discover, teleport...) 
        queue_event(ReadEventType::LittleMapUpdate(robot_map(world).unwrap()));
    }
}
```
