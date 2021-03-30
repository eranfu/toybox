use toybox::plugin::*;

struct ScriptTs {}

impl Plugin for ScriptTs {
    fn name(&self) -> &'static str {
        "script_ts"
    }
}

declare_plugin!(ScriptTs {});
