extern crate tb_app;
extern crate tb_core;

use tb_core::AnyErrorResult;

#[cfg(not(test))]
fn main() -> AnyErrorResult<()> {
    let mut app = tb_app::Application::new()?;
    app.run();
    Ok(())
}
