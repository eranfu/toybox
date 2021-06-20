use std::env;
use std::path::{Path, PathBuf};

pub fn exe_dir() -> PathBuf {
    let mut exe = env::current_exe().unwrap();
    assert!(exe.pop());
    exe
}

pub fn pre_cut(absolute: impl AsRef<Path>, pre: impl AsRef<Path>) -> Option<PathBuf> {
    let mut absolute_components = absolute.as_ref().components();
    let mut pre_components = pre.as_ref().components();
    loop {
        let relative_component = match pre_components.next() {
            None => {
                break;
            }
            Some(next) => next,
        };

        let self_component = match absolute_components.next() {
            None => {
                return None;
            }
            Some(next) => next,
        };

        if relative_component != self_component {
            return None;
        }
    }

    let mut res = PathBuf::new();
    for component in absolute_components {
        res.push(component);
    }
    Some(res)
}

#[cfg(test)]
mod tests {
    use crate::path_util;

    #[test]
    fn target_dir() {
        println!("target dir: {:?}", path_util::exe_dir());
        println!("current dir: {:?}", std::env::current_dir());
    }
}
