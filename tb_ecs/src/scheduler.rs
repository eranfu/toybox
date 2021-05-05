use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::prelude::*;

use tb_core::algorithm::topological_sort::{Node, TopologicalGraph};
use tb_core::event_channel::{EventChannel, ReaderHandle};

use crate::{ResourcesChangeEvent, System, SystemData, SystemInfo, SystemRegistry, World};

pub struct Scheduler {
    resources_change_event_reader: ReaderHandle,
    systems: Vec<Box<dyn RunnableSystem>>,
    dependants: Vec<Vec<usize>>,
    num_dependencies_cache: Vec<AtomicUsize>,
    await_counter: Vec<AtomicUsize>,
}

impl Scheduler {
    pub fn new(world: &mut World) -> Self {
        let channel: &mut EventChannel<ResourcesChangeEvent> = world.insert(Default::default);
        let resources_change_event_reader = channel.register();
        let mut scheduler = Self {
            systems: vec![],
            dependants: vec![],
            num_dependencies_cache: vec![],
            await_counter: vec![],
            resources_change_event_reader,
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
        self.num_dependencies_cache
            .par_iter_mut()
            .for_each(|counter: &mut AtomicUsize| counter.store(0, Ordering::Relaxed));
        self.dependants
            .par_iter_mut()
            .for_each(|dependant| dependant.clear());

        let mut sr = SystemRegistry::get_instance();
        let sr: &mut SystemRegistry = &mut sr;
        let systems = sr.systems();
        let infos: Vec<_> = systems
            .par_iter()
            .filter(|(&system_info, _node)| system_info.is_resource_matched(world))
            .collect();

        let mut info_to_index = HashMap::with_capacity(infos.len());
        self.systems.reserve(infos.len());
        for (i, (&info, _node)) in infos.iter().enumerate() {
            self.systems.push(info.create_system());
            info_to_index.insert(info, i);
        }

        self.num_dependencies_cache
            .resize_with(self.systems.len(), || AtomicUsize::new(0));
        self.dependants.resize(self.systems.len(), Vec::new());

        for (i, (_info, node)) in infos.iter().enumerate() {
            self.add_dependency(i, node, &info_to_index, systems);
        }
    }

    fn add_dependency(
        &mut self,
        dependant_index: usize,
        node: &Node<&SystemInfo>,
        info_to_index: &HashMap<&SystemInfo, usize>,
        systems: &TopologicalGraph<&SystemInfo>,
    ) {
        for dependency in node.dependencies() {
            if let Some(&system_index) = info_to_index.get(dependency) {
                self.num_dependencies_cache[dependant_index].fetch_add(1, Ordering::Relaxed);
                self.dependants[system_index].push(dependant_index);
            } else {
                let dependency_node = systems.node(dependency).unwrap();
                self.add_dependency(dependant_index, dependency_node, info_to_index, systems);
            }
        }
    }
}

pub trait RunnableSystem: Send {
    fn run(&mut self, world: &World);
}

impl<T> RunnableSystem for T
where
    for<'r> T: System<'r> + Send,
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
        _value: i32,
    }

    struct OtherResource {
        value: i32,
    }

    impl<'r> System<'r> for TestSystem {
        type SystemData = Write<'r, TestResource>;

        fn run(&mut self, mut system_data: Self::SystemData) {
            system_data._value = 20;
        }
    }

    impl<'r> System<'r> for OtherSystem {
        type SystemData = (Write<'r, TestResource>, RAW<'r, OtherResource>);

        fn run(&mut self, (mut test, other): Self::SystemData) {
            test._value = 30;
            assert_eq!(other.value, 100);
        }
    }
}
