use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::lazy::{SyncLazy, SyncOnceCell};

use tb_core::algorithm::topological_sort::VisitorWithFlag;

use crate::scheduler::Runnable;
use crate::world::ResourceId;
use crate::*;

pub struct SystemRegistry {
    systems: Vec<&'static SystemInfo>,
    resources_info: HashMap<ResourceId, ResourceInfo>,
    system_topological_graph:
        tb_core::algorithm::topological_sort::TopologicalGraph<&'static SystemInfo>,
}

impl SystemRegistry {
    pub fn get_instance() -> &'static SystemRegistry {
        static SYSTEM_REGISTRY: SyncLazy<SystemRegistry> = SyncLazy::new(|| {
            let mut registry = SystemRegistry {
                systems: Default::default(),
                resources_info: Default::default(),
                system_topological_graph: Default::default(),
            };

            for system_info in inventory::iter::<SystemInfo> {
                registry.systems.push(system_info);
            }

            let resources_info = &mut registry.resources_info;
            registry.systems.iter().for_each(|system_info| {
                system_info
                    .reads_before_write
                    .iter()
                    .for_each(|resource_id| {
                        resources_info
                            .entry(*resource_id)
                            .or_insert_with(ResourceInfo::default)
                            .read_before_write_systems
                            .insert(system_info);
                    });
                system_info.writes.iter().for_each(|resource_id| {
                    resources_info
                        .entry(*resource_id)
                        .or_insert_with(ResourceInfo::default)
                        .write_systems
                        .insert(system_info);
                });
                system_info
                    .reads_after_write
                    .iter()
                    .for_each(|resource_id| {
                        resources_info
                            .entry(*resource_id)
                            .or_insert_with(ResourceInfo::default)
                            .read_after_write_systems
                            .insert(system_info);
                    });
            });

            let graph = &mut registry.system_topological_graph;
            registry.systems.iter().for_each(|system_info| {
                graph.add_item(system_info);
                system_info.writes.iter().for_each(|write_resource| {
                    let write_resource_info = resources_info.get(write_resource).unwrap();
                    write_resource_info
                        .read_before_write_systems
                        .iter()
                        .for_each(|read_before_write_system| {
                            graph.add_dependency(system_info, read_before_write_system);
                        });
                    write_resource_info
                        .read_after_write_systems
                        .iter()
                        .for_each(|read_after_write_system| {
                            graph.add_dependency(read_after_write_system, system_info);
                        });
                    write_resource_info
                        .write_systems
                        .iter()
                        .for_each(|write_system| {
                            graph.add_dependency_if_non_inverse(write_system, system_info);
                        })
                });
            });

            registry
        });
        &*SYSTEM_REGISTRY
    }

    pub fn systems() -> VisitorWithFlag<'static, &'static SystemInfo, usize> {
        SystemRegistry::get_instance()
            .system_topological_graph
            .visit_with_flag()
    }
}

#[derive(Default)]
pub struct ResourceInfo {
    read_before_write_systems: HashSet<&'static SystemInfo>,
    write_systems: HashSet<&'static SystemInfo>,
    read_after_write_systems: HashSet<&'static SystemInfo>,
}

pub struct SystemInfo {
    name: String,
    reads_before_write: Vec<ResourceId>,
    reads_after_write: Vec<ResourceId>,
    writes: Vec<ResourceId>,
    create: fn() -> Box<dyn Runnable>,
}

impl SystemInfo {
    pub fn new<S>() -> Self
    where
        for<'r> S: 'static + std::default::Default + System<'r>,
    {
        println!(
            "new system info. system type id: {:?}, name: {}",
            std::any::TypeId::of::<S>(),
            std::any::type_name::<S>()
        );

        Self {
            name: std::any::type_name::<S>().into(),
            reads_before_write: S::SystemData::reads_before_write(),
            reads_after_write: S::SystemData::reads_after_write(),
            writes: S::SystemData::writes(),
            create: || Box::new(S::default()),
        }
    }
}

impl PartialEq for &SystemInfo {
    fn eq(&self, other: &Self) -> bool {
        (*self as *const SystemInfo).eq(&(*other as *const SystemInfo))
    }
}

impl Eq for &SystemInfo {}

impl Hash for &SystemInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (*self as *const SystemInfo).hash(state);
    }
}

inventory::collect!(SystemInfo);

#[cfg(test)]
mod tests {
    use crate::*;

    #[system]
    struct TestSystem {
        _value: i32,
    }

    impl System<'_> for TestSystem {
        type SystemData = ();

        fn run(&mut self, _system_data: Self::SystemData) {}
    }

    #[test]
    fn it_works() {
        let mut has = false;
        for _x in SystemRegistry::systems() {
            has = true;
        }
        assert!(has);
        let mut has = false;
        for _x in SystemRegistry::systems() {
            has = true;
        }
        assert!(has);
    }
}
