mod core;

fn main() {
    use core::server;
    use core::config::Manifest;

    let manifest: Manifest;
    // assert: manifest must exist in path or die
    match Manifest::try_from("./config/manifest.toml") {
        Ok(v) => manifest = v,
        Err(e) => panic!("{e}"),
    }

    if let Err(e) = server::run(manifest) {
        panic!("{e}");
    }
}

