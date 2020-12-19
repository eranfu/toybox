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

pub trait EntityRef {
    fn convert_to_world(&mut self, link: &mut PrefabLink, entities: &mut Entities);
}

pub trait ComponentWithEntityRef: Component {
    type Ref: EntityRef;
    fn get_entity_ref(&mut self) -> Self::Ref;
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
        let storage = world.fetch_storage_mut::<C>();
        let entities = world.fetch_mut::<Entities>();
        mask.iter().map(Id::from).for_each(|id| {
            link.insert(id, entities);
        });
        mask.iter().map(Id::from).for_each(|id| {
            storage.insert(
                link.get_entity_in_world(id).id(),
                unsafe { components.get(id) }.clone(),
            );
        });
    }
}

impl<C> Components for ComponentStorage<C>
where
    C: ComponentWithEntityRef,
{
    fn attach(&self, world: &mut World, link: &mut PrefabLink) {
        let (mask, components) = self.open();
        world.insert_storage::<C>();
        let storage = world.fetch_storage_mut::<C>();
        let entities = world.fetch_mut::<Entities>();
        mask.iter().map(Id::from).for_each(|id| {
            let mut component = unsafe { components.get(id) }.clone();
            let mut entity_ref = component.get_entity_ref();
            link.insert(id, entities);
            entity_ref.convert_to_world(link, entities);
            storage.insert(link.get_entity_in_world(id).id(), component);
        });
    }
}

impl PrefabLink {
    fn get_entity_in_world(&self, local: Id) -> Entity {
        *self.local_entity_to_world_map.get_by_left(&local).unwrap()
    }

    fn insert(&mut self, local: Id, entities: &mut Entities) -> Entity {
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
        link.insert(self.root_entity, world.fetch_mut());
        for components in &self.components {
            components.attach(world, &mut link);
        }
        let link_storage = world.fetch_storage_mut::<PrefabLink>();
        link_storage.insert(link.get_entity_in_world(self.root_entity).id(), link);
    }
}
