use tb_ecs::World;

use crate::prefab::Prefab;

pub struct Level {
    root: Prefab,
}

impl Level {
    pub fn attach(&self, world: &mut World) {
        self.root.attach(world);
    }
}
