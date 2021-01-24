use rayon::{ThreadPool, ThreadPoolBuilder};

use tb_core::error::AnyError;
use tb_ecs::{Scheduler, World};

pub struct Application {
    env_args: Vec<String>,
}

impl Application {
    pub fn new() -> Result<Application, AnyError> {
        Ok(Application { env_args: std::env::args().collect() })
    }

    pub fn run(&mut self) {
        let mut scheduler = Scheduler::default();
        let mut world = World::default();

        loop {
            scheduler.schedule(&world);
        }
    }
}
