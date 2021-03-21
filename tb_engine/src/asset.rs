use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

pub trait Assets {}

pub struct AssetsOf<T> {
    assets: HashMap<u64, Box<T>>,
}

impl<T> Assets for AssetsOf<T> {}

pub struct AssetHandle<T> {
    id: u64,
    _phantom: PhantomData<T>,
}

impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _phantom: self._phantom,
        }
    }
}

impl<T> Copy for AssetHandle<T> {}

pub struct AssetLoader {
    type_to_assets: HashMap<TypeId, Box<dyn Assets>>,
}

impl AssetLoader {
    pub fn get<T: 'static>(&self, handle: AssetHandle<T>) -> Option<&Box<T>> {
        self.type_to_assets
            .get(&TypeId::of::<T>())
            .and_then(|assets| {
                let assets: &AssetsOf<T> = unsafe { std::mem::transmute(&**assets) };
                assets.assets.get(&handle.id)
            })
    }
}
