use std::time::Instant;

use toybox::*;

fn main() {
    let mut world = World::default();

    let mut scheduler = Scheduler::new(&mut world);
    let start = Instant::now();
    scheduler.update(&mut world);
    let elapsed = start.elapsed();
    println!("{}", elapsed.as_millis())
}
