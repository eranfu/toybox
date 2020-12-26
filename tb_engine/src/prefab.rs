use bimap::BiHashMap;
use hibitset::BitSetLike;

use tb_core::Id;
use tb_ecs::*;

pub struct Prefab {
    root_entity: Id,
    components: Vec<Box<dyn Components>>,
}

trait Components {
    fn attach(&self, world: &mut World, link: &mut PrefabLink);
}

pub trait ConvertToWorld {
    fn convert_to_world(&mut self, link: &mut PrefabLink, entities: &mut Entities);
}

pub trait ComponentWithEntityRef<'e>: Component {
    type RefMut: 'e + ConvertToWorld;
    fn get_entity_ref(&'e mut self) -> Self::RefMut;
}

#[component]
#[derive(Default)]
pub struct PrefabLink {
    local_entity_to_world_map: BiHashMap<Id, Entity>,
}

impl<C> Components for ComponentStorage<C>
where
    C: Component,
{
    default fn attach(&self, world: &mut World, link: &mut PrefabLink) {
        let (mask, components) = self.open();
        world.insert_storage::<C>();
        world.insert(Entities::default);
        let storage = world.fetch_storage_mut::<C>();
        let entities = world.fetch_mut::<Entities>();
        mask.iter().map(Id::from).for_each(|id| {
            storage.insert(
                link.build_link(id, entities).id(),
                unsafe { components.get(id) }.clone(),
            );
        });
    }
}

impl<C> Components for ComponentStorage<C>
where
    for<'e> C: ComponentWithEntityRef<'e>,
{
    fn attach(&self, world: &mut World, link: &mut PrefabLink) {
        let (mask, components) = self.open();
        world.insert_storage::<C>();
        let storage = world.fetch_storage_mut::<C>();
        let entities = world.fetch_mut::<Entities>();
        mask.iter().map(Id::from).for_each(|id| {
            let mut component: C = unsafe { components.get(id) }.clone();
            {
                let mut entity_ref = component.get_entity_ref();
                entity_ref.convert_to_world(link, entities);
            }
            storage.insert(link.build_link(id, entities).id(), component);
        });
    }
}

impl PrefabLink {
    fn build_link(&mut self, local: Id, entities: &mut Entities) -> Entity {
        match self.local_entity_to_world_map.get_by_left(&local) {
            None => {
                let entity = entities.new_entity();
                match self
                    .local_entity_to_world_map
                    .insert_no_overwrite(local, entity)
                {
                    Ok(_) => {}
                    Err(_) => unreachable!(),
                }
                entity
            }
            Some(entity) => *entity,
        }
    }
}

impl Prefab {
    pub(crate) fn attach(&self, world: &mut World) {
        let mut link = PrefabLink::default();
        for components in &self.components {
            components.attach(world, &mut link);
        }
        world.insert(Entities::default);
        world.insert_storage::<PrefabLink>();
        world.fetch_storage_mut::<PrefabLink>().insert(
            link.build_link(self.root_entity, world.fetch_mut()).id(),
            link,
        );
    }
}

impl<'e> ConvertToWorld for &'e mut Entity {
    fn convert_to_world(&mut self, link: &mut PrefabLink, entities: &mut Entities) {
        **self = link.build_link(self.id(), entities);
    }
}

macro_rules! convert_to_world_tuple {
    ($c:ident) => {};
    ($c0:ident, $($c1:ident), +) => {
        convert_to_world_tuple!($($c1), +);
        impl<$c0: ConvertToWorld, $($c1: ConvertToWorld), +> ConvertToWorld for ($c0, $($c1), +) {
            #[allow(non_snake_case)]
            fn convert_to_world(&mut self, link: &mut PrefabLink, entities: &mut Entities) {
                let ($c0, $($c1), +) = self;
                $c0.convert_to_world(link, entities);
                $($c1.convert_to_world(link, entities)); +;
            }
        }
    };
}

convert_to_world_tuple!(C0, C1, C2, C3, C4, C5, C6, C7);

#[cfg(test)]
mod tests {
    use tb_ecs::*;

    use crate::prefab::{ComponentWithEntityRef, Prefab};

    #[component]
    struct Component0 {
        value: i32,
    }

    #[component]
    struct Component1 {
        entity_a: Entity,
    }

    #[component]
    struct Component2 {
        entity_a: Entity,
        entity_b: Entity,
    }

    impl<'e> ComponentWithEntityRef<'e> for Component1 {
        type RefMut = &'e mut Entity;

        fn get_entity_ref(&'e mut self) -> Self::RefMut {
            &mut self.entity_a
        }
    }

    impl<'e> ComponentWithEntityRef<'e> for Component2 {
        type RefMut = (&'e mut Entity, &'e mut Entity);

        fn get_entity_ref(&'e mut self) -> Self::RefMut {
            (&mut self.entity_a, &mut self.entity_b)
        }
    }

    #[test]
    fn convert_to_world() {
        let entities: Vec<Entity> = {
            let mut world = World::default();
            let entities = world.insert(Entities::default);
            (0..16).map(|_i| entities.new_entity()).collect()
        };

        let prefab = {
            let mut components0 = ComponentStorage::<Component0>::default();
            components0.insert(10.into(), Component0 { value: 10 });
            let mut components1 = ComponentStorage::<Component1>::default();
            components1.insert(
                7.into(),
                Component1 {
                    entity_a: entities[15],
                },
            );
            let mut components2 = ComponentStorage::<Component2>::default();
            components2.insert(
                15.into(),
                Component2 {
                    entity_a: entities[7],
                    entity_b: entities[10],
                },
            );
            let components: Vec<Box<dyn crate::prefab::Components>> = vec![
                Box::new(components0),
                Box::new(components1),
                Box::new(components2),
            ];
            Prefab {
                root_entity: 15.into(),
                components,
            }
        };

        let mut world = World::default();
        prefab.attach(&mut world);

        let (components0, components1, components2) = <(
            RAWComponents<Component0>,
            RAWComponents<Component1>,
            RAWComponents<Component2>,
        ) as SystemData>::fetch(&world);
        for component0 in components0.join() {
            let component0: &Component0 = component0;
            assert_eq!(component0.value, 10);
        }
        for component1 in components1.join() {
            let component1: &Component1 = component1;
            assert_eq!(component1.entity_a.id(), 1.into());
        }
        for component2 in components2.join() {
            let component2: &Component2 = component2;
            assert_eq!(component2.entity_a.id(), 2.into());
            assert_eq!(component2.entity_b.id(), 0.into());
        }
    }
}
