pub use data::*;
pub use registry::*;

mod data;
mod registry;

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
}
