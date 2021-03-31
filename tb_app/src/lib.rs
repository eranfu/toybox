use std::{env, io, path};

use tb_core::error::AnyError;
use tb_ecs::{Scheduler, SystemInfo, World};
use tb_engine::level::LevelManager;

mod dir;
mod errors;
mod plugin;

pub struct Application {
    env_args: Vec<String>,
}

impl Application {
    pub fn new() -> Result<Application, AnyError> {
        Ok(Application {
            env_args: std::env::args().collect(),
        })
    }

    pub fn run(&mut self) {
        let mut world = World::default();
        loop {
            let level_manager: &mut LevelManager = world.insert(LevelManager::default);
            // world.insert_components()
            // if let Some(pending_level) = level_manager.pending_level.take() {
            //     world
            // }
            // world.up
        }
    }
}
