use rayon::iter::plumbing::UnindexedConsumer;
use rayon::iter::ParallelIterator;

use tb_core::*;

use crate::{ArchetypeMatcher, Entities, Entity, MatchedEntitiesIter, ParMatchedEntitiesIter};

pub trait Join<'j>: Sized {
    type Element: 'static;
    type ElementFetcher: ElementFetcher;
    type EntitiesIter: Iterator<Item = Entity>;
    type ParEntitiesIter: ParallelIterator<Item = Entity>;

    fn join(self) -> JoinIterator<'j, Self> {
        let (entity_iter, elem_fetcher) = self.open();
        JoinIterator {
            entity_iter,
            elem_fetcher,
        }
    }
    fn par_join(self) -> ParJoinIterator<'j, Self> {
        let (entity_iter, elem_fetcher) = self.par_open();
        ParJoinIterator {
            entity_iter,
            elem_fetcher,
        }
    }
    fn open(self) -> (Self::EntitiesIter, Self::ElementFetcher);
    fn par_open(self) -> (Self::ParEntitiesIter, Self::ElementFetcher);
    fn entities(&self) -> &'j Entities;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn elem_fetcher(&mut self) -> Self::ElementFetcher;
    fn matched_entities_iter(&self) -> Self::EntitiesIter;
    fn par_matched_entities_iter(&self) -> Self::ParEntitiesIter;
    fn create_matcher() -> ArchetypeMatcher {
        let mut matcher = ArchetypeMatcher::default();
        Self::fill_matcher(&mut matcher);
        matcher
    }
    fn fill_matcher(matcher: &mut ArchetypeMatcher);
}

pub trait ElementFetcher: Send {
    type Element: Send;
    fn fetch_elem(&mut self, entity: Entity) -> Option<Self::Element>;
}

pub struct JoinIterator<'j, J: Join<'j>> {
    entity_iter: J::EntitiesIter,
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

pub struct ParJoinIterator<'j, J: Join<'j>> {
    entity_iter: J::ParEntitiesIter,
    elem_fetcher: J::ElementFetcher,
}

impl<'j, J: Join<'j>> ParallelIterator for ParJoinIterator<'j, J> {
    type Item = <<J as Join<'j>>::ElementFetcher as ElementFetcher>::Element;

    fn drive_unindexed<C>(self, _consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        todo!()
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
            type EntitiesIter = MatchedEntitiesIter<'j>;
            type ParEntitiesIter = ParMatchedEntitiesIter<'j>;

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
            fn open(self) -> (Self::EntitiesIter, Self::ElementFetcher) {
                let matched_entities = self.matched_entities_iter();
                let (mut $j0, $(mut $j1), +) = self;
                let ($j0, $($j1), +) = ($j0.elem_fetcher(), $($j1.elem_fetcher()), +);
                (matched_entities, ($j0, $($j1), +))
            }

            #[allow(unused_assignments)]
            #[allow(non_snake_case)]
            fn par_open(self) -> (Self::ParEntitiesIter, Self::ElementFetcher) {
                let matched_entities = self.par_matched_entities_iter();
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

            fn matched_entities_iter(&self) -> Self::EntitiesIter {
                MatchedEntitiesIter::get::<Self>(self.entities().read())
            }

            fn par_matched_entities_iter(&self) -> Self::ParEntitiesIter {
                ParMatchedEntitiesIter::get::<Self>(self.entities().read())
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
