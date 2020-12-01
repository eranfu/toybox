use std::collections::HashMap;

use tb_ecs::Resource;

pub struct Prefab {
    resources: HashMap<String, Box<dyn Resource>>,
}
