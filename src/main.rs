mod core;

fn main() {
    use core::server;
    use core::config::Config;

    let config: Config;
    // assert: config must exist in path or die
    match Config::try_from("interceder.toml") {
        Ok(v) => config = v,
        Err(e) => panic!("{e}"),
    }

    if let Err(e) = server::run(config) {
        panic!("{e}");
    }
}

