use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use errors::*;
use tb_ecs::*;

use crate::app_info::AppInfo;
use crate::asset::AssetLoader;
use crate::hierarchy::{Children, Parent, RecursiveChildrenIter};
use crate::path::TbPath;

mod errors {
    pub use tb_core::error::*;

    error_chain! {}
}

#[component]
#[derive(Default)]
pub struct PrefabLink {
    link: LocalToWorldLink,
}

#[derive(Deserialize, Serialize)]
pub struct Prefab {}

impl Prefab {
    pub(crate) fn create(
        dest_file: &TbPath,
        world: &mut World,
        root: Option<Entity>,
    ) -> Result<Prefab> {
        world.insert(AssetLoader::default);
        let asset_loader = unsafe { world.fetch_mut::<AssetLoader>() };
        let children_components = unsafe { world.fetch_components::<Children>() };

        let entities: Box<dyn Iterator<Item = Entity>> = match root {
            None => {
                let entities = unsafe { world.fetch::<Entities>() };
                Box::new(entities.iter())
            }
            Some(root) => Box::new(RecursiveChildrenIter::new(children_components, root)),
        };

        let extern_folder = dest_file
            .join_prefix_assets_based(AppInfo::extern_entity_dir_name())
            .chain_err(|| "Failed to get assets based path")?
            .to_absolute();

        for entity in entities {
            asset_loader.save()
        }
    }

    fn entity_path_base_on_root(parents: ComponentStorage<Parent>, entity: Entity, root: Entity) {}
}
