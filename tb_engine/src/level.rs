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
