use crate::entity::EntitiesIter;
use crate::join::ElementFetcher;
use crate::system::data::AccessOrder;
use crate::{Component, Components, Entity, Join, Storage};

pub struct AntiComponents<'r, S: 'r + Storage, C: Component, A: AccessOrder> {
    components: &'r Components<'r, S, C, A>,
}

impl<'r, S: 'r + Storage, C: Component, A: AccessOrder> AntiComponents<'r, S, C, A> {
    pub(crate) fn new(components: &'r Components<'r, S, C, A>) -> Self {
        Self { components }
    }
}

impl<'r, S: 'r + Storage, C: Component, A: AccessOrder> Join<'r> for AntiComponents<'r, S, C, A> {
    type ElementFetcher = AntiComponentsFetch<'r, S, C, A>;

    fn len(&self) -> usize {
        self.components.entities.len() - self.components.storage.len()
    }

    fn open(mut self) -> (Box<dyn Iterator<Item = Entity> + 'r>, Self::ElementFetcher) {
        (
            Box::new(AntiComponentsIterator {
                inner: self.components.entities.iter(),
                fetcher: self.elem_fetcher(),
            }),
            self.elem_fetcher(),
        )
    }

    fn elem_fetcher(&mut self) -> Self::ElementFetcher {
        AntiComponentsFetch {
            components: self.components,
        }
    }
}

pub struct AntiComponentsFetch<'r, S: 'r + Storage, C: Component, A: AccessOrder> {
    components: &'r Components<'r, S, C, A>,
}

impl<'r, S: 'r + Storage, C: Component, A: AccessOrder> ElementFetcher
    for AntiComponentsFetch<'r, S, C, A>
{
    type Element = ();

    fn fetch_elem(&mut self, entity: Entity) -> Option<Self::Element> {
        if self.contains(entity) {
            Some(())
        } else {
            None
        }
    }

    fn contains(&self, entity: Entity) -> bool {
        self.components.entities.contains(entity) && !self.components.storage.contains(entity)
    }
}

pub struct AntiComponentsIterator<'r, S: Storage, C: Component, A: AccessOrder> {
    inner: EntitiesIter<'r>,
    fetcher: AntiComponentsFetch<'r, S, C, A>,
}

impl<'r, S: Storage, C: Component, A: AccessOrder> Iterator
    for AntiComponentsIterator<'r, S, C, A>
{
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        let fetch = &self.fetcher;
        self.inner.find(|entity| fetch.contains(*entity))
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[component]
    struct Comp {}

    #[test]
    fn get_none() {
        let mut world = World::default();
        let entity = world.create_entity().with(Comp {}).create();
        let mut comps = WriteComponents::<Comp>::fetch(&world);
        let mut not_comps = !(&mut comps);
        assert_eq!(not_comps.elem_fetcher().fetch_elem(entity), None);
    }
}
