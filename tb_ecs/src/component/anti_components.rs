use std::marker::PhantomData;

use crate::*;

pub struct AntiComponents<'r, S: 'r + Storage, C: Component, A: AccessOrder> {
    components: &'r Components<'r, S, C, A>,
}

impl<'r, S: 'r + Storage, C: Component, A: AccessOrder> AntiComponents<'r, S, C, A> {
    pub(crate) fn new(components: &'r Components<'r, S, C, A>) -> Self {
        Self { components }
    }
}

pub struct AntiComponent<C: Component> {
    _phantom: PhantomData<C>,
}

impl<'r, S: 'r + Storage, C: Component, A: AccessOrder> Join<'r> for AntiComponents<'r, S, C, A> {
    type Element = AntiComponent<C>;
    type ElementFetcher = AntiComponentsFetch<'r, S, C, A>;

    fn open(mut self) -> (Box<dyn 'r + Iterator<Item = Entity>>, Self::ElementFetcher) {
        (self.get_matched_entities(), self.elem_fetcher())
    }

    fn entities(&self) -> &'r Entities {
        self.components.entities
    }

    fn len(&self) -> usize {
        self.components.entities.len() - self.components.storage.len()
    }

    fn elem_fetcher(&mut self) -> Self::ElementFetcher {
        AntiComponentsFetch {
            components: self.components,
        }
    }

    fn get_matched_entities(&self) -> Box<dyn 'r + Iterator<Item = Entity>> {
        let storage = &self.components.storage;
        Box::new(
            self.entities()
                .iter()
                .filter(move |&entity| !storage.contains(entity)),
        )
    }

    fn fill_matcher(matcher: &mut ArchetypeMatcher) {
        matcher.add_none(ComponentIndex::get::<C>())
    }
}

pub struct AntiComponentsFetch<'r, S: 'r + Storage, C: Component, A: AccessOrder> {
    components: &'r Components<'r, S, C, A>,
}

impl<'r, S: 'r + Storage, C: Component, A: AccessOrder> ElementFetcher
    for AntiComponentsFetch<'r, S, C, A>
{
    type Element = ();

    fn fetch_elem(&mut self, _entity: Entity) -> Option<Self::Element> {
        if !self.components.storage.contains(_entity) && self.components.entities.is_alive(_entity)
        {
            Some(())
        } else {
            None
        }
    }
}

impl<'r, S: 'r + Storage, C: Component, A: AccessOrder> Clone for AntiComponentsFetch<'r, S, C, A> {
    fn clone(&self) -> Self {
        Self {
            components: self.components,
        }
    }
}

impl<'r, S: 'r + Storage, C: Component, A: AccessOrder> Copy for AntiComponentsFetch<'r, S, C, A> {}

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
