use ansi_term::Color::{Green, Red, Yellow};
use mailerlite_rs::{data::Data, parameter::Parameter, response::Response, MailerLite};
use serde_json::{Number, Value};

use crate::utils::{read_file, replace_placeholder_values};

const FROM_NAME: &'static str = "meleemajors.gg";
const FROM_EMAIL: &'static str = "hello@meleemajors.gg";
const DEV_GROUP_ID: &'static str = "133853657487639868";

pub struct MailingListService {
    client: MailerLite,
    scheduled_emails: Vec<Value>,
}

impl MailingListService {
    pub fn new() -> Self {
        let api_token = get_mailerlite_api_token();
        let client = MailerLite::new(api_token);
        let scheduled_emails = Vec::new();
        Self {
            client,
            scheduled_emails,
        }
    }

    pub async fn list_scheduled_emails(&mut self) -> &Vec<Value> {
        if self.scheduled_emails.is_empty() {
            let params = Parameter::new()
                .add("filter[status]", "draft") // todo: should change this to "ready"
                .add("limit", "100");
            let response = self.client.campaign().get(params).await;
            self.scheduled_emails = response.content["data"].as_array().unwrap().clone();
        }
        &self.scheduled_emails
    }

    pub async fn schedule_tournament_emails(&mut self, tournament_data: &Value) {
        let reminder_campaign_name =
            campaign_name_tournament_reminder(&tournament_data["name"].as_str().unwrap());

        // Check for existing campaign
        let exisiting_campaign = self
            .list_scheduled_emails()
            .await
            .iter()
            .find(|campaign| campaign["name"].as_str().unwrap() == reminder_campaign_name);

        // Create campaign if needed
        let campaign_id: String;
        if let Some(campaign) = exisiting_campaign {
            campaign_id = campaign["id"].as_str().unwrap().to_string();
        } else {
            let res_create = self.create_campaign(&reminder_campaign_name).await;
            if !res_create.status_code.is_success() {
                println!("{}", Red.paint("- Failed to create campaign"));
                println!("{:?}", res_create);
                return;
            } else {
                println!("{}", Green.paint("- Created campaign"));
                campaign_id = res_create.content["data"]["id"]
                    .as_str()
                    .unwrap()
                    .to_string();
            }
        }

        // Update email contents
        let res_update = self
            .update_tournament_reminder(&campaign_id, tournament_data)
            .await;
        if !res_update.status_code.is_success() {
            println!("{}", Red.paint("- Failed to update campaign"));
            println!("{:?}", res_update);
            return;
        } else {
            println!("{}", Green.paint("- Updated campaign"));
        }

        // todo: schedule email

        // // Top 8 alert (1 hour before)
        // let top_8_campaign_name = campaign_name_top_8(&tournament_data["name"].as_str().unwrap());
        // let top_8_campaign = self
        //     .list_scheduled_emails()
        //     .await
        //     .iter()
        //     .find(|campaign| campaign["name"].as_str().unwrap() == top_8_campaign_name);
        // if let Some(campaign) = top_8_campaign {
        //     println!("{}", Green.paint("- Tournament reminder already scheduled"));
        //     // todo: update email
        // } else {
        //     println!("{}", Yellow.paint("- Missing reminder"));
        //     // todo: schedule email
        // }
    }

    pub async fn create_campaign(&self, name: &str) -> Response {
        let data = Data::new()
            .add("name", &name)
            .add("type", "regular")
            .add("groups[0]", DEV_GROUP_ID)
            .add("emails[0][subject]", &name)
            .add("emails[0][from_name]", FROM_NAME)
            .add("emails[0][from]", FROM_EMAIL);
        let response = self.client.campaign().create(data).await;
        response
    }

    pub async fn update_tournament_reminder(
        &mut self,
        email_id: &str,
        tournament_data: &Value,
    ) -> Response {
        let name = campaign_name_tournament_reminder(&tournament_data["name"].as_str().unwrap());
        let content_template = r#"
            <b>{{name}}</b> is this weekend, {{date}}.<br>
            feat. {{player0}}, {{player1}}, {{player2}}, {{player3}}, {{player4}}, {{player5}}, {{player6}}, {{player7}}, and more.<br>
            <a href="{{start.gg-url}}" target="blank">View bracket on Start.gg</a><br>
        "#;
        let content = replace_placeholder_values(tournament_data, content_template);
        let html = read_file("html/email.html")
            .replace("{{name}}", &name)
            .replace("{{content}}", &content);
        let data = Data::new()
            .add("name", &name)
            .add("groups[0]", DEV_GROUP_ID)
            .add("emails[0][subject]", &name)
            .add("emails[0][from_name]", FROM_NAME)
            .add("emails[0][from]", FROM_EMAIL)
            .add("emails[0][content]", &html);
        let response = self
            .client
            .campaign()
            .update(email_id.to_string(), data)
            .await;
        response
    }
}

fn get_mailerlite_api_token() -> String {
    if let Ok(api_token) = std::env::var("MAILERLITE_API_TOKEN") {
        return api_token;
    }

    let api_url = "https://dashboard.mailerlite.com/integrations/api";
    eprintln!("{}", Red.paint("Missing API token for MailerLite"));
    println!("{}", Yellow.paint("Generate one here:"));
    println!("{}", Yellow.paint(api_url));
    println!("{}", Yellow.paint("Then add it to .env or run.sh"));
    panic!("MAILERLITE_API_TOKEN must be set");
}

fn campaign_name_tournament_reminder(tournament_name: &str) -> String {
    format!("Tournament reminder: {}", tournament_name)
}

fn campaign_name_top_8(tournament_name: &str) -> String {
    format!("Top 8 starting soon: {}", tournament_name)
}
