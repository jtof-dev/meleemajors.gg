use chrono::{DateTime, NaiveDateTime, Utc};
use gql_client::{Client, ClientConfig};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use urlencoding::encode;

#[tokio::main]
async fn main() {
    let token = env::var("STARTGGAPI").unwrap();
    let query_event = read_file("graphql/getTournamentEvent.gql");
    let query_entrants = read_file("graphql/getTournamentEntrants.gql");

    // iterate through tournaments in json array
    let tournaments = read_file("tournaments.json");

    let v: Value = serde_json::from_str(&tournaments).unwrap();

    match v {
        Value::Array(vec) => {
            for tournament in vec {
                let tournament_slug = tournament["start.gg-tournament-name"].as_str().unwrap();
                let event_slug = tournament["start.gg-melee-singles"].as_str().unwrap();
                let vars_event = json!({
                  "slug": tournament_slug,
                  "slug_event": event_slug
                });
                let data = scrape_data(&token, &query_event, &query_entrants, vars_event).await;
                println!("{}", data);
            }
        }
        _ => panic!("root must be an array"),
    }
}

/** Returns string contents of file with given path or panics otherwise */
fn read_file(path: &str) -> String {
    let file = File::open(path).unwrap();
    let file_contents = std::io::read_to_string(file).unwrap();
    return file_contents;
}

async fn scrape_data(
    token: &str,
    query_event: &str,
    query_entrants: &str,
    vars_event: Value,
) -> Value {
    let mut headers = HashMap::new();
    headers.insert("authorization".to_string(), format!("Bearer {}", token));

    let config = ClientConfig {
        endpoint: "https://api.start.gg/gql/alpha".to_string(),
        timeout: Some(60),
        headers: Some(headers),
        proxy: None,
    };

    let client = Client::new_with_config(config);
    let data_event = client
        .query_with_vars_unwrap::<Value, Value>(query_event, vars_event)
        .await
        .unwrap();

    let vars_entrants = json!({
      "eventId": data_event["event"].get("id").unwrap().to_string(),
    });

    let data_entrants = client
        .query_with_vars_unwrap::<Value, Value>(query_entrants, vars_entrants)
        .await
        .unwrap();

    // grab basic info from queries
    let name = data_event["tournament"]["name"].as_str().unwrap(); // ---> result
    let start_date = data_event["tournament"]["startAt"].as_number().unwrap();
    let end_date = data_event["tournament"]["endAt"].as_number().unwrap();
    let address = data_event["tournament"]["venueAddress"].as_str().unwrap();
    let city = data_event["tournament"]["city"].as_str().unwrap();
    let state = data_event["tournament"]["addrState"].as_str().unwrap();
    let entrant_count = data_entrants["event"]["numEntrants"].as_number().unwrap(); // ---> result

    // get human-readable start and date
    let naive_start_date = DateTime::from_timestamp(start_date.as_i64().unwrap(), 0).unwrap();
    let formatted_start_date = naive_start_date.format("%B %d").to_string();

    let naive_end_date = DateTime::from_timestamp(end_date.as_i64().unwrap(), 0).unwrap();
    let formatted_end_date = naive_end_date.format("%B %d").to_string();

    let start_end_date = format!("{} - {}", formatted_start_date, formatted_end_date); // ---> result

    // put together city and state
    let city_state = format!("{}, {}", city, state); // ---> result

    // wrap location into google maps link
    let google_maps_link = format!(
        "https://www.google.com/maps/search/?api=1&query={}",
        encode(address)
    ); // ---> result

    // get larger image url
    let images = data_event["tournament"]["images"].as_array().unwrap();
    let largest_image = images
        .iter()
        .max_by_key(|img| img["width"].as_u64().unwrap())
        .unwrap();
    let largest_image_url = largest_image["url"].as_str().unwrap(); // ---> result

    return json!({
      "name": name,
      "entrants": entrant_count,
      "date": start_end_date,
      "city-and-state": city_state,
      "maps-link": google_maps_link,
      "image-url": largest_image_url
    });
}

fn generate_card() {}

fn make_site() {}
