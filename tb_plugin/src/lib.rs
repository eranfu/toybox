#[macro_use]
extern crate error_chain;

use std::any::Any;
use std::ffi::OsStr;

use libloading::{Library, Symbol};
use log::debug;

use errors::*;

mod errors;

pub trait Plugin: Any + Send + Sync {
    fn name(&self) -> &'static str;
    fn on_load(&self) {}
    fn on_unload(&self) {}
}

#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:path) => {
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> *mut $crate::Plugin {
            // make sure the constructor is the correct type.
            let constructor: fn() -> $plugin_type = $constructor;
            let plugin = constructor();
            let plugin: Box<dyn $crate::Plugin> = Box::new(plugin);
            Box::into_raw(plugin)
        }
    };
}

#[derive(Default)]
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    loaded_libraries: Vec<Library>,
}

impl PluginManager {
    pub fn load_plugin(&mut self, filename: impl AsRef<OsStr>) -> Result<()> {
        type PluginCreate = fn() -> *mut dyn Plugin;
        let lib = unsafe { Library::new(filename).chain_err(|| "Failed to load library")? };
        self.loaded_libraries.push(lib);
        let lib = self.loaded_libraries.last().unwrap();
        let plugin_create: Symbol<PluginCreate> = unsafe {
            lib.get(b"_plugin_create")
                .chain_err(|| "Failed to find _plugin_create symbol")?
        };
        let plugin = plugin_create();
        unsafe { self.plugins.push(Box::from_raw(plugin)) }
        let plugin = self.plugins.last().unwrap();
        debug!("Loaded plugin: {}", plugin.name());
        plugin.on_load();
        Ok(())
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        debug!("Unloading plugins");
        for plugin in self.plugins.drain(..) {
            plugin.on_unload();
        }
        for library in self.loaded_libraries.drain(..) {
            drop(library);
        }
    }
}
