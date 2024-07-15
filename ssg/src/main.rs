use case_converter::kebab_to_camel;
use chrono::DateTime;
use chrono_tz::Tz;
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::event::{FfmpegEvent, LogLevel};
use gql_client::{Client, ClientConfig};
use icalendar::{Calendar, Class, Component, Event, EventLike};
use itertools::Itertools;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::process::Command;
use std::time::Duration;
use std::{env, fs};
use tokio::time::sleep;
use urlencoding::encode;

mod generate_gql;

#[tokio::main]
async fn main() {
    // break off into generate.rs if --generate flag is passed
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&String::from("--generate")) {
        generate_gql::main();
        return;
    }

    // define constant graphql variables for scrape_data()
    let token = env::var("STARTGGAPI").unwrap();
    let mut headers = HashMap::new();
    headers.insert("authorization".to_string(), format!("Bearer {}", token));
    let config = ClientConfig {
        endpoint: "https://api.start.gg/gql/alpha".to_string(),
        timeout: Some(60),
        headers: Some(headers),
        proxy: None,
    };
    let client = Client::new_with_config(config);

    // get graphgl queries
    let query_event = read_file("graphql/getTournamentEvent.gql");
    let query_entrants = read_file("graphql/getTournamentEntrants.gql");
    let query_top_players = read_file("graphql/getFeaturedPlayers.gql");

    // read top players json
    let top_players_json: Value = serde_json::from_str(&read_file("topPlayers.json")).unwrap();

    // create variable that holds the website being made, starting with the header.html
    let mut temp_html = read_file("html/header.html");
    let template_card = read_file("html/templateCard.html");

    // create variable that holds the calendar subscription
    let mut temp_calendar = Calendar::new().name("upcoming melee majors").done();

    // iterate through tournaments in json array
    let tournaments = read_file("tournaments.json");

    let v: Value = serde_json::from_str(&tournaments).unwrap();

    match v {
        Value::Array(vec) => {
            for tournament in vec {
                let tournament_data = scrape_data(
                    tournament,
                    client.clone(),
                    &query_event,
                    &query_entrants,
                    &query_top_players,
                    &top_players_json,
                )
                .await;

                // use scraped info to make a tournament card, and append it to temp_html
                temp_html.push_str(&format!(
                    "\n{}",
                    generate_card(tournament_data.clone(), &template_card)
                ));
                temp_calendar = generate_calendar(tournament_data, &mut temp_calendar);
            }
        }
        _ => panic!("root must be an array"),
    }
    // after all cards have been appended to temp_html, add the footer.html
    temp_html.push_str(&format!("\n{}", read_file("html/footer.html")));
    make_site(&temp_html);
    make_calendar(temp_calendar);
}

// returns string contents of file with given path or panics otherwise
pub fn read_file(path: &str) -> String {
    let file = File::open(path).unwrap();
    std::io::read_to_string(file).unwrap()
}

