use std::env;
use std::path::PathBuf;

use tb_core::error::*;

error_chain! {
    errors {
        NonTargetDir
    }
}

pub fn target_dir() -> Result<PathBuf> {
    let mut exe = env::current_exe().chain_err(|| "Failed to get exe dir")?;
    if exe.pop() {
        Ok(exe)
    } else {
        Err(Error::from_kind(ErrorKind::NonTargetDir))
    }
}

#[cfg(test)]
mod tests {
    use crate::dir;

    #[test]
    fn target_dir() {
        println!("target dir: {:?}", dir::target_dir());
        println!("current dir: {:?}", std::env::current_dir());
    }
}
