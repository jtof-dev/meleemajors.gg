use case_converter::kebab_to_camel;
use chrono::DateTime;
use gql_client::{Client, ClientConfig};
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::{env, fs};
use std::fs::File;
use urlencoding::encode;

#[tokio::main]
async fn main() {
    // get start.gg api token from an environmental variable named "STARTGGAPI"
    let token = env::var("STARTGGAPI").unwrap();
    // get graphgl queries
    let query_event = read_file("graphql/getTournamentEvent.gql");
    let query_entrants = read_file("graphql/getTournamentEntrants.gql");
    // create variable that holds the website being made, starting with the header.html
    let mut temp_html = String::from(read_file("html/header.html"));
    let template_card = read_file("html/templateCard.html");

    // iterate through tournaments in json array
    let tournaments = read_file("tournaments.json");

    let v: Value = serde_json::from_str(&tournaments).unwrap();

    match v {
        Value::Array(vec) => {
            for tournament in vec {
                let tournament_data =
                    scrape_data(tournament, &token, &query_event, &query_entrants).await;
                println!("{}", tournament_data);

                // use scraped info to make a tournament card, and append it to temp_html
                temp_html.push_str(&format!("\n{}", generate_card(tournament_data, &template_card)));
            }
        }
        _ => panic!("root must be an array"),
    }
    // after all cards have been appended to temp_html, add the footer.html
    temp_html.push_str(&format!("\n{}", read_file("html/footer.html")));
    fs::write("index.html", temp_html).unwrap();
}

// returns string contents of file with given path or panics otherwise
fn read_file(path: &str) -> String {
    let file = File::open(path).unwrap();
    let file_contents = std::io::read_to_string(file).unwrap();
    return file_contents;
}

async fn scrape_data(
    tournament: Value,
    token: &str,
    query_event: &str,
    query_entrants: &str,
) -> Value {
    // scrape tournament info for a specific tournament entry
    let startgg_url = tournament["start.gg-melee-singles-url"].as_str().unwrap();
    let regex_startgg_url = Regex::new(r"^(https?://)?(www\.)?start\.gg/").unwrap();
    let event_slug = regex_startgg_url.replace(startgg_url, "");
    let parts: Vec<&str> = event_slug.split("/").collect();
    let tournament_part = parts.get(1).unwrap_or(&"");
    let tournament_slug = tournament_part.to_string();
    let vars_event = json!({
      "slug": tournament_slug,
      "slug_event": event_slug
    });
    // println!("{}", vars_event);
    // format header with api token
    let mut headers = HashMap::new();
    headers.insert("authorization".to_string(), format!("Bearer {}", token));

    // query start.gg with getTournamentEvent.gql
    let config = ClientConfig {
        endpoint: "https://api.start.gg/gql/alpha".to_string(),
        timeout: Some(60),
        headers: Some(headers),
        proxy: None,
    };

    let client = Client::new_with_config(config);
    let data_event = client
        .query_with_vars_unwrap::<Value, Value>(query_event, vars_event.clone())
        .await
        .unwrap();

    let vars_entrants = json!({
      "eventId": data_event["event"].get("id").unwrap().to_string(),
    });

    // query start.gg with getTournamentEntrants.gql
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

    // get start.gg url
    let startgg_url = format!(
        "https://www.start.gg/tournament/{}",
        vars_event["slug"].as_str().unwrap()
    ); // ---> result

    // convert start.gg kebab case name to camel case to keep a consistent naming scheme
    let startgg_tournament_name = kebab_to_camel(&tournament_slug);
    return json!({
      "name": name,
      "entrants": entrant_count,
      "date": start_end_date,
      "city-and-state": city_state,
      "maps-link": google_maps_link,
      "image-url": largest_image_url,
      "start.gg-url": startgg_url,
      "start.gg-tournament-name": startgg_tournament_name,
      "player0": tournament["featured-players"][0],
      "player1": tournament["featured-players"][1],
      "player2": tournament["featured-players"][2],
      "player3": tournament["featured-players"][3],
      "player4": tournament["featured-players"][4],
      "player5": tournament["featured-players"][5],
      "player6": tournament["featured-players"][6],
      "player7": tournament["featured-players"][7],
      "stream-url": tournament["stream-url"],
      "schedule-url": tournament["schedule-url"],
    });
}

fn generate_card(tournament_data: Value, template_card: &str) -> String {
    let schedule_link_class = match tournament_data["schedule-url"].as_str().unwrap() {
        "" => " hidden",
        _ => "",
    };
    let stream_link_class = match tournament_data["stream-url"].as_str().unwrap() {
        "" => " hidden",
        _ => "",
    };

    let temp_card = template_card
        .replace("{{start.gg-tournament-name}}", tournament_data["start.gg-tournament-name"].as_str().unwrap())
        .replace("{{start.gg-url}}", tournament_data["start.gg-url"].as_str().unwrap())
        .replace("{{schedule-url}}", tournament_data["schedule-url"].as_str().unwrap())
        .replace("{{stream-url}}", tournament_data["stream-url"].as_str().unwrap())
        .replace("{{name}}", tournament_data["name"].as_str().unwrap())
        .replace("{{date}}", tournament_data["date"].as_str().unwrap())
        .replace("{{maps-link}}", tournament_data["maps-link"].as_str().unwrap())
        .replace("{{city-and-state}}", tournament_data["city-and-state"].as_str().unwrap())
        .replace("{{entrants}}", tournament_data["entrants"].as_number().unwrap().to_string().as_str())
        .replace("{{player0}}", tournament_data["player0"].as_str().unwrap())
        .replace("{{player1}}", tournament_data["player1"].as_str().unwrap())
        .replace("{{player2}}", tournament_data["player2"].as_str().unwrap())
        .replace("{{player3}}", tournament_data["player3"].as_str().unwrap())
        .replace("{{player4}}", tournament_data["player4"].as_str().unwrap())
        .replace("{{player5}}", tournament_data["player5"].as_str().unwrap())
        .replace("{{player6}}", tournament_data["player6"].as_str().unwrap())
        .replace("{{player7}}", tournament_data["player7"].as_str().unwrap())
        .replace("{{schedule-link-class}}", schedule_link_class)
        .replace("{{stream-link-class}}", stream_link_class);
    return temp_card;
}

// fn make_site() {}
