use rayon::{ThreadPool, ThreadPoolBuilder};

use tb_core::error::AnyError;
use tb_ecs::{Scheduler, World};

pub struct Application {
    thread_pool: ThreadPool,
}

impl Application {
    pub fn new() -> Result<Application, AnyError> {
        let thread_pool = ThreadPoolBuilder::new().build()?;
        Ok(Application { thread_pool })
    }

    pub fn run(&mut self) {
        let mut scheduler = Scheduler::new(&mut self.thread_pool);
        let mut world = World::default();

        loop {
            scheduler.schedule(&world);
        }
    }
}
