use inventory::iter;

use crate::scheduler::Runnable;
use crate::world::ResourceId;
use crate::*;

pub struct SystemRegistry;

impl SystemRegistry {
    pub fn iter() -> iter<SystemInfo> {
        inventory::iter::<SystemInfo>
    }
}

pub struct SystemInfo {
    name: String,
    reads_before_write: Vec<ResourceId>,
    reads_after_write: Vec<ResourceId>,
    writes: Vec<ResourceId>,
    create: fn() -> Box<dyn Runnable>,
}

impl SystemInfo {
    fn new<S>() -> Self
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
            reads_after_write: S::SystemData::reads_before_write(),
            writes: S::SystemData::writes(),
            create: || Box::new(S::default()),
        }
    }
}

inventory::collect!(SystemInfo);

#[cfg(test)]
mod tests {
    use crate::*;

    #[system]
    struct TestSystem {
        value: i32,
    }

    impl System<'_> for TestSystem {
        type SystemData = ();

        fn run(&mut self, _system_data: Self::SystemData) {}
    }

    #[test]
    fn it_works() {
        let mut has = false;
        for x in SystemRegistry::iter() {
            has = true;
            assert_eq!(x.name, std::any::type_name::<TestSystem>())
        }
        assert!(has);
        let mut has = false;
        for x in SystemRegistry::iter() {
            has = true;
            assert_eq!(x.name, std::any::type_name::<TestSystem>())
        }
        assert!(has);
    }
}