async fn scrape_data(
    tournament: Value,
    client: Client,
    query_event: &str,
    query_entrants: &str,
    query_top_players: &str,
    top_players_json: &Value,
) -> Value {
    // scrape tournament info for a specific tournament entry
    let startgg_url = tournament["start.gg-melee-singles-url"].as_str().unwrap();
    let regex_startgg_url = Regex::new(r"^(https?://)?(www\.)?start\.gg/").unwrap();
    let event_slug = regex_startgg_url.replace(startgg_url, "");
    let parts: Vec<&str> = event_slug.split('/').collect();
    let tournament_part = parts.get(1).unwrap_or(&"");
    let tournament_slug = tournament_part.to_string();
    let vars_event = json!({
      "slug": tournament_slug,
      "slug_event": event_slug
    });

    // get tournament data from queries using graphql_query()
    let data_event = graphql_query(client.clone(), query_event, vars_event.clone()).await;
    // get name early for logging
    let name = data_event["tournament"]["name"].as_str().unwrap(); // ---> result
    println!("Successfully scraped data for {}", name);
    let vars_entrants = json!({
      "eventId": data_event["event"].get("id").unwrap().to_string(),
    });
    // get entrant info from graphql_query()
    let data_entrants = graphql_query(client.clone(), query_entrants, vars_entrants).await;
    println!("Successfully scraped entrants for {}", name);
    let vars_top_players = json!({
        "slug_event": event_slug
    });
    let data_entrants_top_players = graphql_query(client, query_top_players, vars_top_players)
        .await
        .to_string();
    println!("Successfully scraped top eight players for {}", name);

    let top_eight = top_players_json
        .as_array()
        .unwrap()
        .iter()
        .filter(|player| {
            data_entrants_top_players.contains(&(player.as_str().unwrap().to_owned() + "\""))
        })
        .take(8)
        .map(|player| player.as_str().unwrap())
        .pad_using(8, |_| "TBD")
        .collect::<Vec<&str>>();

    // grab basic info from queries
    let start_date = data_event["tournament"]["startAt"].as_number().unwrap();
    let end_date = data_event["tournament"]["endAt"].as_number().unwrap();
    let address = data_event["tournament"]["venueAddress"].as_str().unwrap();
    let city = data_event["tournament"]["city"].as_str().unwrap();
    let state = data_event["tournament"]["addrState"].as_str().unwrap();

    // get number of entrants, and assign "TBD" if start.gg returns a null value
    let entrant_count = data_entrants["event"]["numEntrants"].as_number();
    let entrant_count_str = match entrant_count {
        Some(number) => number.to_string(),
        None => "TBD".to_string(),
    }; // ---> result

    // get human-readable start and end date
    // first, get timezone for event
    let event_timezone = data_event["tournament"]["timezone"].as_str().unwrap();
    let formatted_event_timezone: Tz = event_timezone.parse().expect("Invalid timezone");
    // start date
    let naive_start_date = DateTime::from_timestamp(start_date.as_i64().unwrap(), 0)
        .unwrap()
        .with_timezone(&formatted_event_timezone);
    let formatted_start_date = naive_start_date.format("%B %d").to_string();

    // end date
    let naive_end_date = DateTime::from_timestamp(end_date.as_i64().unwrap(), 0)
        .unwrap()
        .with_timezone(&formatted_event_timezone);
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
    let largest_image_url = largest_image["url"].as_str().unwrap();
    let cleaned_largest_image_url = Regex::new(r"\?.*").unwrap().replace(largest_image_url, "");

    // get start.gg url
    let startgg_url = format!(
        "https://www.start.gg/{}",
        vars_event["slug_event"].as_str().unwrap()
    ); // ---> result

    // convert start.gg kebab case name to camel case to keep a consistent naming scheme
    let startgg_tournament_name = kebab_to_camel(&tournament_slug);

    download_tournament_image(&cleaned_largest_image_url, &startgg_tournament_name);

    json!({
        "start.gg-tournament-name": startgg_tournament_name,
        "name": name,
        "date": start_end_date,
        "start-unix-timestamp": start_date,
        "end-unix-timestamp": end_date,
        "player0": top_eight[0],
        "player1": top_eight[1],
        "player2": top_eight[2],
        "player3": top_eight[3],
        "player4": top_eight[4],
        "player5": top_eight[5],
        "player6": top_eight[6],
        "player7": top_eight[7],
        "entrants": entrant_count_str,
        "city-and-state": city_state,
        "maps-link": google_maps_link,
        "full-address": address,
        "start.gg-url": startgg_url,
        "stream-url": tournament["stream-url"],
        "schedule-url": tournament["schedule-url"]
    })
}

