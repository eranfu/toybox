use std::time::Instant;

use toybox::*;

fn main() {
    let start = Instant::now();
    const NUM: usize = 1000000;
    let mut location: Vec<_> = (0..NUM).map(|_| Location::new(0f32, 0f32, 0f32)).collect();
    let velocity: Vec<_> = (0..NUM).map(|_| Location::new(10f32, 0f32, 0f32)).collect();
    let after_entity_generation = Instant::now();
    println!(
        "create entity cost: {}ms",
        (after_entity_generation - start).as_millis()
    );

    let after_scheduler_setup = Instant::now();
    println!(
        "setup scheduler cost: {}ms",
        (after_scheduler_setup - after_entity_generation).as_millis()
    );

    location
        .iter_mut()
        .zip(velocity.iter())
        .for_each(|(location, velocity)| {
            location.location.x += velocity.location.x * 0.1f32;
            location.location.y += velocity.location.y * 0.1f32;
            location.location.z += velocity.location.z * 0.1f32;
        });

    let elapsed = after_scheduler_setup.elapsed();
    println!("scheduler update cost: {}ms", elapsed.as_millis())
}
