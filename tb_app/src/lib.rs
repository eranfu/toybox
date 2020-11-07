use std::time::Duration;

use rayon::{ThreadPool, ThreadPoolBuilder};

use tb_core::AnyError;
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
        let mut scheduler = Scheduler::default();
        let mut world = World::default();
        #[cfg(test)]
            let mut loop_number = 10;

        loop {
            scheduler.schedule(&world);
            std::thread::sleep(Duration::default());

            #[cfg(test)] {
                loop_number -= 1;
                if loop_number <= 0 {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tb_core::AnyErrorResult;
    use tb_ecs::Scheduler;

    use crate::Application;

    #[test]
    fn app_works() -> AnyErrorResult<()> {
        let mut app = Application::new()?;
        app.run();
        Ok(())
    }
}
