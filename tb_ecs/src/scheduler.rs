use std::cell::RefCell;

use crate::{System, World};
use crate::system::SystemData;

#[derive(Default)]
pub struct Scheduler {
    stages: Vec<Stage>,
}

impl Scheduler {
    pub(crate) fn insert(&mut self, runnable: impl Runnable) {
        let last_stage = {
            match self.stages.last_mut() {
                None => {
                    self.stages.push(Stage::default());
                    self.stages.last_mut().unwrap()
                }
                Some(last) => {
                    last
                }
            }
        };

        let last_group = {
            match last_stage.groups.last_mut() {
                None => {
                    last_stage.groups.push(Group::default());
                    last_stage.groups.last_mut().unwrap()
                }
                Some(last) => {
                    last
                }
            }
        };

        last_group.runnable_list.push(Box::new(RefCell::new(runnable)));
    }
}

#[derive(Default)]
struct Stage {
    groups: Vec<Group>
}

#[derive(Default)]
struct Group {
    runnable_list: Vec<Box<RefCell<dyn Runnable>>>,
}

pub(crate) trait Runnable: 'static {
    fn run(&mut self, world: &World);
}

impl Scheduler {
    pub fn schedule(&self, world: &World) {
        for stage in &self.stages {
            for group in &stage.groups {
                for runnable in &group.runnable_list {
                    runnable.borrow_mut().run(world);
                }
            }
        }
    }
}

impl<T> Runnable for T where for<'r> T: 'static + System<'r> {
    fn run(&mut self, world: &World) {
        let mut system_data = T::SystemData::fetch(world);
        self.run(&mut system_data);
    }
}

#[cfg(test)]
mod tests {
    use crate::{Scheduler, System, World};
    use crate::system::{RAW, Write};

    struct TestSystem {}

    struct OtherSystem {}

    struct TestResource {
        value: i32,
    }

    struct OtherResource {
        value: i32,
    }

    impl<'r> System<'r> for TestSystem {
        type SystemData = Write<'r, TestResource>;

        fn run(&mut self, system_data: &mut Self::SystemData) {
            system_data.value = 20;
        }
    }

    impl<'r> System<'r> for OtherSystem {
        type SystemData = (Write<'r, TestResource>, RAW<'r, OtherResource>);

        fn run(&mut self, system_data: &mut Self::SystemData) {
            system_data.0.value = 30;
            assert_eq!(system_data.1.value, 100);
        }
    }

    #[test]
    fn scheduler_works() {
        let mut world = World::default();
        let mut scheduler = Scheduler::default();
        let resource = TestResource { value: 10 };
        world.insert(resource);
        world.insert(OtherResource { value: 100 });
        assert_eq!(world.fetch::<TestResource>().value, 10);

        scheduler.insert(TestSystem {});
        scheduler.schedule(&world);
        assert_eq!(world.fetch::<TestResource>().value, 20);

        scheduler.insert(OtherSystem {});
        scheduler.schedule(&world);
        assert_eq!(world.fetch::<TestResource>().value, 30);
    }
}