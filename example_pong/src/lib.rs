use toybox::plugin::*;

struct ExamplePong {}

impl Plugin for ExamplePong {
    fn name(&self) -> &'static str {
        "exit"
    }
}

declare_plugin!(ExamplePong {});
