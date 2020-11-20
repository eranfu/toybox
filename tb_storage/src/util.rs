use std::ptr;

use tb_core::Id;

pub(crate) unsafe fn setup_base_id<T>(
    base_id: &mut Option<Id>,
    vec_with_base_id: &mut Vec<T>,
    new_id: Id,
) {
    match base_id.as_mut() {
        None => {
            *base_id = Some(new_id);
        }
        Some(base_id) => {
            if new_id < *base_id {
                // rebase
                let delta = (*base_id - new_id) as usize;
                vec_with_base_id.reserve(delta);
                let old_len = vec_with_base_id.len();
                vec_with_base_id.set_len(old_len + delta);
                ptr::copy(
                    vec_with_base_id.as_ptr(),
                    vec_with_base_id.as_mut_ptr().add(delta),
                    old_len,
                );
                *base_id = new_id;
            }
        }
    }
}

pub(crate) unsafe fn ensure_index<T>(vec: &mut Vec<T>, index: usize) {
    if vec.len() <= index {
        vec.reserve(index + 1 - vec.len());
        vec.set_len(index + 1);
    }
}
