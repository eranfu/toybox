use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_box::*;

use tb_ecs::*;

#[component]
#[derive(Default)]
pub struct PrefabLink {
    link: LocalToWorldLink,
}

#[derive(Deserialize, Serialize)]
pub struct Prefab {
    root_entity: Entity,
    components: Vec<SerdeBox<dyn ComponentStorageTrait>>,
}

impl Prefab {
    pub(crate) fn from_world(world: &World) -> Self {}
    pub(crate) fn attach(&self, world: &mut World) {
        let mut link = PrefabLink::default();
        for components in &self.components {
            components.attach_to_world(world, &mut link.link);
        }
        world.insert(Entities::default);
        world.insert_components::<PrefabLink>();
        let mut prefab_links = unsafe { WriteComponents::<PrefabLink>::fetch(world) };
        unsafe {
            prefab_links.insert(
                link.link.build_link(self.root_entity, world.fetch_mut()),
                link,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_box::*;

    use tb_ecs::*;

    use crate::prefab::{ComponentStorageInPrefab, ComponentWithEntityRef, Prefab};

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

    #[test]
    fn convert_to_world() {
        let entities: Vec<Entity> = {
            let mut world = World::default();
            let entities = world.insert(Entities::default);
            (0..16).map(|_i| entities.new_entity()).collect()
        };

        let prefab = {
            let mut components0 = ComponentStorageInPrefab::<Component0>::default();
            components0.insert(Entity::new(10), Component0 { value: 10 });
            let mut components1 = ComponentStorageInPrefab::<Component1>::default();
            components1.insert(
                Entity::new(7),
                Component1 {
                    entity_a: entities[15],
                },
            );
            let mut components2 = ComponentStorageInPrefab::<Component2>::default();
            components2.insert(
                Entity::new(15),
                Component2 {
                    entity_a: entities[7],
                    entity_b: entities[10],
                },
            );
            let components: Vec<SerdeBox<dyn crate::prefab::ComponentStorageTrait>> = vec![
                SerdeBox(Box::new(components0)),
                SerdeBox(Box::new(components1)),
                SerdeBox(Box::new(components2)),
            ];
            Prefab {
                root_entity: Entity::new(15),
                components,
            }
        };

        let mut world = World::default();
        prefab.attach(&mut world);

        let (components0, components1, components2) = unsafe {
            <(
                RAWComponents<Component0>,
                RAWComponents<Component1>,
                RAWComponents<Component2>,
            ) as SystemData>::fetch(&world)
        };
        for component0 in components0.join() {
            let component0: &Component0 = component0;
            assert_eq!(component0.value, 10);
        }
        for component1 in components1.join() {
            let component1: &Component1 = component1;
            assert_eq!(component1.entity_a, Entity::new(1));
        }
        for component2 in components2.join() {
            let component2: &Component2 = component2;
            assert_eq!(component2.entity_a, Entity::new(2));
            assert_eq!(component2.entity_b, Entity::new(0));
        }
    }
}
