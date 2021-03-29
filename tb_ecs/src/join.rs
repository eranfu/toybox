use std::any::TypeId;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::iter::FlatMap;
use std::lazy::SyncLazy;
use std::slice::Iter;
use std::sync::{RwLock, RwLockReadGuard};

use crate::{
    ArchetypeIndex, ArchetypeVisitor, ComponentIndex, ComponentMask, Entities, EntitiesInner,
    Entity,
};

pub trait Join<'j>: Sized {
    type Element: 'static;
    type ElementFetcher: ElementFetcher;

    fn join(self) -> JoinIterator<'j, Self> {
        let (entity_iter, elem_fetcher) = self.open();
        JoinIterator {
            entity_iter,
            elem_fetcher,
        }
    }
    fn open(self) -> (Box<dyn 'j + Iterator<Item = Entity>>, Self::ElementFetcher);
    fn entities(&self) -> &'j Entities;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn elem_fetcher(&mut self) -> Self::ElementFetcher;

    fn get_matched_entities(&self) -> Box<dyn 'j + Iterator<Item = Entity>>;
    fn create_matcher() -> ArchetypeMatcher {
        let mut matcher = ArchetypeMatcher {
            all: Default::default(),
            none: Default::default(),
        };
        Self::fill_matcher(&mut matcher);
        matcher
    }
    fn fill_matcher(matcher: &mut ArchetypeMatcher);
}

pub struct ArchetypeMatcher {
    all: ComponentMask,
    none: ComponentMask,
    // todo: any: ComponentMask,
}

impl ArchetypeMatcher {
    pub(crate) fn add_all(&mut self, component_index: ComponentIndex) {
        self.all.insert(*component_index);
    }
    pub(crate) fn add_none(&mut self, component_index: ComponentIndex) {
        self.none.insert(*component_index);
    }
    pub(crate) fn is_matched(&self, archetype_component_mask: &ComponentMask) -> bool {
        self.all.is_subset(archetype_component_mask)
            && self.none.is_disjoint(archetype_component_mask)
    }
}

struct MatchedEntities {
    matcher: ArchetypeMatcher,
    archetype_visitor: ArchetypeVisitor,
    matched_archetypes: Vec<ArchetypeIndex>,
}

impl MatchedEntities {
    fn get<'j, T: Join<'j>>(
        entities: &RwLockReadGuard<EntitiesInner>,
    ) -> (
        RwLockReadGuard<'static, HashMap<TypeId, RwLock<MatchedEntities>>>,
        RwLockReadGuard<'static, MatchedEntities>,
    ) {
        static CACHE: SyncLazy<RwLock<HashMap<TypeId, RwLock<MatchedEntities>>>> =
            SyncLazy::new(Default::default);
        let cache = &*CACHE;
        let t_id = TypeId::of::<T::Element>();
        let cache_read = cache.read().unwrap();
        let (cache, matched): (_, &'static RwLock<MatchedEntities>) =
            if let Some(matched) = cache_read.get(&t_id) {
                let matched = unsafe { std::mem::transmute(matched) };
                (cache_read, matched)
            } else {
                drop(cache_read);
                let mut cache_write = cache.write().unwrap();
                match cache_write.entry(t_id) {
                    Entry::Occupied(_occupied) => {}
                    Entry::Vacant(vacant) => {
                        vacant.insert(RwLock::new(MatchedEntities {
                            matcher: T::create_matcher(),
                            archetype_visitor: ArchetypeVisitor {
                                cur_index: ArchetypeIndex(0),
                            },
                            matched_archetypes: vec![],
                        }));
                    }
                }
                drop(cache_write);
                let cache_read = cache.read().unwrap();
                let matched = unsafe { std::mem::transmute(cache_read.get(&t_id).unwrap()) };
                (cache_read, matched)
            };
        let matched_read = matched.read().unwrap();
        let matched = if *matched_read.archetype_visitor.cur_index
            >= entities.archetypes_component_mask.len()
        {
            matched_read
        } else {
            drop(matched_read);
            let mut matched_write = matched.write().unwrap();
            matched_write.do_match(entities);
            drop(matched_write);
            let matched_read = matched.read().unwrap();
            matched_read
        };

        (cache, matched)
    }

    fn do_match(&mut self, entities: &RwLockReadGuard<EntitiesInner>) {
        let matcher = &self.matcher;
        let matched_archetypes = &mut self.matched_archetypes;
        entities.visit_archetype(
            &mut self.archetype_visitor,
            |archetype, archetype_component_mask| {
                if matcher.is_matched(archetype_component_mask) {
                    matched_archetypes.push(archetype);
                }
            },
        );
    }
}

struct MatchedEntitiesIter<'e> {
    _entities: RwLockReadGuard<'e, EntitiesInner>,
    _matched_entities_cache: RwLockReadGuard<'static, HashMap<TypeId, RwLock<MatchedEntities>>>,
    _matched_entities: RwLockReadGuard<'static, MatchedEntities>,
    matched:
        FlatMap<Iter<'e, ArchetypeIndex>, Iter<'e, Entity>, fn(&ArchetypeIndex) -> Iter<Entity>>,
}

