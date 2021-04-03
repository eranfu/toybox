use toybox::plugin::*;

struct ScriptTs {}

impl Plugin for ScriptTs {
    fn name(&self) -> &'static str {
        "script_t"
    }
}

declare_plugin!(ScriptTs {});
