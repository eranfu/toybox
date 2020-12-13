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
    fn build_link(&self, link: &mut PrefabLink, entities: &mut Entities);
    fn convert_to_world(&mut self, link: &PrefabLink);
}

pub trait ComponentWithEntityRef: Component {
    type Ref: EntityRef;
    fn get_entity_ref(&self) -> Self::Ref;
    fn get_entity_ref_mut(&mut self) -> Self::Ref;
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
        let entities = world.fetch_mut::<Entities>();
        mask.iter().map(Id::from).for_each(|id| {
            link.insert(id, entities);
        });
        let storage = world.fetch_or_insert_storage::<C>();
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
        let entities = world.fetch_mut::<Entities>();
        mask.iter().map(Id::from).for_each(|id| {
            let component: &C = unsafe { components.get(id) };
            component.get_entity_ref().build_link(link, entities);
            link.insert(id, entities);
        });
        let storage = world.fetch_or_insert_storage::<C>();
        mask.iter().map(Id::from).for_each(|id| {
            let mut component: C = unsafe { components.get(id) }.clone();
            component.get_entity_ref_mut().convert_to_world(link);
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
