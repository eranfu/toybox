use toybox::*;

#[test]
fn it_works() -> AnyErrorResult<()> {
    let mut plugin_manager = PluginManager::default();
    plugin_manager.load_plugin("script_ts")?;
    plugin_manager.load_plugin("example_pong")?;
    Ok(())
}

mod load_ecs_info {
    use toybox::*;

    #[system]
    struct TestSystem {}

    impl<'s> System<'s> for TestSystem {
        type SystemData = ();

        fn run(&mut self, _system_data: Self::SystemData) {}
    }

    #[test]
    fn load_ecs_info() -> AnyErrorResult<()> {
        let mut plugin_manager = PluginManager::default();
        plugin_manager.load_plugin("script_ts")?;
        plugin_manager.load_plugin("example_pong")?;

        for system in SystemRegistry::systems() {
            let system = system?;
            println!("{}", system.0.name());
        }
        Ok(())
    }
}
