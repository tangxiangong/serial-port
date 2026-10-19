use interoptopus_csharp::{RustLibrary, dispatch::Dispatch};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .ok_or("Pass an output directory")?;
    std::fs::create_dir_all(&output)?;
    RustLibrary::builder(serial_studio_csharp::inventory())
        .dll_name("serial_studio_csharp")
        .dispatch(Dispatch::single_file("SerialStudio.Core"))
        .build()
        .process()?
        .write_buffers_to(output)?;
    Ok(())
}
