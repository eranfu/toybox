use std::any::Any;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use live_lib::{Lib, Loader, Symbol};

use tb_core::error::*;

error_chain! {}

pub trait Plugin: Any + Send + Sync {
    fn name(&self) -> &'static str;
    fn on_load(&self) {}
    fn on_unload(&self) {}
}

#[macro_export]
macro_rules! declare_plugin {
    ($plugin:expr) => {
        #[no_mangle]
        pub fn _plugin_create() -> Box<dyn Plugin> {
            Box::new($plugin)
        }
    };
}

pub struct PluginManager {
    loader: Loader<HashMap<String, Box<dyn Plugin>>, Result<()>>,
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn update(&mut self) {
        self.loader.update(&mut self.plugins);
    }

    pub fn load_plugin(&mut self, filename: &str) -> Result<()> {
        self.loader
            .add_library(filename, &mut self.plugins)
            .chain_err(|| "Failed to load library")
    }

    fn post_load(plugins: &mut HashMap<String, Box<dyn Plugin>>, lib: &Lib) -> Result<()> {
        type PluginCreate = fn() -> Box<dyn Plugin>;

        match plugins.entry(lib.name().clone()) {
            Entry::Occupied(_occupied) => {
                bail!(format!("The lib is already loaded. name: {}", lib.name()))
            }
            Entry::Vacant(vacant) => {
                let plugin_create: Symbol<PluginCreate> = unsafe {
                    lib.lib()
                        .get(b"_plugin_create")
                        .chain_err(|| "Failed to find _plugin_create symbol".to_owned())?
                };
                let plugin = plugin_create();
                println!("Loaded plugin: {}", plugin.name());
                vacant.insert(plugin).on_load();
                Ok(())
            }
        }
    }

    fn pre_unload(plugins: &mut HashMap<String, Box<dyn Plugin>>, lib: &Lib) -> Result<()> {
        if let Some(plugin) = plugins.remove(lib.name()) {
            plugin.on_unload();
        }
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self {
            loader: Loader::new(Self::post_load, Self::pre_unload).unwrap(),
            plugins: Default::default(),
        }
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        println!("Unloading plugins");
        for (_, plugin) in self.plugins.drain() {
            plugin.on_unload();
        }
    }
}
