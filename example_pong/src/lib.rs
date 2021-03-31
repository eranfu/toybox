use toybox::*;

struct ExamplePong {}

impl Plugin for ExamplePong {
    fn name(&self) -> &'static str {
        "example_pong"
    }
}

declare_plugin!(ExamplePong {});
