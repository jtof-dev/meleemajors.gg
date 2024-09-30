extern crate dotenv;
use dotenv::dotenv;
use mailerlite_rs::{parameter::Parameter, response::Response, MailerLite};

fn get_mailerlite_api_token() -> String {
    // Read vars from .env file if present
    dotenv().ok();

    if let Ok(api_token) = std::env::var("MAILERLITE_API_TOKEN") {
        api_token
    } else {
        // todo: color error messages
        eprintln!("⚠️  Missing API token for MailerLite");
        println!("👉 Generate one here: https://dashboard.mailerlite.com/integrations/api");
        println!("👉 Then add it to run.sh or your environment variables");
        panic!("MAILERLITE_API_TOKEN must be set");
    }
}

/// https://developers.mailerlite.com/docs/subscribers.html#list-all-subscribers
pub async fn list_subscribers(mailerlite: &MailerLite) -> Response {
    mailerlite
        .subscriber()
        .get(Parameter::new().add("limit", "1000"))
        .await
}

pub async fn main() {
    let api_token = get_mailerlite_api_token();
    let client = MailerLite::new(api_token);
    // todo: read list of tournaments (either from disk or after creation in main.rs)
    // todo: list all scheduled campaigns (week before & top 8)
    // todo: find any tournaments that are missing a campaign
    // todo: schedule campaigns and show preview links
}
