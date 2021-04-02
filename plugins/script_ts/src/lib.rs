use toybox::plugin::*;

struct ScriptTs {}

impl Plugin for ScriptTs {
    fn name(&self) -> &'static str {
        "script_t hot"
    }
}

declare_plugin!(ScriptTs {});
