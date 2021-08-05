use std::ops::Mul;
use std::time::Instant;

use toybox::*;

struct Time {
    this_frame: f32,
    delta: f32,
    start: Instant,
}

impl Default for Time {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            this_frame: 0f32,
            delta: 0f32,
            start: now,
        }
    }
}

#[component]
struct Velocity {
    velocity: Vector3,
}

impl From<Vector3> for Velocity {
    fn from(vec: Vector3) -> Self {
        Self { velocity: vec }
    }
}

#[component]
struct AngularVelocity {
    euler_velocity: Euler,
}

impl From<Euler> for AngularVelocity {
    fn from(euler_velocity: Euler) -> Self {
        Self { euler_velocity }
    }
}

#[system]
struct StepTimeSystem;

impl<'r> System<'r> for StepTimeSystem {
    type SystemData = Write<'r, Time>;

    fn run(&mut self, mut time: Self::SystemData) {
        let time: &mut Time = &mut time;
        let this_frame = time.start.elapsed().as_secs_f32();
        time.delta = this_frame - time.this_frame;
        time.this_frame = this_frame;
    }
}

#[system]
struct MoveSystem;

impl<'r> System<'r> for MoveSystem {
    type SystemData = (
        RAWComps<'r, Velocity>,
        WriteComps<'r, Location>,
        RAW<'r, Time>,
    );

    fn run(&mut self, (velocity, mut location, time): Self::SystemData) {
        (&velocity, &mut location).join().for_each(
            |(velocity, location): (&Velocity, &mut Location)| {
                location.location += velocity.velocity * time.delta;
            },
        );
    }
}

#[system]
struct RotateSystem;

impl<'r> System<'r> for RotateSystem {
    type SystemData = (
        RBWComps<'r, AngularVelocity>,
        WriteComps<'r, RotationComp>,
        RAW<'r, Time>,
    );

    fn run(&mut self, (a_velocity, mut rotation, time): Self::SystemData) {
        (&a_velocity, &mut rotation).join().for_each(
            |(a_velocity, rotation): (&AngularVelocity, &mut RotationComp)| {
                rotation.euler += a_velocity.euler_velocity * time.delta;
            },
        );
    }
}

fn main() {
    let mut world = World::default();
    world.insert(Time::default);
    let start = Instant::now();
    for _i in 0..10000 {
        world
            .create_entity()
            .with(Location::new(0f32, 0f32, 0f32))
            .with(Velocity::from(Vector3::new(10f32, 0f32, 0f32)))
            .with(RotationComp::from(Euler::zero()))
            .with(AngularVelocity::from(Euler::new(10f32, 0f32, 0f32)))
            .create();
    }
    let after_entity_generation = Instant::now();
    println!(
        "create entity cost: {}ms",
        (after_entity_generation - start).as_millis()
    );

    let mut scheduler = Scheduler::new(&mut world);
    let after_scheduler_setup = Instant::now();
    println!(
        "setup scheduler cost: {}ms",
        (after_scheduler_setup - after_entity_generation).as_millis()
    );

    scheduler.update(&mut world);
    let elapsed = after_scheduler_setup.elapsed();
    println!("scheduler update cost: {}ms", elapsed.as_millis())
}
