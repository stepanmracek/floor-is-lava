use bevy::prelude::*;

pub fn get_top_parent(mut curr_entity: Entity, parent_query: &Query<&Parent>) -> Entity {
    debug!("Looking for parent of {:?}", curr_entity);
    while let Ok(parent) = parent_query.get(curr_entity) {
        curr_entity = parent.get();
    }
    curr_entity
}
