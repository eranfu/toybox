use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use errors::*;
use tb_ecs::*;

use crate::app_info::AppInfo;
use crate::asset::AssetLoader;
use crate::path::TbPath;
use crate::transform::Children;

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
pub struct Prefab {
    extern_folder: PathBuf,
}

impl Prefab {
    pub(crate) fn create_asset(
        dest_file: &TbPath,
        world: &mut World,
        root: Option<Entity>,
    ) -> Result<Prefab> {
        let prefab = std::fs::File::create(dest_file.to_absolute())
            .chain_err(|| "Failed to create prefab file.")?;
        world.insert(AssetLoader::default);

        let entities = match root {
            None => {}
            Some(_) => {}
        };
        let (asset_loader, children_components) =
            unsafe { <(Write<AssetLoader>, RAWComponents<Children>) as SystemData>::fetch(world) };

        let extern_folder = dest_file
            .join_prefix_assets_based(AppInfo::extern_dir_name())
            .chain_err(|| "Failed to get assets based path")?;

        bail!("unimplemented")
    }
}
