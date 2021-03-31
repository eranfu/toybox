use std::env;
use std::path::PathBuf;

use crate::errors::*;

fn target_dir() -> Result<PathBuf> {
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
    use crate::errors::*;

    #[test]
    fn target_dir() -> Result<()> {
        println!("{:?}", dir::target_dir()?);
        Ok(())
    }
}
