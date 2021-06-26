use std::lazy::SyncLazy;
use std::path::PathBuf;

use errors::*;
use tb_core::path_util;

mod errors {
    use tb_core::error::*;

    error_chain! {}
}

pub enum LaunchMethod {
    Project { project_dir: PathBuf },
    Archive,
}

pub struct AppInfo {
    pub method: LaunchMethod,
    pub engine_root_dir: PathBuf,
    pub engine_assets_dir: PathBuf,
    pub project_root_dir: PathBuf,
    pub project_assets_dir: PathBuf,
}

impl AppInfo {
    pub fn get() -> &'static Self {
        static INSTANCE: SyncLazy<AppInfo> = SyncLazy::new(|| {
            let mut method = LaunchMethod::Archive;
            let mut args = std::env::args();
            args.next();
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--project" | "-p" => {
                        let project = args.next().unwrap();
                        method = LaunchMethod::Project {
                            project_dir: PathBuf::from(project),
                        }
                    }
                    arg => {
                        eprintln!("unknown argument: {}", arg);
                    }
                }
            }

            let project_root_dir = match &method {
                LaunchMethod::Project { project_dir } => project_dir.clone(),
                LaunchMethod::Archive => std::env::current_dir()
                    .chain_err(|| "Failed to get current_dir")
                    .unwrap(),
            };

            let engine_root_dir = path_util::exe_dir();
            AppInfo {
                method,
                engine_assets_dir: engine_root_dir.join(AppInfo::assets_dir_name()),
                engine_root_dir,
                project_assets_dir: project_root_dir.join(AppInfo::assets_dir_name()),
                project_root_dir,
            }
        });

        &INSTANCE
    }

    pub fn assets_dir_name() -> &'static str {
        "assets"
    }

    pub fn extern_entity_dir_name() -> &'static str {
        "__EXTERN_ENTITY__"
    }
}
