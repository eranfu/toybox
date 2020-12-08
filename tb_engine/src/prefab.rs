use bimap::BiHashMap;
use tb_core::Id;
use tb_ecs::*;

pub struct Prefab {
    root_entity: Id,
    components: Vec<Box<dyn Components>>,
}
trait Components {
    fn attach(&self, world: &mut World, link: &mut PrefabLink);
}

pub trait ComponentWithEntityRef: Component {
    fn convert_to_world() -> Self;
}

#[component]
#[derive(Default)]
struct PrefabLink {
    local_entity_to_world_map: BiHashMap<Id, Id>,
}

impl PrefabLink {
    fn insert_map(&mut self, local: Id, world: Id) {
        self.local_entity_to_world_map.insert(local, world);
    }
}

impl Prefab {
    pub(crate) fn attach(&self, world: &mut World) {
        let root_entity = world.create_entity().with(PrefabLink::default()).create();
        let mut link = PrefabLink::default();
        link.insert_map(self.root_entity, root_entity.id());
        link.insert_map(self.root_entity, root_entity.id());
        for components in &self.components {
            components.attach(world, &mut link);
        }
        let link_storage = world.fetch_storage_mut::<PrefabLink>();
        link_storage.insert(root_entity.id(), link);
    }
}
