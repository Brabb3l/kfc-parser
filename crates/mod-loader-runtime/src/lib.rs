use mod_loader::ModEnvironment;

pub struct RuntimeOptions {}

pub fn loader_attach(
    _env: &ModEnvironment,
    _options: RuntimeOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("loader_attach called");

    Ok(())
}

pub fn loader_detach() -> Result<(), Box<dyn std::error::Error>> {
    println!("loader_detach called");

    Ok(())
}
