use toybox::*;

error_chain! {}

fn main() -> Result<()> {
    Application::run().chain_err(|| "App running error")
}
