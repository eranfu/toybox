use std::any::Any;
use std::path::PathBuf;

use live_lib::{LibPartner, Library, Loader, Symbol};

use errors::*;
use tb_ecs::*;

mod errors {
    pub use tb_core::error::*;

    error_chain! {}
}

pub trait Plugin: Any + Send + Sync {
    fn name(&self) -> &'static str;
    fn on_load(&self) {}
    fn on_unload(&self) {}
    fn system_infos(&self) -> Box<dyn Iterator<Item = &'static SystemInfo>> {
        Box::new(inventory::iter::<SystemInfo>.into_iter())
    }
    fn component_infos(&self) -> Box<dyn Iterator<Item = &'static ComponentInfo>> {
        Box::new(inventory::iter::<ComponentInfo>.into_iter())
    }
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

impl LibPartner for Box<dyn Plugin> {
    type LoadResult = Result<Self>;
    type UnloadResult = Result<()>;

    fn load(lib: &Library) -> Self::LoadResult {
        type PluginCreate = fn() -> Box<dyn Plugin>;
        let plugin_create: Symbol<PluginCreate> = unsafe {
            lib.get(b"_plugin_create")
                .chain_err(|| "Failed to find _plugin_create symbol".to_owned())?
        };
        let plugin: Box<dyn Plugin> = plugin_create();
        println!("Loaded plugin: {}", plugin.name());
        plugin.on_load();
        SystemRegistry::add_system_infos(plugin.system_infos());
        ComponentRegistry::add_component_infos(plugin.component_infos());
        Ok(plugin)
    }

    fn unload(&mut self, _lib: &Library) -> Self::UnloadResult {
        let name = self.name().to_owned();
        self.on_unload();
        println!("Unloaded plugin: {}", name);
        Ok(())
    }
}

pub struct PluginManager {
    loader: Loader<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new(additional_search_dirs: Vec<PathBuf>) -> Self {
        Self {
            loader: Loader::new(additional_search_dirs).unwrap(),
        }
    }

    pub fn add_search_dir(&mut self, dir: PathBuf) {
        self.loader.add_search_dir(dir)
    }

    pub fn update(&mut self) {
        self.loader.update().unwrap();
    }

    pub fn add_plugin(&mut self, lib_name: &str) {
        self.loader
            .add_library(lib_name)
            .chain_err(|| "Failed to load library")
            .unwrap()
    }

    pub fn get_plugin(&self, lib_name: &str) -> Option<&dyn Plugin> {
        self.loader
            .get(lib_name)
            .map(|(_lib, plugin)| plugin.as_ref())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new(vec![])
    }
}

#[system]
struct PluginSystem;

impl<'s> System<'s> for PluginSystem {
    type SystemData = Write<'s, PluginManager>;

    fn run(&mut self, mut plugin_manager: Self::SystemData) {
        plugin_manager.update()
    }
}
