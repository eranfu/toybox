use std::sync::atomic::AtomicUsize;

use tb_core::event_channel::{EventChannel, ReaderHandle};

use crate::{ResourcesChangeEvent, System, SystemData, SystemRegistry, World};

pub struct Scheduler {
    resources_change_event_reader: ReaderHandle,
    systems: Vec<Box<dyn RunnableSystem>>,
    await_counter_cache: Vec<AtomicUsize>,
    await_counter: Vec<AtomicUsize>,
    dependents: Vec<Vec<usize>>,
}

impl Scheduler {
    pub fn new(world: &mut World) -> Self {
        let channel: &mut EventChannel<ResourcesChangeEvent> = world.insert(Default::default);
        let resources_change_event_reader = channel.register();
        let mut scheduler = Self {
            systems: vec![],
            await_counter_cache: vec![],
            await_counter: vec![],
            resources_change_event_reader,
            dependents: vec![],
        };
        scheduler.refresh_systems(world);
        scheduler
    }
    pub fn update(&mut self, world: &mut World) {
        let events: &mut EventChannel<ResourcesChangeEvent> = world.fetch_mut();
        if events.read_any(&mut self.resources_change_event_reader) {
            self.refresh_systems(world);
        }
    }
    fn refresh_systems(&mut self, world: &mut World) {
        self.systems.clear();
        self.await_counter_cache.clear();
        SystemRegistry::par_iter()
    }
}

pub(crate) trait RunnableSystem {
    fn run(&mut self, world: &World);
}

impl<T> RunnableSystem for T
where
    for<'r> T: System<'r>,
{
    fn run(&mut self, world: &World) {
        self.run(T::SystemData::fetch(world));
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[system]
    struct TestSystem {}

    #[system]
    struct OtherSystem {}

    struct TestResource {
        value: i32,
    }

    struct OtherResource {
        value: i32,
    }

    impl<'r> System<'r> for TestSystem {
        type SystemData = Write<'r, TestResource>;

        fn run(&mut self, mut system_data: Self::SystemData) {
            system_data.value = 20;
        }
    }

    impl<'r> System<'r> for OtherSystem {
        type SystemData = (Write<'r, TestResource>, RAW<'r, OtherResource>);

        fn run(&mut self, (mut test, other): Self::SystemData) {
            test.value = 30;
            assert_eq!(other.value, 100);
        }
    }
}
