use std::fs;
use std::io::prelude::*;
use std::sync::OnceLock;
use tide::{Request, Result, Response, StatusCode};

use crate::core::config::Config;

const PAYLOAD_PATH: &str = "./payload";

static CONFIG: OnceLock<Config> = OnceLock::new();
static WEBHOOK: OnceLock<String> = OnceLock::new();

#[tokio::main]
pub async fn run(config: Config) -> Result<()> {
    use tide::security::{CorsMiddleware, Origin};
    use http_types::headers::HeaderValue;

    let server = String::from(&config.server.address) + ":" + &config.server.port;

    // transfer config ownership to init
    if let Err(e) = init(config) {
        panic!("{e}");
    }

    let mut app = tide::new();

    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);
    app.with(cors);

    // setup endpoints
    app.at("/redirect").post(redirect);
    app.at("/replay").get(replay);

    println!("Interceder is listening on {}", server);
    app.listen(server).await?;

    Ok(())
}

fn init(config: Config) -> Result<()> {
    fs::create_dir_all(PAYLOAD_PATH)?;

    // set global variable and drop config
    if let Ok(_) = CONFIG.set(config) {
        let config = CONFIG.get().unwrap();
        
        // assert: env must exist or die
        config.server.env.iter().for_each(|v| {
            if let Err(e) = std::env::var(&v) {
                panic!("{}: {}", e, v);
            }
        });

        let url = config.webhook.params.iter()
            .fold(
                String::from(&config.webhook.url),
                // env should already exists here after init
                |acc, i| acc + "/" + &std::env::var(i).unwrap());
        WEBHOOK.set(url).unwrap();
    }

    Ok(())
}

async fn redirect(mut req: Request<()>) -> Result {
    let body = req.body_bytes().await?;

    let headers = get_required_headers_values(&req);

    // find matching topic from config
    let topic = CONFIG.get().unwrap().webhook.topics.iter()
        .find(|x| headers.iter().find(|y| y.eq(x)).is_some());

    // cache request body
    if let Some(t) = topic {
        fs::write(
            format!(
                "{}/{}.json",
                PAYLOAD_PATH,
                t.split('/').collect::<Vec<_>>().get(0).unwrap()),
            &body)?;
    }

    send_request(headers, body).await;

    Ok(Response::new(StatusCode::Ok))
}

async fn replay(req: Request<()>) -> Result { 
    let headers = get_required_headers_values(&req);

    // find matching topic from config
    let topic = CONFIG.get().unwrap().webhook.topics.iter()
        .find(|x| headers.iter().find(|y| y.eq(x)).is_some());

    // read request body from cache
    let mut buffer = Vec::new();
    if let Some(t) = topic {
        let mut input = fs::File::open(
            format!(
                "{}/{}.json",
                PAYLOAD_PATH,
                t.split('/').collect::<Vec<_>>().get(0).unwrap()))?;
        input.read_to_end(&mut buffer)?;
    }

    send_request(headers, buffer).await;

    Ok(Response::new(StatusCode::Ok))
}

async fn send_request(headers: Vec<String>, body_buffer: Vec<u8>) {
    // take headers ownership to mutate the elements
    let mut iter = headers.into_iter();

    // all the unwraps here should be fine as it was asserted in init
    let mut rebuilt_headers = CONFIG.get().unwrap().webhook.headers.iter()
        // check if header needs value from env or not
        .map(|v| match v.get(1).is_some_and(|x| x.eq("req")) {
            true => {
                let header = v.get(0).unwrap();
                // removes the first index in the headers
                // this should already be aligned with the headers
                let value = iter.next().unwrap();
                vec![header.clone(), value.clone()]
            },
            false => {
                let header = v.get(0).unwrap();
                let value = std::env::var(v.get(1).unwrap()).unwrap();
                vec![header.clone(), value.clone()]
            },
        })
        .collect::<Vec<Vec<_>>>();

    // optional: check if webhook requires hash
    let hash = &CONFIG.get().unwrap().webhook.hash;
    if hash.is_required {
        let header = hash.header.clone();
        let mut value = iter.next().unwrap();

        // optional: check if request requires rehash from another secret
        if CONFIG.get().unwrap().webhook.rehash.is_required {
            value = rehash(body_buffer.clone());
        }

        rebuilt_headers.push(vec![header, value]);
    }

    // create RequestBuilder with basic headers
    let mut builder = reqwest::Client::new().post(WEBHOOK.get().unwrap())
        .header("accept", "application/json")
        .header("content-type", "application/json");

    // append all of the rebuilt headers
    for header in rebuilt_headers {
        builder = builder.header(
            header.get(0).unwrap(), header.get(1).unwrap());
    }

    // add request body
    builder = builder.body(body_buffer);

    // send request
    if let Err(e) = builder.send().await {
        panic!("{e}");
    }
}

fn get_required_headers_values(req: &Request<()>) -> Vec<String> {
    // get base headers value
    let mut headers = CONFIG.get().unwrap().webhook.headers.iter()
        .filter_map(|v| match v.get(1).is_some_and(|x| x.eq("req")) {
            true => v.get(0),
            false => None,
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|v| req.header(v.as_str())
            .expect(format!("missing header: {}", v).as_str())
            .get(0)
            .unwrap()
            .to_string())
        .collect::<Vec<String>>();

    // optional: add hash value to required headers
    let hash = &CONFIG.get().unwrap().webhook.hash;
    if hash.is_required {
        headers.push(req.header(hash.header.as_str())
            .expect(format!("missing header: {}", hash.header).as_str())
            .get(0)
            .unwrap()
            .to_string())
    }

    headers
}

// NOTE: currently only supports HMAC SHA256
fn rehash(input_buffer: Vec<u8>) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use base64::prelude::*;

    let secret = std::env::var(
        &CONFIG.get().unwrap().webhook.rehash.secret).unwrap();

    let mut generator = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .expect("cannot generate hmac");

    generator.update(&input_buffer);

    let hash = generator.finalize().into_bytes();

    BASE64_STANDARD.encode(hash)
}
