use crate::Entity;

pub trait Join<'j>: Sized {
    type ElementFetcher: ElementFetcher;

    fn join(self) -> JoinIterator<'j, Self> {
        let (entity_iter, elem_fetcher) = self.open();
        JoinIterator {
            entity_iter,
            elem_fetcher,
        }
    }
    fn open(self) -> (Box<dyn 'j + Iterator<Item = Entity>>, Self::ElementFetcher);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn elem_fetcher(&mut self) -> Self::ElementFetcher;
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

            #[allow(non_snake_case)]
            fn contains(&self, entity: Entity) -> bool {
                let ($j0, $($j1), +) = self;
                $j0.contains(entity) && $($j1.contains(entity)) && +
            }
        }

        impl<'j, $j0: Join<'j>, $($j1: Join<'j>), +> Join<'j> for ($j0, $($j1), +) {
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
                let ($j0, $(mut $j1), +) = self;
                let mut min_len = $j0.len();
                let (iter, $j0) = $j0.open();
                $(let (iter, $j1) = {
                    let cur_len = $j1.len();
                    if cur_len < min_len {
                        min_len = cur_len;
                        $j1.open()
                    } else {
                        (iter, $j1.elem_fetcher())
                    }
                });
                +;
                (iter, ($j0, $($j1), +))
            }

            #[allow(non_snake_case)]
            fn elem_fetcher(&mut self) -> Self::ElementFetcher {
                let ($j0, $($j1), +) = self;
                ($j0.elem_fetcher(), $($j1.elem_fetcher()), +)
            }
        }
    };
}

impl_join_tuple!(J0, J1, J2, J3, J4, J5, J6, J7);
