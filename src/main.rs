use tb_app::Application;
use tb_core::error::*;

error_chain! {}

fn main() -> Result<()> {
    Application::run().chain_err(|| "App running error")
}
