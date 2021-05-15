use serde::{Deserialize, Serialize};

use tb_ecs::*;

use crate::asset::AssetHandle;
use crate::prefab::Prefab;

#[derive(Deserialize, Serialize)]
pub struct Level {
    root: Prefab,
}

impl Level {
    pub fn attach(&self, world: &mut World) {
        self.root.attach(world);
    }

    pub fn create_entity(&mut self) -> EntityCreator {
        EntityCreator {
            prefab: &mut self.root,
        }
    }
}

pub struct EntityCreator<'p> {
    prefab: &'p mut Prefab,
}

#[derive(Default)]
pub struct LevelManager {
    pub current_level: Option<AssetHandle<Level>>,
    pub pending_level: Option<AssetHandle<Level>>,
}

impl LevelManager {
    pub fn request_switch(&mut self, level: AssetHandle<Level>) {
        self.pending_level.replace(level);
    }
}
