use crate::utils::{log_error, log_skip, log_success, log_warn, replace_placeholder_values};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, NaiveDateTime};
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

        // Note: Kit V4 API will use standard Auth/bearer token,
        // but V3 uses a custom query/body param for api secret (??)

        // headers.insert(
        //     header::AUTHORIZATION,
        //     HeaderValue::from_str(&format!("Bearer {}", api_secret))?,
        // );

        let client = Client::builder().default_headers(headers).build()?;
        let broadcasts = Vec::new(); // todo: should broadcasts be fetched immediately?

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
            .context("Missing start time")?;
        let timezone: Tz = tournament_data["timezone"]
            .as_str()
            .context("Missing timezone")?
            .parse()?;
        let start_time = DateTime::from_timestamp(unix_start_time, 0)
            .context(format!("invalid start time: {}", unix_start_time))?
            .with_timezone(&timezone);
        let send_time = start_time - chrono::Duration::days(1);

        // Email subject
        let tournament_name = tournament_data["name"]
            .as_str()
            .context("Missing tournament name")?;
        let subject = format!("Tournament reminder: {}", tournament_name);

        // Generate content
        let content_template = r#"
            <h1>{{name}}</h1>
            <br>This weekend, {{date}}.
            <br>feat. {{player0}}, {{player1}}, {{player2}}, {{player3}}, {{player4}}, {{player5}}, {{player6}}, {{player7}}, and more.<br>
            <br><a href="{{start.gg-url}}" target="blank">View bracket on Start.gg</a>
        "#;
        let content = replace_placeholder_values(tournament_data, content_template);

        // Check for existing broadcast
        let existing_broadcast = self.get_broadcast_by_subject(&subject).await;
        if existing_broadcast.is_some() {
            log_skip("email", "reminder broadcast already scheduled");
            return Ok(());
        }

        // Create broadcast
        self.create_broadcast(&send_time, &subject, &content)
            .await
            .inspect_err(|e| {
                log_error("email", "reminder broadcast scheduling failed");
                println!("{:?}", e);
            })?;
        log_success("email", "reminder broadcast scheduled");

        Ok(())
    }

    /// Schedule a reminder email for the start of Top 8, if needed
    pub async fn schedule_top8_broadcast(&mut self, tournament_data: &Value) -> Result<()> {
        // Parse top 8 start time
        let top8_start_time_str = tournament_data["top8-start-time"].as_str().unwrap_or("");
        if top8_start_time_str.is_empty() {
            log_warn("email", "Missing top 8 start time");
            return Ok(());
        }
        let top8_datetime_format = "%Y-%m-%d %I:%M%P"; // e.g. "2024-10-06 3:00PM"
        let timezone: Tz = tournament_data["timezone"]
            .as_str()
            .context("Missing timezone")?
            .parse()?;
        let top8_start_time =
            NaiveDateTime::parse_from_str(top8_start_time_str, top8_datetime_format)?
                .and_local_timezone(timezone)
                .single()
                .context("Invalid top8-start-time")?;

        // Email subject
        let tournament_name = tournament_data["name"]
            .as_str()
            .context("Missing tournament name")?;
        let subject = format!("Top 8 starting now: {}", tournament_name);

        // Generate content
        let mut content_template = r#"
            <h1>{{name}}</h1>
            <br>Live now! Top 8 starts soon.
            <br><a href="{{start.gg-url}}" target="blank">Bracket</a>
        "#
        .to_string();
        let has_stream = !tournament_data["stream-url"]
            .as_str()
            .unwrap_or("")
            .is_empty();
        if has_stream {
            content_template.push_str(r#"<br><a href="{{stream-url}}" target="blank">Stream</a>"#);
        }
        let content = replace_placeholder_values(tournament_data, &content_template);

        // Check for existing broadcast
        let existing_broadcast = self.get_broadcast_by_subject(&subject).await;
        if existing_broadcast.is_some() {
            log_skip("email", "top 8 broadcast already scheduled");
            return Ok(());
        }

        // Create broadcast
        self.create_broadcast(&top8_start_time, &subject, &content)
            .await
            .inspect_err(|e| {
                log_error("email", "top 8 broadcast scheduling failed");
                println!("{:?}", e);
            })?;
        log_success("email", "top 8 broadcast scheduled");

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

    /// https://developers.kit.com/v3#create-a-broadcast
    async fn create_broadcast(
        &self,
        send_time: &DateTime<Tz>,
        subject: &str,
        content: &str,
    ) -> Result<Value> {
        // Validate send time
        let mut send_time_iso8601 = Some(send_time.to_rfc3339());
        let now = chrono::Utc::now();
        if send_time < &now {
            send_time_iso8601 = None;
            log_warn("email", "Already past send time");
            log_warn("email", "This broadcast will be created as a draft");
            log_warn("email", "You can send it manually from the web interface");
        }

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
            log_error("email", &response_code_str);
            log_error("email", &to_string_pretty(&json)?);
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
        log_error("", "Missing API secret for Kit");
        log_warn("", "Generate one here:");
        log_warn("", api_url);
        log_warn("", "Then add it to .env or run.sh");
        log_warn("", &format!("{}=your-api-secret", env_key));
        None
    }
}
