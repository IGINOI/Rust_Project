use robotics_lib::interface::{Direction, go};
use robotics_lib::runner::Runnable;

pub (crate) fn move_with_backoff(
    bot: &mut impl Runnable,
    world: &mut robotics_lib::world::World,
    main_direction: &Direction,
    backoff1: &Direction,
    backoff2: &Direction,
    backoff3: &Direction,
    text_debug: bool,
) -> bool {
    let r = go(bot, world, main_direction.clone());
    let mut shifted = false;
    match r {
        Ok(_) => { if text_debug { println!("Successfully moved {:?}", main_direction) } }
        Err(_) => {
            if text_debug { println!("Error while moving {:?}, shifting right", main_direction); }
            shifted = true;
            let mut rr = go(bot, world, backoff1.clone());
            match rr {
                Ok(_) => { if text_debug { println!("Shifting {:?} successful", backoff1) } }
                Err(_) => {
                    rr = go(bot, world, backoff2.clone());
                    match rr {
                        Ok(_) => { if text_debug { println!("Shifting {:?} successful", backoff2) } }
                        Err(_) => {
                            _ = go(bot, world, backoff3.clone());
                            if text_debug { println!("Moving back {:?}", backoff3); }
                        }
                    }
                }
            }
        }
    }
    shifted
}