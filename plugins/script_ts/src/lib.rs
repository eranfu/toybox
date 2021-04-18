use toybox::*;

struct ScriptTs {}

#[system]
struct TestSystem {}

impl<'s> System<'s> for TestSystem {
    type SystemData = ();

    fn run(&mut self, _system_data: Self::SystemData) {}
}

impl Plugin for ScriptTs {
    fn name(&self) -> &'static str {
        "script_t"
    }
}

declare_plugin!(ScriptTs {});
