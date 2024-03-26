You have to create a function that gives an intruction per tick and put it in the process tick (find it in src/runner.rs).
Here you have an example:

```rust
impl Runnable for MyRobot {
    fn process_tick(&mut self, world: &mut robotics_lib::world::World){
        //Here you have to put your code, it is better for the GUI to have an action per tick.
        my_function_computing_moves(self, world);
        //If you have a command per tick you have to put this line of code at the end of the process tick,
          //otherwise if you have multiple commands per tick you have to put this line of code after every commands that could update the map (go, discover, teleport...) 
        queue_event(ReadEventType::LittleMapUpdate(robot_map(world).unwrap()));
    }
}
```
