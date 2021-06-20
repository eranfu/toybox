use std::path::PathBuf;

use tb_core::path_util;

use crate::app_info::AppInfo;

#[derive(Copy, Clone)]
enum TbPathBase {
    Absolute,
    EngineRoot,
    EngineAssets,
    ProjectRoot,
    ProjectAssets,
}

#[derive(Clone)]
pub struct TbPath {
    base: TbPathBase,
    path: PathBuf,
}

impl TbPath {
    pub fn new_project_assets(path: impl Into<PathBuf>) -> Self {
        Self {
            base: TbPathBase::ProjectAssets,
            path: path.into(),
        }
    }
    pub fn to_absolute(&self) -> PathBuf {
        let base = match self.base {
            TbPathBase::Absolute => return self.path.clone(),
            TbPathBase::EngineRoot => &AppInfo::get().engine_root_dir,
            TbPathBase::EngineAssets => &AppInfo::get().engine_assets_dir,
            TbPathBase::ProjectRoot => &AppInfo::get().project_root_dir,
            TbPathBase::ProjectAssets => &AppInfo::get().project_assets_dir,
        };
        base.join(&self.path)
    }

    pub fn join_prefix_assets_based(&self, prefix: impl Into<PathBuf>) -> Option<TbPath> {
        self.to_assets_based().map(|mut assets_based| {
            let mut prefix = prefix.into();
            std::mem::swap(&mut prefix, &mut assets_based.path);
            assets_based.path.push(prefix);
            assets_based
        })
    }

    fn to_assets_based(&self) -> Option<TbPath> {
        match self.base {
            TbPathBase::Absolute => {
                let app_info = AppInfo::get();
                match path_util::pre_cut(&self.path, &app_info.project_assets_dir) {
                    None => path_util::pre_cut(&self.path, &app_info.engine_assets_dir).map(
                        |engine_assets_based_path| TbPath {
                            base: TbPathBase::EngineAssets,
                            path: engine_assets_based_path,
                        },
                    ),
                    Some(project_assets_based_path) => Some(TbPath {
                        base: TbPathBase::ProjectAssets,
                        path: project_assets_based_path,
                    }),
                }
            }
            TbPathBase::EngineRoot => path_util::pre_cut(&self.path, AppInfo::assets_dir_name())
                .map(|path| TbPath {
                    base: TbPathBase::EngineAssets,
                    path,
                }),
            TbPathBase::EngineAssets => Some(self.clone()),
            TbPathBase::ProjectRoot => path_util::pre_cut(&self.path, AppInfo::assets_dir_name())
                .map(|path| TbPath {
                    base: TbPathBase::ProjectAssets,
                    path,
                }),
            TbPathBase::ProjectAssets => Some(self.clone()),
        }
    }
}

impl From<TbPath> for PathBuf {
    fn from(from: TbPath) -> Self {
        let base = match from.base {
            TbPathBase::Absolute => return from.path,
            TbPathBase::EngineRoot => &AppInfo::get().engine_root_dir,
            TbPathBase::EngineAssets => &AppInfo::get().engine_assets_dir,
            TbPathBase::ProjectRoot => &AppInfo::get().project_root_dir,
            TbPathBase::ProjectAssets => &AppInfo::get().project_assets_dir,
        };
        base.join(from.path)
    }
}
