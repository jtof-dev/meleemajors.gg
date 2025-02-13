use crate::utils::{log_red, log_success, log_warn, log_yellow, replace_placeholder_values};
use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, NaiveDateTime};
use chrono_tz::Tz;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client, Method,
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

        // Note: Kit V4 API will use standard Auth/bearer token,
        // but V3 uses a custom query/body param for api secret (??)

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

    /// Schedule a reminder email for 5 days before the tournament starts, if needed
    pub async fn schedule_reminder_broadcast(&mut self, tournament_data: &Value) -> Result<()> {
        // Determine send time (5 days before tournament start)
        let unix_start_time = tournament_data["start-unix-timestamp"]
            .as_i64()
            .context("missing start time")?;
        let timezone: Tz = tournament_data["timezone"]
            .as_str()
            .context("missing timezone")?
            .parse()?;
        let start_time = DateTime::from_timestamp(unix_start_time, 0)
            .context(format!("invalid start time: {}", unix_start_time))?
            .with_timezone(&timezone);
        let send_time = start_time - chrono::Duration::days(5);

        // Email subject
        let tournament_name = tournament_data["name"]
            .as_str()
            .context("missing tournament name")?;
        let subject = format!("Tournament reminder: {}", tournament_name);

        // Player names
        let mut player_names: Vec<&str> = Vec::new();
        for i in 0..8 {
            let player_key = format!("player{}", i);
            if let Some(player_name) = tournament_data[player_key].as_str() {
                if !player_name.eq("TBD") {
                    player_names.push(player_name);
                }
            }
        }
        let mut player_names_str = "".to_string();
        if !player_names.is_empty() {
            player_names_str = format!("Featuring {}, and more.", player_names.join(", "));
        }

        // Generate content
        let content_template = r#"
            <h1 style="text-align: center; width: 100%; margin-bottom: 24px">{{name}}</h1>
            <div>This weekend, {{date}}.</div>
            {{player_names_str}}
            <div style="display: flex; justify-content: center; align-items: center; margin: 24px 0"><a href="{{start.gg-url}}" target="blank">View bracket on Start.gg</a></div>
        "#.replace("{{player_names_str}}", &player_names_str);

        let content = replace_placeholder_values(tournament_data, &content_template);

        // Check for existing broadcast
        let existing_broadcast = self
            .get_broadcast_by_subject(&subject)
            .await
            .map(Clone::clone);
        if let Some(existing) = existing_broadcast {
            // Update broadcast
            let broadcast_id = existing["id"].to_string();
            self.update_broadcast(&broadcast_id, &send_time, &subject, &content)
                .await?;
            log_success("email", "reminder broadcast updated");
        } else {
            // Create broadcast
            self.create_broadcast(&send_time, &subject, &content)
                .await?;
            log_success("email", "reminder broadcast created");
        }
        Ok(())
    }

    /// Schedule a reminder email for the start of Top 8, if needed
    pub async fn schedule_top8_broadcast(&mut self, tournament_data: &Value) -> Result<()> {
        // Parse top 8 start time
        let top8_start_time_str = tournament_data["top8-start-time"].as_str().unwrap_or("");
        if top8_start_time_str.is_empty() {
            log_warn("email", "missing top 8 start time");
            return Ok(());
        }
        let top8_datetime_format = "%Y-%m-%d %I:%M%P"; // e.g. "2024-10-06 3:00PM"
        let timezone: Tz = tournament_data["timezone"]
            .as_str()
            .context("missing timezone")?
            .parse()?;
        let top8_start_time =
            NaiveDateTime::parse_from_str(top8_start_time_str, top8_datetime_format)?
                .and_local_timezone(timezone)
                .single()
                .context("invalid top8-start-time")?;

        // Email subject
        let tournament_name = tournament_data["name"]
            .as_str()
            .context("missing tournament name")?;
        let subject = format!("Top 8 starting now: {}", tournament_name);

        // Generate content
        let mut content_template = r#"
            <h1 style="text-align: center; width: 100%; margin-bottom: 24px">{{name}}</h1>
            <div style="display: flex; justify-content: center; align-items: center; text-align: center">Live now! Top 8 starts soon.</div>
            <div style="display: flex; justify-content: center; align-items: center; margin: 24px 0"><a href="{{start.gg-url}}" target="blank">View bracket on Start.gg</a></div>
        "#.to_string();

        let has_stream = !tournament_data["stream-url"]
            .as_str()
            .unwrap_or("")
            .is_empty();
        if has_stream {
            content_template.push_str(r#"<div style="display: flex; justify-content: center; align-items: center; margin: 24px 0"><a href="{{stream-url}}" target="blank">Stream</a></div>"#);
        }
        let content = replace_placeholder_values(tournament_data, &content_template);

        // Check for existing broadcast
        let existing_broadcast = self
            .get_broadcast_by_subject(&subject)
            .await
            .map(Clone::clone);
        if let Some(existing) = existing_broadcast {
            // Update broadcast
            let broadcast_id = existing["id"].to_string();
            self.update_broadcast(&broadcast_id, &top8_start_time, &subject, &content)
                .await?;
            log_success("email", "top 8 broadcast updated");
        } else {
            // Create broadcast
            self.create_broadcast(&top8_start_time, &subject, &content)
                .await?;
            log_success("email", "top 8 broadcast created");
        }
        Ok(())
    }

    /// https://developers.kit.com/v3#list-broadcasts
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

    /// - https://developers.kit.com/v3#create-a-broadcast
    async fn create_broadcast(
        &self,
        send_time: &DateTime<Tz>,
        subject: &str,
        content: &str,
    ) -> Result<Value> {
        let url = "https://api.convertkit.com/v3/broadcasts";
        self.create_or_update_broadcast(Method::POST, url, send_time, subject, content)
            .await
    }

    /// - https://developers.kit.com/v3#update-a-broadcast
    async fn update_broadcast(
        &self,
        broadcast_id: &str,
        send_time: &DateTime<Tz>,
        subject: &str,
        content: &str,
    ) -> Result<Value> {
        let url = &format!("https://api.convertkit.com/v3/broadcasts/{}", broadcast_id);
        self.create_or_update_broadcast(Method::PUT, url, send_time, subject, content)
            .await
    }

    /// Handles the common logic & parameters for creating or updating a broadcast
    async fn create_or_update_broadcast(
        &self,
        method: Method,
        url: &str,
        send_time: &DateTime<Tz>,
        subject: &str,
        content: &str,
    ) -> Result<Value> {
        // Validate send time
        let send_time_iso8601 = Some(send_time.to_rfc3339());
        let now = chrono::Utc::now();
        if send_time < &now {
            bail!("Already past send time");
        }

        // Send API request
        let req = self.client.request(method, url).json(&json!({
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
            log_red(&response_code_str);
            log_red(&to_string_pretty(&json)?);
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
        log_red("Missing API secret for Kit");
        log_yellow("Generate one here:");
        log_yellow(api_url);
        log_yellow("Then add it to .env or run.sh");
        log_yellow(&format!("{}=your-api-secret", env_key));
        None
    }
}
