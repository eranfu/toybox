use serde::{Deserialize, Serialize};

use errors::*;
use tb_ecs::*;

use crate::asset::prefab::Prefab;
use crate::asset::AssetHandle;
use crate::path::TbPath;

mod errors {
    use tb_core::error::*;

    error_chain! {}
}

#[derive(Deserialize, Serialize)]
pub struct Level {
    root: Prefab,
}

impl Level {
    pub fn create(path: &TbPath, world: &mut World) -> Result<Self> {
        Ok(Self {
            root: Prefab::create(path, world, None)
                .chain_err(|| "Failed to create root prefab.")?,
        })
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
