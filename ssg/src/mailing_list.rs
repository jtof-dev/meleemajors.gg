use crate::utils::{
    log_heading, log_red, log_success, log_warn, log_yellow, read_file,
    replace_placeholder_values,
};
use anyhow::{anyhow, bail, Context, Result};
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

        Ok(Self {
            client,
            api_secret,
        })
    }

    /// Schedule a reminder email for 5 days before the tournament starts
    pub async fn schedule_reminder_broadcast(&self, tournament_data: &Value) -> Result<()> {
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

        // Generate content
        let content_template =
            read_file("html/emailMessage.html").replace("{{email-intro-text}}", "This weekend:");
        let content = replace_placeholder_values(tournament_data, &content_template);

        self.create_broadcast(&send_time, &subject, &content)
            .await?;
        log_success("email", "reminder broadcast created");
        Ok(())
    }

    /// Schedule a reminder email for the start of Top 8
    pub async fn schedule_top8_broadcast(&self, tournament_data: &Value) -> Result<()> {
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
        let content_template = read_file("html/emailMessage.html")
            .replace("{{email-intro-text}}", "Top 8 starting now:");
        let content = replace_placeholder_values(tournament_data, &content_template);

        self.create_broadcast(&top8_start_time, &subject, &content)
            .await?;
        log_success("email", "top 8 broadcast created");
        Ok(())
    }

    /// Delete all scheduled (unsent) broadcasts, paginating through all results.
    /// Broadcasts that were already sent return 422 and are skipped.
    /// https://developers.kit.com/v3#list-broadcasts
    /// https://developers.kit.com/v3#destroy-a-broadcast
    pub async fn delete_scheduled_broadcasts(&self) -> Result<()> {
        log_heading("Cleaning up scheduled broadcasts");

        // Page through broadcasts newest-first, deleting each one.
        // Stop as soon as we hit one that's already sent (422),
        // since everything older is also sent.
        let mut deleted = 0;
        let page = 1;
        'outer: loop {
            let response = self
                .client
                .get("https://api.convertkit.com/v3/broadcasts")
                .query(&[
                    ("api_secret", &self.api_secret),
                    ("page", &page.to_string()),
                    ("sort_order", &"desc".to_string()),
                ])
                .send()
                .await?
                .error_for_status()?;
            let broadcasts = response.json::<Value>().await?["broadcasts"]
                .as_array()
                .context("missing broadcasts field")?
                .to_vec();
            if broadcasts.is_empty() {
                break;
            }
            for broadcast in &broadcasts {
                let id = broadcast["id"].as_i64().context("missing broadcast id")?;
                let subject = broadcast["subject"].as_str().unwrap_or("(no subject)");
                match self.delete_broadcast(id).await {
                    Ok(true) => {
                        deleted += 1;
                        log_success("email", &format!("deleted: {}", subject));
                    }
                    Ok(false) => {
                        // Hit a sent broadcast — everything older is also sent
                        break 'outer;
                    }
                    Err(e) => {
                        log_warn("email", &format!("failed to delete broadcast {}: {}", id, e));
                    }
                }
            }
            // Don't increment page — deletions shift results forward,
            // so page 1 always has the next batch
        }
        log_success("email", &format!("deleted {} scheduled broadcasts", deleted));
        Ok(())
    }

    /// Returns Ok(true) if deleted, Ok(false) if already sent (422), Err otherwise.
    async fn delete_broadcast(&self, broadcast_id: i64) -> Result<bool> {
        let response = self
            .client
            .delete(format!(
                "https://api.convertkit.com/v3/broadcasts/{}",
                broadcast_id
            ))
            .query(&[("api_secret", &self.api_secret)])
            .send()
            .await?;
        match response.status().as_u16() {
            200..=299 => Ok(true),
            422 => Ok(false), // already sent/sending
            code => bail!("Failed to delete broadcast {}: HTTP {}", broadcast_id, code),
        }
    }

    /// https://developers.kit.com/v3#create-a-broadcast
    async fn create_broadcast(
        &self,
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
        let url = "https://api.convertkit.com/v3/broadcasts";
        let res = self.client.post(url).json(&json!({
            "api_secret": &self.api_secret,
            "email_layout_template": Value::Null, // use default template
            "content": &content,
            "subject": &subject,
            "send_at": &send_time_iso8601,
            "public": true, // false == draft
        })).send().await?;
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
