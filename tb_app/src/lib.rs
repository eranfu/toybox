use std::process::Command;
use std::time::{Duration, Instant};

use errors::*;
use tb_ecs::*;
use tb_engine::app_info::{AppInfo, LaunchMethod};
use tb_engine::asset::AssetLoader;
use tb_engine::level::{Level, LevelManager};
use tb_engine::path::TbPath;
use tb_plugin::PluginManager;

mod errors {
    pub use tb_core::error::*;

    error_chain! {}
}

#[derive(Default)]
pub struct Application {}

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
        let app_info = AppInfo::get();
        match &app_info.method {
            LaunchMethod::Project { project_dir } => {
                if !project_dir.exists() {
                    bail!("project not exists. path: {:?}", project_dir);
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

        if !app_info.project_assets_dir.exists() {
            std::fs::create_dir_all(&app_info.project_assets_dir).chain_err(|| {
                format!(
                    "Failed to create asset dir: {:?}",
                    app_info.project_assets_dir
                )
            })?;
        }
        Ok(())
    }

    fn setup_entry_level(&self, world: &mut World) -> Result<()> {
        let path = TbPath::new_project_assets("levels/entry.tbasset");
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
