use crate::utils::replace_placeholder_values;
use ansi_term::Color::{Green, Red, Yellow};
use anyhow::{anyhow, Context, Result};
use chrono::DateTime;
use chrono_tz::Tz;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client,
};
use serde_json::{json, to_string_pretty, Value};

/// Holds all the state and methods needed to interact with a third-party email
/// provider API for scheduling tournament reminder emails.
pub struct MailingListService {
    /// a `reqwest` HTTP client instance configured for the email provider API
    client: Client,

    /// must be provided in each individual request for Kit API V3
    api_secret: String,

    /// cached result of `list_broadcasts` API call to avoid redundant requests
    broadcasts: Vec<Value>,
}

impl MailingListService {
    /// Read API token from env and initialize the HTTP client
    pub fn new() -> Result<Self> {
        let api_secret = get_kit_api_secret().context("missing API secret")?;
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));

        // Note: Kit v4 API will use standard Auth/bearer token,
        // but v3 uses a custom query/body param for api secret (??)

        // headers.insert(
        //     header::AUTHORIZATION,
        //     HeaderValue::from_str(&format!("Bearer {}", api_secret))?,
        // );

        let client = Client::builder().default_headers(headers).build()?;
        let broadcasts = Vec::new();
        Ok(Self {
            client,
            api_secret,
            broadcasts,
        })
    }

    /// Main entrypoint: Schedules both a tournament reminder email and a Top 8 email for the given tournament.
    pub async fn schedule_tournament_emails(&mut self, tournament_data: &Value) -> Result<()> {
        // Check for existing broadcast
        let tournament_name = tournament_data["name"]
            .as_str()
            .context("Missing tournament name")?;
        let subject = format!("Tournament reminder: {}", tournament_name);
        let existing_broadcast = self.get_broadcast_by_subject(&subject).await;
        if existing_broadcast.is_some() {
            println!("{}", Green.paint("- Broadcast already scheduled"));
            return Ok(());
        }

        // Create broadcast
        self.create_broadcast_v3(&subject, &tournament_data)
            .await
            .inspect_err(|e| {
                println!("{}", Red.paint("- Failed to create broadcast"));
                println!("{:?}", e);
            })?;
        println!("{}", Green.paint("- Created broadcast"));
        // let broadcast_id = res_create["broadcast"]["id"].to_string();

        // todo: update broadcast if needed
        // todo: repeat all of the above for Top 8

        Ok(())
    }

    /// - V3: https://developers.kit.com/v3#list-broadcasts
    /// - V4: https://developers.kit.com/v4?shell#list-broadcasts
    async fn list_broadcasts(&mut self) -> Result<&Vec<Value>> {
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
                .context("missing broadcasts field")?
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
            .find(|campaign| campaign["subject"].as_str().unwrap_or("") == subject)
    }

    /// https://developers.kit.com/v3#create-a-broadcast
    async fn create_broadcast_v3(&self, subject: &str, tournament_data: &Value) -> Result<Value> {
        // Determine send time (5 days before tournament start)
        let unix_start_time = tournament_data["start-unix-timestamp"]
            .as_i64()
            .context("Missing start time")?;
        let timezone: Tz = tournament_data["timezone"]
            .as_str()
            .context("Missing timezone")?
            .parse()?;
        let start_time = DateTime::from_timestamp(unix_start_time, 0)
            .context(format!("invalid start time: {}", unix_start_time))?
            .with_timezone(&timezone);
        let send_time = start_time - chrono::Duration::days(1);
        let mut send_time_iso8601 = Some(send_time.to_rfc3339());
        let now = chrono::Utc::now();
        if send_time < now {
            send_time_iso8601 = None;
            println!("{}", Yellow.paint("Warning: Already past send time"));
            println!(
                "{}",
                Yellow.paint("This broadcast will be created as a draft")
            );
            println!(
                "{}",
                Yellow.paint("You can send it manually from the web interface")
            );
        }

        // Construct content
        let content_template = r#"
            <b>{{name}}</b> is this weekend, {{date}}.
            <br>feat. {{player0}}, {{player1}}, {{player2}}, {{player3}}, {{player4}}, {{player5}}, {{player6}}, {{player7}}, and more.<br>
            <br><a href="{{start.gg-url}}" target="blank">View bracket on Start.gg</a>
        "#;
        let content = replace_placeholder_values(tournament_data, content_template);

        // Send API request
        let req = self
            .client
            .post("https://api.convertkit.com/v3/broadcasts")
            .json(&json!({
                "api_secret": &self.api_secret,
                "email_layout_template": Value::Null, // use default template
                "content": &content,
                "subject": &subject,
                "send_at": &send_time_iso8601,
                "public": true, // false == draft
            }));
        let res = req.send().await?;
        let status = res.status();
        let json = res.json::<Value>().await?;
        if status.is_success() {
            Ok(json)
        } else {
            let response_code_str = format!("Response code {}", status.as_str());
            eprintln!("{}", Red.paint(&response_code_str));
            eprintln!("{}", Red.paint(to_string_pretty(&json)?));
            Err(anyhow!(response_code_str).context(json))
        }
    }

    /// https://developers.kit.com/v4?shell#create-a-broadcast
    async fn _create_broadcast_v4(&self, subject: &str, tournament_data: &Value) -> Result<Value> {
        let content_template = r#"
            <b>{{name}}</b> is this weekend, {{date}}.<br>
            feat. {{player0}}, {{player1}}, {{player2}}, {{player3}}, {{player4}}, {{player5}}, {{player6}}, {{player7}}, and more.<br>
            <a href="{{start.gg-url}}" target="blank">View bracket on Start.gg</a><br>
        "#;
        let content = replace_placeholder_values(tournament_data, content_template);
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
            eprintln!("{}", Red.paint(to_string_pretty(&json)?));
            Err(anyhow!(response_code_str).context(json))
        }
    }
}

fn get_kit_api_secret() -> Option<String> {
    let env_key = "KIT_V3_API_SECRET";
    if let Ok(api_token) = std::env::var(env_key) {
        Some(api_token)
    } else {
        let api_url = "https://app.kit.com/account_settings/developer_settings";
        eprintln!("{}", Red.paint("Missing API secret for Kit"));
        println!("{}", Yellow.paint("Generate one here:"));
        println!("{}", Yellow.paint(api_url));
        println!("{}", Yellow.paint("Then add it to .env or run.sh"));
        println!("{}", Yellow.paint(format!("{}=your-api-secret", env_key)));
        None
    }
}
