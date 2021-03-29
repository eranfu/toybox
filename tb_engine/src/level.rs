use tb_ecs::*;

use crate::asset::AssetHandle;
use crate::prefab::Prefab;

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