impl<'e> MatchedEntitiesIter<'e> {
    fn get<'j, T: Join<'j>>(entities: RwLockReadGuard<EntitiesInner>) -> MatchedEntitiesIter {
        let (matched_entities_cache, matched_entities) = MatchedEntities::get::<T>(&entities);
        let matched = unsafe {
            std::mem::transmute(
                matched_entities
                    .matched_archetypes
                    .iter()
                    .flat_map(|&archetype| entities.archetypes_entities[archetype].iter()),
            )
        };
        MatchedEntitiesIter {
            _entities: entities,
            _matched_entities_cache: matched_entities_cache,
            _matched_entities: matched_entities,
            matched,
        }
    }
}

impl<'e> Iterator for MatchedEntitiesIter<'e> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.matched.next().copied()
    }
}

pub trait ElementFetcher {
    type Element;
    fn fetch_elem(&mut self, entity: Entity) -> Option<Self::Element>;
}

pub struct JoinIterator<'j, J: Join<'j>> {
    entity_iter: Box<dyn 'j + Iterator<Item = Entity>>,
    elem_fetcher: J::ElementFetcher,
}

impl<'j, J: Join<'j>> Iterator for JoinIterator<'j, J> {
    type Item = <<J as Join<'j>>::ElementFetcher as ElementFetcher>::Element;

    fn next(&mut self) -> Option<Self::Item> {
        let fetch = &mut self.elem_fetcher;
        self.entity_iter
            .next()
            .map(|entity| fetch.fetch_elem(entity).unwrap())
    }
}

macro_rules! impl_join_tuple {
    ($j:ident) => {};
    ($j0:ident, $($j1:ident), +) => {
        impl_join_tuple!($($j1), +);

        impl<$j0: ElementFetcher, $($j1: ElementFetcher), +> ElementFetcher for ($j0, $($j1), +) {
            type Element = ($j0::Element, $($j1::Element), +);

            #[allow(non_snake_case)]
            fn fetch_elem(&mut self, entity: Entity) -> Option<Self::Element> {
                let ($j0, $($j1), +) = self;
                let $j0 = $j0.fetch_elem(entity)?;
                $(let $j1 = $j1.fetch_elem(entity)?);
                +;
                Some(($j0, $($j1), +))
            }
        }

        impl<'j, $j0: Join<'j>, $($j1: Join<'j>), +> Join<'j> for ($j0, $($j1), +) {
            type Element = ($j0::Element, $($j1::Element), +);
            type ElementFetcher = ($j0::ElementFetcher, $($j1::ElementFetcher), +);

            #[allow(non_snake_case)]
            fn len(&self) -> usize {
                let ($j0, $($j1), +) = self;
                let res = $j0.len();
                $(let res = res.min($j1.len()));
                +;
                res
            }

            #[allow(unused_assignments)]
            #[allow(non_snake_case)]
            fn open(self) -> (Box<dyn Iterator<Item = Entity> + 'j>, Self::ElementFetcher) {
                let matched_entities = self.get_matched_entities();
                let (mut $j0, $(mut $j1), +) = self;
                let ($j0, $($j1), +) = ($j0.elem_fetcher(), $($j1.elem_fetcher()), +);
                (matched_entities, ($j0, $($j1), +))
            }

            #[allow(non_snake_case)]
            fn entities(&self) -> &'j Entities {
                let ($j0, ..) = self;
                $j0.entities()
            }

            #[allow(non_snake_case)]
            fn elem_fetcher(&mut self) -> Self::ElementFetcher {
                let ($j0, $($j1), +) = self;
                ($j0.elem_fetcher(), $($j1.elem_fetcher()), +)
            }

            fn get_matched_entities(&self) -> Box<dyn 'j + Iterator<Item = Entity>> {
                Box::new(MatchedEntitiesIter::get::<Self>(self.entities().read()))
            }

            fn fill_matcher(matcher: &mut ArchetypeMatcher) {
                $j0::fill_matcher(matcher);
                $($j1::fill_matcher(matcher));
                +
            }
        }
    };
}

impl_join_tuple!(J0, J1, J2, J3, J4, J5, J6, J7);
