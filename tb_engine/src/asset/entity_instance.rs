use std::path::PathBuf;

use tb_core::serde::*;
use tb_ecs::Component;

#[derive(Serialize, Deserialize)]
pub struct EntityInstance {
    header: EntityInstanceHeader,
    data: PrefabOrInstance,
}

#[derive(Serialize, Deserialize)]
enum PrefabOrInstance {
    Prefab {
        prefab_path: EntityPath,
        modifies: Vec<SerdeBox<dyn PrefabModifier>>,
    },
    Instance {
        components: Vec<SerdeBox<dyn Component>>,
    },
}

#[serde_box]
trait PrefabModifier: SerdeBoxSer + SerdeBoxDe {}

#[derive(Serialize, Deserialize)]
struct EntityPath {
    prefab_file: PathBuf,
    entity: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct EntityInstanceHeader {
    bounds: Option<tb_physics::bounds::Bounds>,
}
