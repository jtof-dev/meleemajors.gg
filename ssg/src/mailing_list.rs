extern crate dotenv;
use dotenv::dotenv;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, Error,
};
use serde_json::Value;

fn get_mailerlite_api_token() -> String {
    // Read vars from .env file if present
    dotenv().ok();

    if let Ok(api_token) = std::env::var("MAILERLITE_API_TOKEN") {
        api_token
    } else {
        eprintln!("⚠️  Missing API token for MailerLite");
        println!("👉 Generate one here: https://dashboard.mailerlite.com/integrations/api");
        println!("👉 Then add it to run.sh or your environment variables");
        panic!("MAILERLITE_API_TOKEN must be set");
    }
}

fn create_reqwest_client(api_token: &str) -> Result<Client, Error> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", api_token)).unwrap(),
    );
    Client::builder().default_headers(headers).build()
}

/// https://developers.mailerlite.com/docs/subscribers.html#list-all-subscribers
pub async fn list_subscribers(client: &Client) -> Result<Value, Error> {
    client
        .get("https://connect.mailerlite.com/api/subscribers")
        .send()
        .await?
        .json::<Value>()
        .await
}

/// https://developers.mailerlite.com/docs/campaigns.html#campaign-list
pub async fn list_campaigns(client: &Client) -> Result<Value, Error> {
    client
        .get("https://connect.mailerlite.com/api/campaigns")
        .send()
        .await?
        .json::<Value>()
        .await
}

pub async fn main() {
    let api_token = get_mailerlite_api_token();
    let client = create_reqwest_client(&api_token).unwrap();
    // todo: read list of tournaments (either from disk or after creation in main.rs)
    // todo: list all scheduled campaigns (week before & top 8)
    // todo: find any tournaments that are missing a campaign
    // todo: schedule campaigns and show preview links
}
