use std::collections::LinkedList;

use tb_ecs::*;

#[component]
pub struct Parent {
    parent: Entity,
}

#[component]
pub struct Children {
    children: Vec<Entity>,
}

pub struct RecursiveChildrenIter<'s> {
    children_components: &'s ComponentStorage<Children>,
    stack: LinkedList<(Entity, Option<std::slice::Iter<'s, Entity>>)>,
}

impl<'s> RecursiveChildrenIter<'s> {
    fn new(children_components: &'s ComponentStorage<Children>, root: Entity) -> Self {
        let mut stack = LinkedList::new();
        stack.push_back((root, Self::get_children(children_components, root)));
        RecursiveChildrenIter {
            children_components,
            stack,
        }
    }

    fn get_children(
        children_components: &'s ComponentStorage<Children>,
        entity: Entity,
    ) -> Option<std::slice::Iter<'s, Entity>> {
        children_components
            .get(entity)
            .map(|children| children.children.iter())
    }
}

impl<'s> Iterator for RecursiveChildrenIter<'s> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let top = match self.stack.back_mut() {
                None => {
                    return None;
                }
                Some(top) => top,
            };
            match &mut top.1 {
                None => {}
                Some(top_children) => match top_children.next() {
                    None => {}
                    Some(&top_child) => {
                        self.stack.push_back((
                            top_child,
                            Self::get_children(self.children_components, top_child),
                        ));
                        continue;
                    }
                },
            };
            let res = top.0;
            self.stack.pop_back();
            return Some(res);
        }
    }
}

#[cfg(test)]
mod tests {
    use tb_ecs::*;

    use crate::transform::{Children, RecursiveChildrenIter};

    #[test]
    fn recursive_children_iter() {
        let root = Entity::new(0);
        let children_components = ComponentStorage::<Children>::default();
        let entities: Vec<Entity> =
            RecursiveChildrenIter::new(&children_components, root).collect();
        assert_eq!(entities, vec![root])
    }

    #[test]
    fn recursive_children_iter_a() {
        let entities: Vec<Entity> = (0..100).into_iter().map(Entity::new).collect();
        let mut children_components = ComponentStorage::<Children>::default();
        children_components.insert(
            entities[0],
            Children {
                children: (1..10).map(|i| entities[i]).collect(),
            },
        );

        assert_eq!(
            RecursiveChildrenIter::new(&children_components, entities[0]).collect::<Vec<Entity>>(),
            (1..10)
                .map(|i| entities[i])
                .chain(vec![entities[0]])
                .collect::<Vec<Entity>>()
        );

        children_components.insert(
            entities[1],
            Children {
                children: (11..20).map(|i| entities[i]).collect(),
            },
        );
        children_components.insert(
            entities[5],
            Children {
                children: (21..30).map(|i| entities[i]).collect(),
            },
        );
        assert_eq!(
            RecursiveChildrenIter::new(&children_components, entities[0]).collect::<Vec<Entity>>(),
            (11..20)
                .map(|i| entities[i])
                .chain((1..5).map(|i| entities[i]))
                .chain((21..30).map(|i| entities[i]))
                .chain((5..10).map(|i| entities[i]))
                .chain(vec![entities[0]])
                .collect::<Vec<Entity>>()
        )
    }
}
