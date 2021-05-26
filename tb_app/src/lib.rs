use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use tb_core::error::*;
use tb_ecs::*;
use tb_engine::asset::AssetLoader;
use tb_engine::level::{Level, LevelManager};
use tb_plugin::PluginManager;

pub mod dir;

error_chain! {}

enum LaunchMethod {
    Project { project_dir: PathBuf },
    Archive,
}

pub struct Application {
    method: LaunchMethod,
    project_root_dir: PathBuf,
    project_asset_dir: PathBuf,
}

impl Application {
    pub fn run() -> Result<()> {
        let mut app = Self::default();
        let mut world = World::default();
        app.setup_project(&mut world)?;
        app.setup_entry_level(&mut world)?;
        app.main_loop(&mut world);
        Ok(())
    }

    fn setup_project(&mut self, world: &mut World) -> Result<()> {
        let plugin_manager: &mut PluginManager = world.insert(PluginManager::default);
        match &self.method {
            LaunchMethod::Project { project_dir } => {
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
            LaunchMethod::Archive => {}
        }

        if !self.project_asset_dir.exists() {
            std::fs::create_dir_all(&self.project_asset_dir).chain_err(|| {
                format!("Failed to create asset dir: {:?}", self.project_asset_dir)
            })?;
        }
        Ok(())
    }

    fn setup_entry_level(&self, world: &mut World) -> Result<()> {
        let path = self.project_asset_dir.join("levels/entry.tblevel");
        world.insert(LevelManager::default);
        world.insert(AssetLoader::default);
        let (mut level_manager, mut asset_loader) =
            unsafe { <(Write<LevelManager>, Write<AssetLoader>)>::fetch(world) };
        let level = asset_loader.load::<Level>(path);
        level_manager.request_switch(level);
        Ok(())
    }

    fn main_loop(&mut self, world: &mut World) {
        let mut scheduler = Scheduler::new(world);
        const FPS: f32 = 30f32;
        let frame_duration = Duration::from_secs_f32(1f32 / FPS);
        loop {
            let start = Instant::now();

            scheduler.update(world);

            let elapsed = start.elapsed();
            if frame_duration > elapsed {
                let should_sleep = frame_duration - elapsed;
                std::thread::sleep(should_sleep);
            } else {
                std::thread::yield_now();
            }
        }
    }
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

        let project_root_dir = match &method {
            LaunchMethod::Project { project_dir } => project_dir.clone(),
            LaunchMethod::Archive => std::env::current_dir()
                .chain_err(|| "Failed to get current_dir")
                .unwrap(),
        };
        Application {
            method,
            project_asset_dir: project_root_dir.join("assets"),
            project_root_dir,
        }
    }
}
