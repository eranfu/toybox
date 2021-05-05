use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use tb_core::error::*;
use tb_ecs::World;
use tb_plugin::PluginManager;

pub mod dir;

error_chain! {}

enum LaunchMethod {
    Project { project_dir: PathBuf },
    Archive,
}

pub struct Application {
    method: LaunchMethod,
}

impl Application {
    pub fn run() -> Result<()> {
        let mut app = Self::default();
        let mut world = World::default();
        app.setup_project(&mut world)?;
        app.main_loop(&mut world);
        Ok(())
    }

    fn setup_project(&mut self, world: &mut World) -> Result<()> {
        let plugin_manager: &mut PluginManager = world.insert(PluginManager::default);
        if let LaunchMethod::Project { project_dir } = &self.method {
            if !project_dir.exists() {
                bail!("project not exists. path: {:?}", project_dir)
            }

            Command::new("cargo")
                .current_dir(project_dir)
                .arg("build")
                .status()
                .chain_err(|| "Failed to build project")?;
            let project_lib_dir = if cfg!(debug_assertions) {
                project_dir.join("target/debug")
            } else {
                project_dir.join("target/release")
            };
            plugin_manager.add_search_dir(project_lib_dir);
            plugin_manager.add_plugin(project_dir.file_name().unwrap().to_str().unwrap())
        }

        Ok(())
    }

    fn main_loop(&mut self, world: &mut World) {
        loop {
            self.prepare_systems(world);
            // world.insert_components()
            // if let Some(pending_level) = level_manager.pending_level.take() {
            //     world
            // }
            // world.up
            std::thread::sleep(Duration::from_secs_f32(1f32 / 30f32))
        }
    }

    fn prepare_systems(&mut self, world: &mut World) {}
}

impl Default for Application {
    fn default() -> Self {
        let mut method = LaunchMethod::Archive;
        let mut args = std::env::args();
        args.next();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--project" | "-p" => {
                    let project = args.next().unwrap();
                    method = LaunchMethod::Project {
                        project_dir: PathBuf::from(project),
                    }
                }
                arg => {
                    eprintln!("unknown argument: {}", arg);
                }
            }
        }
        Application { method }
    }
}
