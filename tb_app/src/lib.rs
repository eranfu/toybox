use tb_ecs::World;
use tb_engine::level::LevelManager;

pub mod dir;

pub struct Application {
    env_args: Vec<String>,
}

impl Application {
    pub fn new() -> Application {
        Application {
            env_args: std::env::args().collect(),
        }
    }

    pub fn run(&mut self) {
        let mut world = World::default();
        loop {
            let _level_manager: &mut LevelManager = world.insert(LevelManager::default);
            // world.insert_components()
            // if let Some(pending_level) = level_manager.pending_level.take() {
            //     world
            // }
            // world.up
        }
    }
}
