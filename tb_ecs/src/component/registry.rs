use std::borrow::Borrow;
use std::lazy::SyncLazy;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::thread;

use crate::{Component, Entity, World};

pub struct ComponentRegistry {
    infos: Mutex<Vec<ComponentInfo>>,
}

impl ComponentRegistry {
    fn get_instance() -> &'static ComponentRegistry {
        static INSTANCE: SyncLazy<ComponentRegistry> = SyncLazy::new(|| ComponentRegistry {
            infos: Default::default(),
        });
        &*INSTANCE
    }
}


const STATE_UNINITIALIZED: u8 = 0;
const STATE_INITIALIZING: u8 = 1;
const STATE_COMPLETED: u8 = 2;

pub struct ComponentIndex {}

impl ComponentIndex {
    pub fn get<C: Component>() -> usize {
        static STATE: AtomicU8 = AtomicU8::new(STATE_UNINITIALIZED);
        static INDEX: MaybeUninit<usize> = MaybeUninit::uninit();
        loop {
            match STATE.compare_exchange(STATE_UNINITIALIZED, STATE_INITIALIZING, Ordering::Acquire, Ordering::Acquire) {
                Ok(STATE_UNINITIALIZED) => {}
                Err(STATE_COMPLETED) => {
                    break;
                }
                Err(STATE_INITIALIZING) => {
                    thread::yield_now()
                }
                _ => { unreachable!() }
            }
        }

        let registry = ComponentRegistry::get_instance();
        let mut infos = &mut *registry.infos.lock().unwrap();
        let mut index = infos
        INDEX.compare_exchange(usize::MAX, infos.len())
        let index = infos.len();
        infos.push(ComponentInfo::new::<C>());
        index * INDEX
    }
}

pub trait ComponentOperation {
    fn remove_from_world(&self, world: &mut World, entity: Entity);
}

struct Operation<C: Component> {
    _phantom: PhantomData<C>,
}

impl<C: Component> ComponentOperation for Operation<C> {
    fn remove_from_world(&self, world: &mut World, entity: Entity) {
        unimplemented!()
    }
}

pub struct ComponentInfo {
    operation: Box<dyn ComponentOperation>,
}

impl ComponentInfo {
    fn new<C: Component>() -> Self {
        Self {
            operation: Box::new(Operation::<C> {
                _phantom: Default::default(),
            }),
        }
    }
}

inventory::collect!(ComponentInfo);
