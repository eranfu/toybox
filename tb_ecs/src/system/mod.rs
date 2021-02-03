use crate::system::data::SystemData;

pub(crate) mod data;
pub(crate) mod registry;

pub trait System<'r> {
    type SystemData: SystemData<'r>;
    fn run(&mut self, system_data: Self::SystemData);
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[system]
    struct TestSystem {}

    impl System<'_> for TestSystem {
        type SystemData = ();

        fn run(&mut self, _system_data: Self::SystemData) {}
    }

    #[test]
    fn empty_system_data() {
        let mut scheduler = Scheduler::default();
        scheduler.insert(TestSystem {});
        let world = World::default();
        scheduler.schedule(&world);
    }
}
