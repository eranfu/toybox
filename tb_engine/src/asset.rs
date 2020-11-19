use std::marker::PhantomData;
use std::rc::Rc;

pub trait Asset {}

pub struct Assets<A: Asset> {
    assets: Vec<Option<A>>,
}

pub struct AssetHandle<A: Asset> {
    index: usize,
    _phantom: PhantomData<A>,
}

impl<A: Asset> Assets<A> {
    pub fn get(&self, handle: AssetHandle<A>) -> Option<&A> {
        let a = self.assets.get(handle.index);
        match a {
            None => None,
            Some(a) => a.as_ref(),
        }
    }
}
