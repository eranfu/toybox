use std::collections::LinkedList;
use std::path::PathBuf;

use tb_ecs::*;

#[component]
pub struct Name {
    pub name: String,
}

#[component]
pub struct Parent {
    pub entity: Entity,
}

#[component]
pub struct Children {
    children: Vec<Entity>,
}

pub struct RecursiveChildrenIter<'s> {
    children_components: &'s ComponentStorage<Children>,
    names: &'s ComponentStorage<Name>,
    stack: LinkedList<(Entity, Option<std::slice::Iter<'s, Entity>>)>,
    path: PathBuf,
}

impl<'s> RecursiveChildrenIter<'s> {
    pub fn new(
        children_components: &'s ComponentStorage<Children>,
        names: &'s ComponentStorage<Name>,
        root: Entity,
    ) -> Self {
        let mut stack = LinkedList::new();
        stack.push_back((root, Self::get_children(children_components, root)));
        RecursiveChildrenIter {
            children_components,
            names,
            stack,
            path: Default::default(),
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
    type Item = (Entity, PathBuf);

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
                        let name = self.names.get(top_child).unwrap();
                        self.path.push(&name.name);
                        continue;
                    }
                },
            };
            let res = top.0;
            self.stack.pop_back();
            let path = {
                let path = self.path.clone();
                if self.path.pop() {
                    path
                } else {
                    "__ROOT__".into()
                }
            };
            return Some((res, path));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tb_ecs::*;

    use crate::hierarchy::{Children, Name, RecursiveChildrenIter};

    #[test]
    fn recursive_children_iter() {
        let root = Entity::new(0);
        let children_components = ComponentStorage::<Children>::default();
        let mut names = ComponentStorage::<Name>::default();
        names.insert(
            root,
            Name {
                name: "root".to_string(),
            },
        );
        let entities: Vec<(Entity, PathBuf)> =
            RecursiveChildrenIter::new(&children_components, &names, root).collect();
        assert_eq!(entities, vec![(root, "__ROOT__".into())])
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
