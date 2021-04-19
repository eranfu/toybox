use tb_app::Application;
use toybox::*;

error_chain! {}

fn main() -> Result<()> {
    let mut app = Application::default();
    app.run().chain_err(|| "Application run error")
}
