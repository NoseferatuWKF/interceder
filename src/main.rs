use std::fs;
use std::io::prelude::*;
use tide::{Request, Result, Response, StatusCode};

const PAYLOAD_PATH: &str = "./payload";
const HOST: &str = "0.0.0.0:42069";
const WEBHOOK: &str = "https://local.unleashedsoftware.com/api/api/webhooks/incoming/shopify";

#[tokio::main]
async fn main() -> Result<()> {
    use tide::security::{CorsMiddleware, Origin};
    use http_types::headers::HeaderValue;

    init();

    let mut app = tide::new();

    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);
    app.with(cors);

    app.at("/redirect").post(redirect);
    app.at("/replay").get(replay);

    println!("Server is listening on {}", HOST);
    app.listen(HOST).await?;

    Ok(())
}

fn init() {
    fs::create_dir_all(PAYLOAD_PATH)
        .expect("cannot create payload directory");

    let envs = vec![
        (std::env::var("AWSSecret"),"AWSSecret"),
        (std::env::var("OrgId"), "OrgId"),
        (std::env::var("AuthSignature"), "AuthSignature"),
    ];

    for (env, name) in envs {
        if let Err(e) = env {
            panic!("{}: {}", e, name);
        }
    }
}

async fn redirect(mut req: Request<()>) -> Result {
    let body = req.body_bytes().await?;

    let (shopify_shop_domain, shopify_topic) = get_required_headers(&req);

    match shopify_topic.as_str() {
        "orders/updated" => fs::write(
            format!("{}/order.json", PAYLOAD_PATH), &body)?,
        "fulfillments/create" => fs::write(
            format!("{}/fulfillment.json", PAYLOAD_PATH), &body)?,
        "fulfillments/update" => fs::write(
            format!("{}/fulfillment.json", PAYLOAD_PATH), &body)?,
        "customers/update" => fs::write(
            format!("{}/customer.json", PAYLOAD_PATH), &body)?,
        _ => unimplemented!(),
    };

    let hash = rehash(body.clone());

    send_request(
        hash,
        shopify_shop_domain,
        shopify_topic,
        body)
    .await;

    Ok(Response::new(StatusCode::Ok))
}

async fn replay(req: Request<()>) -> Result { 
    let (shopify_shop_domain, shopify_topic) = get_required_headers(&req);

    let mut input = match shopify_topic.as_str() {
        "orders/updated" => fs::File::open(
            format!("{}/order.json", PAYLOAD_PATH))?,
        "fulfillments/create" => fs::File::open(
            format!("{}/fulfillment.json", PAYLOAD_PATH))?,
        "fulfillments/update" => fs::File::open(
            format!("{}/fulfillment.json", PAYLOAD_PATH))?,
        "customers/update" => fs::File::open(
            format!("{}/customer.json", PAYLOAD_PATH))?,
        _ => unimplemented!(),
    };

    let mut buffer = Vec::new();
    input.read_to_end(&mut buffer)?;

    let hash = rehash(buffer.clone());

    send_request(
        hash,
        shopify_shop_domain,
        shopify_topic,
        buffer)
    .await;

    Ok(Response::new(StatusCode::Ok))
}

async fn send_request(hash: String, shop_domain: String, topic: String, body_buffer: Vec<u8>) {
    let org_id = std::env::var("OrgId").unwrap();
    let auth_signature = std::env::var("AuthSignature").unwrap();

    let client = reqwest::Client::new();
    let _ = client.post(format!("{}/{}", WEBHOOK, org_id))
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .header("api-auth-id", org_id)
        .header("api-auth-signature", auth_signature)
        .header("x-shopify-hmac-sha256", hash)
        .header("x-shopify-shop-domain", shop_domain)
        .header("x-shopify-topic", topic)
        .body(body_buffer)
        .send()
    .await; 
}

fn rehash(input_buffer: Vec<u8>) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use base64::prelude::*;

    let secret = std::env::var("AWSSecret").unwrap();

    let mut generator = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .expect("cannot generate hmac");

    generator.update(&input_buffer);

    let hash = generator.finalize().into_bytes();

    BASE64_STANDARD.encode(hash)
}

fn get_required_headers(req: &Request<()>) -> (String, String) {
    let shopify_shop_domain = req.header("x-shopify-shop-domain")
        .expect("missing header: x-shopify-shop-domain")
        .get(0)
        .unwrap()
        .to_string();

    let shopify_topic = req.header("x-shopify-topic")
        .expect("missing header: x-shopify-topic")
        .get(0)
        .unwrap()
        .to_string();

    (shopify_shop_domain, shopify_topic)
}
