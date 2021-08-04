use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use errors::*;
use tb_ecs::*;

use crate::app_info::AppInfo;
use crate::asset::entity_instance::EntityInstance;
use crate::asset::AssetLoader;
use crate::hierarchy::{Children, Name, Parent, RecursiveChildrenIter};
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
    // todo:
    // pub(crate) fn create(dest_file: &TbPath, world: &mut World, root: Entity) -> Result<Prefab> {
    //     world.insert(AssetLoader::default);
    //     let asset_loader = unsafe { world.fetch_mut::<AssetLoader>() };
    //     let children_components = unsafe { world.fetch_components::<Children>() };
    //     let names = unsafe { world.fetch_components::<Name>() };
    //     let entities = RecursiveChildrenIter::new(children_components, names, root);
    //
    //     let extern_folder = dest_file
    //         .join_prefix_assets_based(AppInfo::extern_entity_dir_name())
    //         .chain_err(|| "Failed to get assets based path")?
    //         .to_absolute();
    //
    //     let parents = unsafe { world.fetch_components::<Parent>() };
    //     for (entity, path) in entities {
    //         let path = extern_folder.join(path);
    //
    //         asset_loader.save(path, EntityInstance::new())
    //     }
    //
    //     Ok(prefab)
    // }
}
