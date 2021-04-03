use std::time::Duration;

use error::*;
use toybox::*;

error_chain! {
    links {
        Plugin(plugin::Error, plugin::ErrorKind);
    }
}

#[test]
fn hot_reload() {
    let mut plugin_manager = plugin::PluginManager::default();
    plugin_manager.load_plugin("script_ts");
    plugin_manager.load_plugin("example_pong");
}

mod load_ecs_info {
    use error::*;
    use toybox::*;

    error_chain! {
        links {
            Plugin(plugin::Error, plugin::ErrorKind);
            Topo(algorithm::topological_sort::Error, algorithm::topological_sort::ErrorKind);
        }
    }

    #[system]
    struct TestSystem {}

    impl<'s> System<'s> for TestSystem {
        type SystemData = ();

        fn run(&mut self, _system_data: Self::SystemData) {}
    }

    #[test]
    fn load_ecs_info() -> Result<()> {
        let mut plugin_manager = plugin::PluginManager::default();
        plugin_manager.load_plugin("script_ts");
        plugin_manager.load_plugin("example_pong");

        for system in SystemRegistry::systems() {
            let system = system?;
            println!("{}", system.0.name());
        }
        Ok(())
    }
}
