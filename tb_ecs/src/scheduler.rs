use std::any::TypeId;
use std::cell::RefCell;

use rayon::ThreadPool;

use crate::system::SystemData;
use crate::{System, World};

pub struct Scheduler<'t> {
    thread_pool: &'t mut ThreadPool,
    stages: Vec<Stage>,
}

struct Stage {
    groups: Vec<Group>,
}

#[derive(Default)]
struct Group {
    runnable_list: Vec<Box<RefCell<dyn Runnable>>>,
}

pub(crate) trait Runnable: 'static {
    fn run(&mut self, world: &World);
}

#[derive(Eq, PartialEq, Hash)]
struct RunnableId {
    type_id: TypeId,
}

impl Stage {
    fn new(group_num: usize) -> Self {
        let mut groups = vec![];
        groups.reserve(group_num);
        for _i in 0..group_num {
            groups.push(Group::default());
        }
        Self { groups }
    }
}

impl RunnableId {
    fn new<R: Runnable>() -> Self {
        Self {
            type_id: TypeId::of::<R>(),
        }
    }
}

impl<'t> Scheduler<'t> {
    pub fn new(thread_pool: &'t mut ThreadPool) -> Self {
        Self {
            thread_pool,
            stages: vec![],
        }
    }

    pub(crate) fn insert<R: Runnable>(&mut self, runnable: R) {
        self.stages
            .push(Stage::new(self.thread_pool.current_num_threads() * 2));
        let last_stage = self.stages.last_mut().unwrap();
        last_stage.groups[0]
            .runnable_list
            .push(Box::new(RefCell::new(runnable)));
    }

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

impl<T> Runnable for T
where
    for<'r> T: 'static + System<'r>,
{
    fn run(&mut self, world: &World) {
        let mut system_data = T::SystemData::fetch(world);
        self.run(&mut system_data);
    }
}

#[cfg(test)]
mod tests {
    use rayon::ThreadPoolBuilder;

    use crate::system::{Write, RAW};
    use crate::{Scheduler, System, World};

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
        let mut thread_pool = ThreadPoolBuilder::new().build().unwrap();
        let mut scheduler = Scheduler::new(&mut thread_pool);
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
