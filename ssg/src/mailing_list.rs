use std::time::Duration;

use ansi_term::Color::{Green, Red, Yellow};
use anyhow::{anyhow, Result};
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client,
};
use serde_json::{json, to_string_pretty, Value};
use tokio::time::sleep;

use crate::utils::{read_file, replace_placeholder_values};

const FROM_NAME: &'static str = "meleemajors.gg";
const FROM_EMAIL: &'static str = "hello@meleemajors.gg";
const DEV_GROUP_ID: &'static str = "e3Ow7r";

pub struct MailingListService {
    client: Client,
    api_secret: String,
    broadcasts: Vec<Value>,
}

impl MailingListService {
    pub fn new() -> Self {
        let api_secret = get_kit_api_secret();
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));

        // Note: v4 API will use standard Auth/bearer token,
        // but v3 uses a custom query param for api secret (??)

        // headers.insert(
        //     header::AUTHORIZATION,
        //     HeaderValue::from_str(&format!("Bearer {}", api_secret)).unwrap(),
        // );

        let client = Client::builder().default_headers(headers).build().unwrap();
        let broadcasts = Vec::new();
        Self {
            client,
            api_secret,
            broadcasts,
        }
    }

    pub async fn schedule_tournament_emails(&mut self, tournament_data: &Value) {
        // Check for existing broadcast
        let tournament_name = tournament_data["name"].as_str().unwrap();
        let subject = get_subject_for_tournament(tournament_name);
        let existing_broadcast = self.get_broadcast_by_subject(&subject).await;
        if existing_broadcast.is_some() {
            println!("{}", Green.paint("- Broadcast already scheduled"));
            return;
        }

        // Create broadcast
        let broadcast_id: String;
        let res_create = self.create_broadcast_v3(&subject, &tournament_data).await;
        match res_create {
            Ok(json) => {
                println!("{}", Green.paint("- Created broadcast"));
                broadcast_id = json["broadcast"]["id"].as_number().unwrap().to_string();
            }
            Err(err) => {
                println!("{}", Red.paint("- Failed to create broadcast"));
                println!("{:?}", err);
                return;
            }
        }

        // todo: update broadcast if needed
    }

    /// - V3: https://developers.kit.com/v3#list-broadcasts
    /// - V4: https://developers.kit.com/v4?shell#list-broadcasts
    pub async fn list_broadcasts(&mut self) -> Result<&Vec<Value>> {
        if self.broadcasts.is_empty() {
            let response = self
                .client
                .get("https://api.convertkit.com/v3/broadcasts")
                .query(&[("api_secret", &self.api_secret)])
                .send()
                .await?
                .error_for_status()?;
            let json = response.json::<Value>().await?["broadcasts"]
                .as_array()
                .unwrap()
                .to_vec();
            self.broadcasts = json;
        }
        Result::Ok(&self.broadcasts)
    }

    async fn get_broadcast_by_subject(&mut self, subject: &str) -> Option<&Value> {
        self.list_broadcasts()
            .await
            .ok()?
            .iter()
            .find(|campaign| campaign["subject"].as_str().unwrap() == subject)
    }

    /// https://developers.kit.com/v3#list-broadcasts
    async fn create_broadcast_v3(&self, subject: &str, tournament_data: &Value) -> Result<Value> {
        let content_template = r#"
            <b>{{name}}</b> is this weekend, {{date}}.<br>
            feat. {{player0}}, {{player1}}, {{player2}}, {{player3}}, {{player4}}, {{player5}}, {{player6}}, {{player7}}, and more.<br>
            <a href="{{start.gg-url}}" target="blank">View bracket on Start.gg</a><br>
        "#;
        let content = replace_placeholder_values(tournament_data, content_template);
        let html = read_file("html/email.html")
            .replace("{{name}}", &subject)
            .replace("{{content}}", &content);
        let req = self
            .client
            .post("https://api.convertkit.com/v3/broadcasts")
            .json(&json!({
                "api_secret": &self.api_secret,
                "email_layout_template": Value::Null, // todo
                "content": content,
            }));
        let res = req.send().await?;
        let status = res.status();
        let json = res.json::<Value>().await?;
        if status.is_success() {
            Ok(json)
        } else {
            let response_code_str = format!("Response code {}", status.as_str());
            eprintln!("{}", Red.paint(&response_code_str));
            eprintln!("{}", Red.paint(to_string_pretty(&json).unwrap()));
            Err(anyhow!(response_code_str).context(json))
        }
    }

    /// https://developers.kit.com/v4?shell#create-a-broadcast
    async fn create_broadcast_v4(&self, subject: &str, tournament_data: &Value) -> Result<Value> {
        let content_template = r#"
            <b>{{name}}</b> is this weekend, {{date}}.<br>
            feat. {{player0}}, {{player1}}, {{player2}}, {{player3}}, {{player4}}, {{player5}}, {{player6}}, {{player7}}, and more.<br>
            <a href="{{start.gg-url}}" target="blank">View bracket on Start.gg</a><br>
        "#;
        let content = replace_placeholder_values(tournament_data, content_template);
        let html = read_file("html/email.html")
            .replace("{{name}}", &subject)
            .replace("{{content}}", &content);
        let req = self
            .client
            .post("https://api.kit.com/v4/broadcasts")
            .json(&json!({
                "email_template_id": Value::Null, // todo: create template
                "broadcast_id": Value::Null,
                "content": content,
                "description": Value::Null,
                "public": false, // false == draft
                "published_at": Value::Null,
                "send_at": Value::Null, // The scheduled send time for this broadcast in ISO8601 format
                "thumbnail_alt": Value::Null,
                "preview_text": "This weekend...", // todo
                "subject": subject,
                "subscriber_filter": Value::Null, // todo: dev vs all
            }));
        let res = req.send().await?;
        let status = res.status();
        let json = res.json::<Value>().await?;
        if status.is_success() {
            Ok(json)
        } else {
            let response_code_str = format!("Response code {}", status.as_str());
            eprintln!("{}", Red.paint(&response_code_str));
            eprintln!("{}", Red.paint(to_string_pretty(&json).unwrap()));
            Err(anyhow!(response_code_str).context(json))
        }
    }
}

fn get_kit_api_secret() -> String {
    let env_key = "KIT_V3_API_SECRET";
    if let Ok(api_token) = std::env::var(env_key) {
        return api_token;
    }

    let api_url = "https://app.kit.com/account_settings/developer_settings";
    eprintln!("{}", Red.paint("Missing API secret for Kit"));
    println!("{}", Yellow.paint("Generate one here:"));
    println!("{}", Yellow.paint(api_url));
    println!("{}", Yellow.paint("Then add it to .env or run.sh"));
    println!("{}", Yellow.paint(format!("{}=your-api-secret", env_key)));
    panic!("Missing API Secret env var");
}

fn get_subject_for_tournament(tournament_name: &str) -> String {
    format!("Tournament reminder: {}", tournament_name)
}

fn get_subject_for_top8(tournament_name: &str) -> String {
    format!("Top 8 starting soon: {}", tournament_name)
}
