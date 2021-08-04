use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicUsize, Ordering};

use dashmap::DashSet;
use rayon::prelude::*;

use tb_core::algorithm::topological_sort::{Node, TopologicalGraph};
use tb_core::event_channel::ReaderHandle;

use crate::{System, SystemData, SystemInfo, SystemRegistry, World};

pub struct Scheduler {
    resources_change_event_reader: ReaderHandle,
    systems: Vec<RunnableCell>,
    dependants: Vec<DashSet<usize>>,
    dependencies_counter_cache: Vec<AtomicUsize>,
    dependencies_counter: Vec<AtomicUsize>,
}

impl Scheduler {
    pub fn new(world: &mut World) -> Self {
        let channel = world.resource_change_events_mut();
        let resources_change_event_reader = channel.register();
        let mut scheduler = Self {
            systems: vec![],
            dependants: vec![],
            dependencies_counter_cache: vec![],
            dependencies_counter: vec![],
            resources_change_event_reader,
        };
        scheduler.refresh_systems(world);
        scheduler
    }

    pub fn update(&mut self, world: &mut World) {
        let events = world.resource_change_events();
        if events.read_any(&mut self.resources_change_event_reader) {
            self.refresh_systems(world);
        }

        self.dependencies_counter.par_iter().enumerate().for_each(
            |(i, counter): (usize, &AtomicUsize)| {
                counter.store(
                    self.dependencies_counter_cache[i].load(Ordering::Relaxed),
                    Ordering::Relaxed,
                );
            },
        );

        (0..self.systems.len())
            .into_par_iter()
            .for_each(|i| unsafe {
                self.run_system_recursive(i, world);
            })
    }

    unsafe fn run_system_recursive(&self, i: usize, world: &World) {
        let counter = &self.dependencies_counter[i];
        if counter.fetch_sub(1, Ordering::Release) == 1 {
            counter.load(Ordering::Acquire);
            self.systems[i].get_mut().run(world);
            self.dependants[i].par_iter().for_each(|dependant| {
                self.run_system_recursive(*dependant, world);
            })
        }
    }

    fn refresh_systems(&mut self, world: &mut World) {
        let mut sr = SystemRegistry::instance();
        let sr: &mut SystemRegistry = &mut sr;
        let systems = sr.systems();
        let infos: Vec<_> = systems
            .par_iter()
            .filter(|(&system_info, _node)| system_info.is_resource_matched(world))
            .collect();

        let mut info_to_index = HashMap::with_capacity(infos.len());
        self.systems.clear();
        self.systems.reserve(infos.len());
        for (i, (&info, _node)) in infos.iter().enumerate() {
            self.systems
                .push(RunnableCell(UnsafeCell::new(info.create_system())));
            info_to_index.insert(info, i);
        }

        self.dependants
            .par_iter_mut()
            .for_each(|dependant| dependant.clear());
        self.dependants.resize(self.systems.len(), DashSet::new());
        infos
            .par_iter()
            .enumerate()
            .for_each(|(i, (_info, node))| self.add_dependants(i, node, &info_to_index, systems));

        self.dependencies_counter_cache
            .par_iter()
            .for_each(|counter: &AtomicUsize| counter.store(1, Ordering::Relaxed));
        self.dependencies_counter_cache
            .resize_with(self.systems.len(), || AtomicUsize::new(1));
        self.dependants
            .par_iter()
            .for_each(|dependants: &DashSet<usize>| {
                dependants.par_iter().for_each(|dependant| {
                    self.dependencies_counter_cache[*dependant].fetch_add(1, Ordering::Relaxed);
                });
            });

        self.dependencies_counter
            .resize_with(self.dependencies_counter_cache.len(), || {
                AtomicUsize::new(1)
            });
    }

    fn add_dependants(
        &self,
        dependant_index: usize,
        node: &Node<&SystemInfo>,
        info_to_index: &HashMap<&SystemInfo, usize>,
        systems: &TopologicalGraph<&SystemInfo>,
    ) {
        node.dependencies()
            .par_iter()
            .for_each(|dependency: &&SystemInfo| {
                if let Some(&system_index) = info_to_index.get(dependency) {
                    self.dependants[system_index].insert(dependant_index);
                } else {
                    let dependency_node = systems.node(dependency).unwrap();
                    self.add_dependants(dependant_index, dependency_node, info_to_index, systems);
                }
            });
    }
}

struct RunnableCell(UnsafeCell<Box<dyn RunnableSystem>>);

impl RunnableCell {
    #[allow(clippy::mut_from_ref)]
    pub(crate) fn get_mut(&self) -> &mut dyn RunnableSystem {
        (unsafe { &mut *self.0.get() }).deref_mut()
    }
}

unsafe impl Sync for RunnableCell {}

pub trait RunnableSystem: Send + Sync {
    /// Run system
    ///
    /// # Safety
    ///
    /// Access to a resource can only have multiple reads or one write at the same time
    unsafe fn run(&mut self, world: &World);
}

impl<T> RunnableSystem for T
where
    for<'r> T: System<'r> + Send + Sync,
{
    unsafe fn run(&mut self, world: &World) {
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
