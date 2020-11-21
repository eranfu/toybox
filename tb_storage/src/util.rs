use std::ptr;

use tb_core::Id;

unsafe fn setup_base_id<T>(base_id: &mut Option<Id>, vec_with_base: &mut Vec<T>, id: Id) {
    match base_id.as_mut() {
        None => {
            *base_id = Some(id);
        }
        Some(base_id) => {
            if id < *base_id {
                // rebase
                let delta = (*base_id - id) as usize;
                vec_with_base.reserve(delta);
                let old_len = vec_with_base.len();
                vec_with_base.set_len(old_len + delta);
                ptr::copy(
                    vec_with_base.as_ptr(),
                    vec_with_base.as_mut_ptr().add(delta),
                    old_len,
                );
                *base_id = id;
            }
        }
    }
}

unsafe fn ensure_index<T>(vec: &mut Vec<T>, index: usize) {
    if vec.len() <= index {
        vec.reserve(index + 1 - vec.len());
        vec.set_len(index + 1);
    }
}

pub(crate) fn get_index_with_base(base_id: Option<Id>, id: Id) -> usize {
    id.checked_sub(base_id.unwrap()).unwrap() as usize
}

pub(crate) unsafe fn setup_index_with_base<T>(
    base_id: &mut Option<Id>,
    vec_with_base: &mut Vec<T>,
    id: Id,
) -> usize {
    setup_base_id(base_id, vec_with_base, id);
    let index = (id - base_id.unwrap()) as usize;
    ensure_index(vec_with_base, index);
    index
}
