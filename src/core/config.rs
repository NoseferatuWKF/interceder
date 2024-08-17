use serde::Deserialize;

#[derive(Deserialize)]
pub struct Manifest {
    pub server: Server,
    pub webhook: Webhook,
}

impl TryFrom<&str> for Manifest {
    type Error = Box<dyn std::error::Error>;

    fn try_from(path: &str) -> Result<Self, Self::Error> {
        use std::fs;
        use std::io::Read;

        let mut file = fs::File::open(path)?;

        let mut contents = String::new();

        file.read_to_string(&mut contents)?;
        let config: Manifest = toml::from_str(&contents)?;

        Ok(config)
    }
}

#[derive(Deserialize)]
pub struct Server {
    pub address: String,
    pub port: String,
    pub env: Vec<String>,
}

#[derive(Deserialize)]
pub struct Webhook {
    pub url: String,
    pub params: Vec<String>,
    pub topics: Vec<String>,
    pub headers: Vec<Vec<String>>,
    pub hash: Hash,
    pub rehash: Rehash,
}

#[derive(Deserialize)]
pub struct Hash {
    pub is_required: bool,
    pub header: String,
}

#[derive(Deserialize)]
pub struct Rehash {
    pub is_required: bool,
    pub secret: String,
}
