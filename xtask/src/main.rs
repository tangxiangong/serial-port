use std::path::Path;
fn run() -> xtask::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["release-notes", source, output] => xtask::notes::generate(Path::new(source), Path::new(output)),
        ["sign-windows", directory, tag] => xtask::windows::sign(Path::new(directory), tag, &std::env::var("WINDOWS_UPDATE_PUBLIC_KEY")?),
        ["feeds", directory, output, tag] => xtask::feeds(Path::new(directory), Path::new(output), tag, "tangxiangong/serial-port"),
        _ => Err("usage: cargo run -p xtask -- release-notes SOURCE OUTPUT | sign-windows DIRECTORY vVERSION | feeds DIRECTORY OUTPUT vVERSION".into()),
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
