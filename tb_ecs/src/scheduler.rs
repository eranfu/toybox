use crate::{System, SystemData, World};

pub struct Scheduler {
    systems: Vec<Box<dyn RunnableSystem>>,
}

impl Scheduler {
    pub fn update(&self, world: &World) {}
}

impl Default for Scheduler {
    fn default() -> Self {
        Self { systems: vec![] }
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
