use tb_core::AnyErrorResult;
use tb_plugin::PluginManager;

#[test]
fn it_works() -> AnyErrorResult<()> {
    let mut plugin_manager = PluginManager::default();
    plugin_manager.load_plugin("script_ts")?;
    plugin_manager.load_plugin("example_pong")?;
    Ok(())
}