async fn graphql_query(client: Client, query: &str, vars: Value) -> Value {
    // infinitely requery start.gg if a query fails (which happens often) - and only return a successful response
    loop {
        match client
            .query_with_vars_unwrap::<Value, Value>(query, vars.clone())
            .await
        {
            Ok(data) => {
                return data;
            }
            Err(e) => {
                println!("Error while querying: {:?}", e);
                println!("Retrying in 10 seconds...");
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
}

/// ffmpeg -i "image_url" -vf "scale=-1:340" "tournament_name".webp
fn download_tournament_image(image_url: &str, tournament_name: &str) {
    println!("[ffmpeg] downloading {image_url}");
    FfmpegCommand::new()
        .input(image_url)
        .args(["-vf", "scale=-1:340"])
        .output(format!("cards/{}.webp", tournament_name))
        .spawn()
        .unwrap()
        .iter()
        .unwrap()
        .for_each(|event| {
            match event {
                FfmpegEvent::Log(LogLevel::Error | LogLevel::Fatal, msg) => eprintln!("[ffmpeg] {:?}", msg),
                FfmpegEvent::Progress(progress) => println!("[ffmpeg] {:?}", progress),
                FfmpegEvent::Done => println!("[ffmpeg] downloaded {image_url}"),
                _ => {}
            }
        });
}

fn generate_calendar(tournament_data: Value, temp_calendar: &mut Calendar) -> Calendar {
    return temp_calendar
        .push(
            Event::new()
                .starts(
                    DateTime::from_timestamp(
                        tournament_data["start-unix-timestamp"]
                            .as_number()
                            .unwrap()
                            .as_i64()
                            .unwrap(),
                        0,
                    )
                    .unwrap()
                    .date_naive(),
                )
                .ends(
                    DateTime::from_timestamp(
                        tournament_data["end-unix-timestamp"]
                            .as_number()
                            .unwrap()
                            .as_i64()
                            .unwrap(),
                        0,
                    )
                    .unwrap()
                    .date_naive(),
                )
                .summary(tournament_data["name"].as_str().unwrap())
                .description(&format!(
                    "{}\n\nattendees: {}\n\nnotable entrants:\n\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
                    tournament_data["start.gg-url"].as_str().unwrap(),
                    tournament_data["entrants"].as_str().unwrap(),
                    tournament_data["player0"].as_str().unwrap(),
                    tournament_data["player1"].as_str().unwrap(),
                    tournament_data["player2"].as_str().unwrap(),
                    tournament_data["player3"].as_str().unwrap(),
                    tournament_data["player4"].as_str().unwrap(),
                    tournament_data["player5"].as_str().unwrap(),
                    tournament_data["player6"].as_str().unwrap(),
                    tournament_data["player7"].as_str().unwrap()
                ))
                .class(Class::Public)
                .location(tournament_data["full-address"].as_str().unwrap())
                .done(),
        )
        .done();
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
        .replace(
            "{{start.gg-tournament-name}}",
            tournament_data["start.gg-tournament-name"]
                .as_str()
                .unwrap(),
        )
        .replace(
            "{{start.gg-url}}",
            tournament_data["start.gg-url"].as_str().unwrap(),
        )
        .replace(
            "{{schedule-url}}",
            tournament_data["schedule-url"].as_str().unwrap(),
        )
        .replace(
            "{{stream-url}}",
            tournament_data["stream-url"].as_str().unwrap(),
        )
        .replace("{{name}}", tournament_data["name"].as_str().unwrap())
        .replace("{{date}}", tournament_data["date"].as_str().unwrap())
        .replace(
            "{{maps-link}}",
            tournament_data["maps-link"].as_str().unwrap(),
        )
        .replace(
            "{{city-and-state}}",
            tournament_data["city-and-state"].as_str().unwrap(),
        )
        .replace(
            "{{entrants}}",
            tournament_data["entrants"].as_str().unwrap(),
        )
        .replace("{{player0}}", tournament_data["player0"].as_str().unwrap())
        .replace("{{player1}}", tournament_data["player1"].as_str().unwrap())
        .replace("{{player2}}", tournament_data["player2"].as_str().unwrap())
        .replace("{{player3}}", tournament_data["player3"].as_str().unwrap())
        .replace("{{player4}}", tournament_data["player4"].as_str().unwrap())
        .replace("{{player5}}", tournament_data["player5"].as_str().unwrap())
        .replace("{{player6}}", tournament_data["player6"].as_str().unwrap())
        .replace("{{player7}}", tournament_data["player7"].as_str().unwrap())
        .replace("{{schedule-link-class}}", schedule_link_class)
        .replace("{{stream-link-class}}", stream_link_class)
        .replace(
            "{{start-unix-timestamp}}",
            &tournament_data["start-unix-timestamp"]
                .as_number()
                .unwrap()
                .to_string(),
        )
        .replace(
            "{{end-unix-timestamp}}",
            &tournament_data["end-unix-timestamp"]
                .as_number()
                .unwrap()
                .to_string(),
        );
    temp_card
}

fn make_site(temp_html: &str) {
    fs::write("../../site/index.html", temp_html).unwrap();
    fs::remove_dir_all("../../site/assets/cards").unwrap();

    Command::new("cp")
        .args(["-r", "cards", "../../site/assets"])
        .output()
        .unwrap();

    Command::new("rm").args(["-rf", "cards"]).output().unwrap();

    Command::new("mkdir").arg("cards").output().unwrap();
}

fn make_calendar(temp_calendar: Calendar) {
    fs::write("../../site/calendar.ics", temp_calendar.to_string()).unwrap();
}
