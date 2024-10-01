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
    campaigns: Vec<Value>,
}

impl MailingListService {
    pub fn new() -> Self {
        let api_token = get_sender_api_token();
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_token)).unwrap(),
        );
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
        let client = Client::builder().default_headers(headers).build().unwrap();
        let campaigns = Vec::new();
        Self { client, campaigns }
    }

    /// https://api.sender.net/campaigns/get-all/
    pub async fn get_all_campaigns(&mut self) -> reqwest::Result<&Vec<Value>> {
        if self.campaigns.is_empty() {
            let response = self
                .client
                .get("https://api.sender.net/v2/campaigns")
                .query(&[("limit", "1000"), ("status", "scheduled")])
                .send()
                .await?
                .error_for_status()?;
            let json = response.json::<Value>().await?["data"]
                .as_array()
                .unwrap()
                .to_vec();
            self.campaigns = json;
        }
        reqwest::Result::Ok(&self.campaigns)
    }

    async fn get_existing_campaign(&mut self, title: &str) -> Option<&Value> {
        self.get_all_campaigns()
            .await
            .ok()?
            .iter()
            .find(|campaign| campaign["title"].as_str().unwrap() == title)
    }

    pub async fn schedule_tournament_emails(&mut self, tournament_data: &Value) {
        // Check for existing campaign
        let tournament_name = tournament_data["name"].as_str().unwrap();
        let campaign_name = get_name(tournament_name);
        let existing_campaign = self.get_existing_campaign(&campaign_name).await;
        if existing_campaign.is_some() {
            println!("{}", Green.paint("- Campaign already scheduled"));
            return;
        }

        // Create campaign
        let campaign_id: String;
        let res_create = self.create_campaign(&campaign_name, &tournament_data).await;
        match res_create {
            Ok(json) => {
                println!("{}", Green.paint("- Created campaign"));
                campaign_id = json["data"]["id"].as_str().unwrap().to_string();
            }
            Err(err) => {
                println!("{}", Red.paint("- Failed to create campaign"));
                println!("{:?}", err);
                return;
            }
        }

        sleep(Duration::from_millis(1000)).await;

        // Schedule campaign
        let res_schedule = self.schedule_campaign(&campaign_id, tournament_data).await;
        match res_schedule {
            Ok(_) => println!("{}", Green.paint("- Scheduled campaign")),
            Err(_) => {
                println!("{}", Red.paint("- Failed to schedule campaign"));
                let errors = self.get_campaign_errors(&campaign_id).await.unwrap();
                println!("{}", Red.paint(to_string_pretty(&errors).unwrap()));
            }
        }
    }

    /// https://api.sender.net/campaigns/create-campaign/
    async fn create_campaign(&self, name: &str, tournament_data: &Value) -> Result<Value> {
        let content_template = r#"
            <b>{{name}}</b> is this weekend, {{date}}.<br>
            feat. {{player0}}, {{player1}}, {{player2}}, {{player3}}, {{player4}}, {{player5}}, {{player6}}, {{player7}}, and more.<br>
            <a href="{{start.gg-url}}" target="blank">View bracket on Start.gg</a><br>
        "#;
        let content = replace_placeholder_values(tournament_data, content_template);
        let html = read_file("html/email.html")
            .replace("{{name}}", &name)
            .replace("{{content}}", &content);
        let response = self
            .client
            .post("https://api.sender.net/v2/campaigns")
            .json(&json!({
                "title": name,
                "preheader": "This weekend...", // todo
                "subject": name,
                "from": FROM_NAME,
                "reply_to": FROM_EMAIL,
                "content_type": "html",
                "groups": [DEV_GROUP_ID],
                "content": html,
            }))
            .send()
            .await?
            .error_for_status()?;
        let json = response.json::<Value>().await?;
        Ok(json)
    }

    /// https://api.sender.net/campaigns/schedule-send/
    async fn schedule_campaign(&self, campaign_id: &str, tournament_data: &Value) -> Result<Value> {
        let url = format!("https://api.sender.net/v2/campaigns/{campaign_id}/schedule");
        // schedule time is always in Sender.net account timezone:
        // GMT-06:00 Central Time
        // Y-m-d H:i:s
        let request = self.client.post(&url).json(&json!({
            "schedule_time": "2024-10-02 05:40:00"
        }));
        let response = request.send().await?;
        let status = response.status();
        let json = response.json::<Value>().await?;
        if status.is_success() {
            Ok(json)
        } else {
            eprintln!(
                "{}",
                Red.paint(format!("Response code {}", status.as_str()))
            );
            eprintln!("{}", Red.paint(to_string_pretty(&json).unwrap()));
            Err(anyhow!("Response code: {}", status).context(json))
        }
    }

    /// https://api.sender.net/campaigns/errors/
    async fn get_campaign_errors(&self, campaign_id: &str) -> Result<Value> {
        let url = format!("https://api.sender.net/v2/campaigns/{campaign_id}/errors");
        let request = self.client.get(&url);
        let response = request.send().await?;
        let status = response.status();
        let json = response.json::<Value>().await?;
        if status.is_success() {
            Ok(json)
        } else {
            eprintln!(
                "{}",
                Red.paint(format!("Response code {}", status.as_str()))
            );
            eprintln!("{}", Red.paint(to_string_pretty(&json).unwrap()));
            Err(anyhow!("Response code: {}", status).context(json))
        }
    }
}

fn get_sender_api_token() -> String {
    if let Ok(api_token) = std::env::var("SENDER_API_TOKEN") {
        return api_token;
    }

    let api_url = "https://app.sender.net/settings/tokens";
    eprintln!("{}", Red.paint("Missing API token for Sender"));
    println!("{}", Yellow.paint("Generate one here:"));
    println!("{}", Yellow.paint(api_url));
    println!("{}", Yellow.paint("Then add it to .env or run.sh"));
    panic!("SENDER_API_TOKEN must be set");
}

fn get_name(tournament_name: &str) -> String {
    format!("Tournament reminder: {}", tournament_name)
}

fn get_name_top8(tournament_name: &str) -> String {
    format!("Top 8 starting soon: {}", tournament_name)
}
