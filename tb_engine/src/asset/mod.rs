use std::any::Any;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use errors::*;
use tb_ecs::*;

use crate::path::TbPath;

pub mod entity_instance;
pub mod prefab;

mod errors {
    pub use tb_core::error::*;

    error_chain! {}
}

#[derive(Copy, Clone)]
pub struct AssetHandle<T> {
    id: u64,
    _phantom: PhantomData<T>,
}

pub type AssetArc = Arc<SerdeBox<dyn Asset>>;

#[serde_box]
pub trait Asset: Any + Send + Sync + SerdeBoxSer + SerdeBoxDe {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any + Send + Sync + SerdeBoxSer + SerdeBoxDe> Asset for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

type PendingReceiver = Arc<Mutex<(Receiver<PathBuf>, Receiver<(PathBuf, AssetArc)>)>>;

type IdToPendingChannel = HashMap<
    u64,
    (
        Sender<PathBuf>,
        Sender<(PathBuf, AssetArc)>,
        PendingReceiver,
    ),
>;

type CompletedAssetsChannel = (
    Sender<(u64, Result<AssetArc>)>,
    Receiver<(u64, Result<AssetArc>)>,
);

pub struct AssetLoader {
    id_to_assets: HashMap<u64, AssetArc>,
    path_to_ids: HashMap<PathBuf, u64>,
    next_id: u64,
    threads: thread_pool::ThreadPool,
    id_to_pending_channel: IdToPendingChannel,
    completed_assets_channel: CompletedAssetsChannel,
}

///
/// # Safety
///
/// Don't use `completed_assets_channel` and `id_to_pending_channel` in immutable methods
unsafe impl Sync for AssetLoader {}

impl AssetLoader {
    pub fn load<T: Asset>(&mut self, path: TbPath) -> AssetHandle<T> {
        let id_to_pending_channel = &mut self.id_to_pending_channel;
        let id = match self.path_to_ids.entry(path.into()) {
            Entry::Occupied(occupied) => *occupied.get(),
            Entry::Vacant(vacant) => {
                let id = self.next_id;
                let path = vacant.key().clone();
                vacant.insert(id);
                self.next_id += 1;
                let (pending_load_sender, _, pending_receiver) =
                    Self::pending_channel_of_id(id_to_pending_channel, id);
                pending_load_sender.send(path).unwrap();

                let pending_receiver = pending_receiver.clone();
                let completed_sender = self.completed_assets_channel.0.clone();
                self.threads.execute(move || {
                    Self::process_pending_task(id, pending_receiver, completed_sender)
                });
                id
            }
        };
        AssetHandle {
            id,
            _phantom: Default::default(),
        }
    }

    pub fn save<T: Asset>(&mut self, path: TbPath, asset: Box<T>) -> AssetHandle<T> {
        let id = match self.path_to_ids.entry(path.into()) {
            Entry::Occupied(occupied) => *occupied.get(),
            Entry::Vacant(vacant) => {
                let id = self.next_id;
                vacant.insert(id);
                self.next_id += 1;
                id
            }
        };
        let asset: AssetArc = Arc::new(SerdeBox(asset as Box<dyn Asset>));
        self.id_to_assets.insert(id, asset.clone());

        AssetHandle {
            id,
            _phantom: Default::default(),
        }
    }

    pub fn update(&mut self) {
        let id_to_assets = &mut self.id_to_assets;
        self.completed_assets_channel
            .1
            .try_iter()
            .for_each(|(id, asset)| {
                match asset {
                    Ok(asset) => {
                        id_to_assets.insert(id, asset);
                    }
                    Err(e) => {
                        eprintln!("{}", e.display_chain());
                        id_to_assets.remove(&id);
                    }
                };
            });
    }

    pub fn get<T: 'static>(&self, handle: AssetHandle<T>) -> Option<&T> {
        match self.id_to_assets.get(&handle.id) {
            None => None,
            Some(asset) => asset.as_any().downcast_ref(),
        }
    }

    fn save_block(path: impl AsRef<Path>, asset: AssetArc) -> Result<AssetArc> {
        let path = path.as_ref();
        let file = match Self::open_file(path) {
            Ok(file) => file,
            Err(e) => {
                return Err(e);
            }
        };
        serde_json::to_writer(file, asset.deref())
            .chain_err(|| format!("Failed to serialize asset. path: {:?}", path))?;
        Ok(asset)
    }

    fn load_block(path: impl AsRef<Path>) -> Result<AssetArc> {
        let path = path.as_ref();
        let file = Self::open_file(path)?;
        let res: AssetArc = Arc::new(
            serde_json::from_reader(file)
                .chain_err(|| format!("Failed to deserialize asset. path: {:?}", path))?,
        );
        Ok(res)
    }

    fn open_file(path: &Path) -> Result<File> {
        std::fs::File::open(path)
            .chain_err(|| format!("Failed to open asset file. path: {:?}", path))
    }

    fn process_pending_task(
        id: u64,
        pending_receiver: PendingReceiver,
        completed_sender: Sender<(u64, Result<AssetArc>)>,
    ) {
        let pending_receiver = match pending_receiver.lock() {
            Ok(receiver) => receiver,
            Err(e) => {
                completed_sender
                    .send((
                        id,
                        Err(Error::with_chain(
                            Error::from(e.to_string()),
                            "Failed to lock pending_receiver.",
                        )),
                    ))
                    .unwrap();
                return;
            }
        };
        let (pending_load_receiver, pending_save_receiver) = pending_receiver.deref();
        if let Some((path, asset)) = pending_save_receiver.try_iter().last() {
            pending_load_receiver.try_iter().last();
            completed_sender
                .send((id, Self::save_block(path, asset)))
                .unwrap();
            return;
        }

        if let Some(path) = pending_load_receiver.try_iter().last() {
            completed_sender.send((id, Self::load_block(path))).unwrap();
        }
    }

    fn pending_channel_of_id(
        id_to_pending_channel: &mut IdToPendingChannel,
        id: u64,
    ) -> &(
        Sender<PathBuf>,
        Sender<(PathBuf, AssetArc)>,
        PendingReceiver,
    ) {
        id_to_pending_channel.entry(id).or_insert_with(|| {
            let (pending_load_sender, pending_load_receiver) = channel();
            let (pending_save_sender, pending_save_receiver) = channel();
            (
                pending_load_sender,
                pending_save_sender,
                Arc::new(Mutex::new((pending_load_receiver, pending_save_receiver))),
            )
        })
    }
}

impl Default for AssetLoader {
    fn default() -> Self {
        Self {
            id_to_assets: Default::default(),
            path_to_ids: Default::default(),
            next_id: 0,
            threads: Default::default(),
            id_to_pending_channel: Default::default(),
            completed_assets_channel: channel(),
        }
    }
}

#[system]
struct LoadAssetSystem {}

impl<'s> System<'s> for LoadAssetSystem {
    type SystemData = Write<'s, AssetLoader>;

    fn run(&mut self, mut asset_loader: Self::SystemData) {
        asset_loader.update()
    }
}
