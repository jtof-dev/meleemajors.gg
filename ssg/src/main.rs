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
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&String::from("--generate")) {
        generate_gql::main();
        return;
    }
    let startgg_api_token = env::var("STARTGGAPI").unwrap();
    let mut startgg_query_headers = HashMap::new();
    startgg_query_headers.insert(
        "authorization".to_string(),
        format!("Bearer {}", startgg_api_token),
    );
    let startgg_query_config = ClientConfig {
        endpoint: "https://api.start.gg/gql/alpha".to_string(),
        timeout: Some(60),
        headers: Some(startgg_query_headers),
        proxy: None,
    };
    let startgg_query_client = Client::new_with_config(startgg_query_config);
    let startgg_query_tournament_info = read_file("graphql/getTournamentInfo.gql");
    let startgg_query_tournament_entrants = read_file("graphql/getTournamentEntrants.gql");
    let startgg_query_featured_players = read_file("graphql/getFeaturedPlayers.gql");
    let featured_players_json: Value = serde_json::from_str(&read_file("topPlayers.json")).unwrap();
    let mut index_html = read_file("html/header.html");
    let template_card = read_file("html/templateCard.html");
    let mut calendar_ics = Calendar::new().name("upcoming melee majors").done();
    let tournaments = read_file("tournaments.json");
    let tournaments_json: Value = serde_json::from_str(&tournaments).unwrap();

    match tournaments_json {
        Value::Array(vec) => {
            for tournament in vec {
                let tournament_data = scrape_data(
                    tournament,
                    startgg_query_client.clone(),
                    &startgg_query_tournament_info,
                    &startgg_query_tournament_entrants,
                    &startgg_query_featured_players,
                    &featured_players_json,
                )
                .await;

                index_html.push_str(&format!(
                    "\n{}",
                    generate_card(tournament_data.clone(), &template_card)
                ));
                calendar_ics = generate_calendar(tournament_data, &mut calendar_ics);
            }
        }
        _ => panic!("root must be an array"),
    }
    index_html.push_str(&format!("\n{}", read_file("html/footer.html")));
    make_site(&index_html);
    make_calendar(calendar_ics);
}

pub fn read_file(path: &str) -> String {
    let file = File::open(path).unwrap();
    std::io::read_to_string(file).unwrap()
}

async fn scrape_data(
    tournament: Value,
    query_client: Client,
    query_tournament_info: &str,
    query_tournament_entrants: &str,
    query_featured_players: &str,
    featured_players_json: &Value,
) -> Value {
    let melee_singles_url = tournament["start.gg-melee-singles-url"].as_str().unwrap();
    let event_slug = Regex::new(r"^(https?://)?(www\.)?start\.gg/")
        .unwrap()
        .replace(melee_singles_url, "");

    let event_slug_parts: Vec<&str> = event_slug.split('/').collect();
    let tournament_slug = event_slug_parts.get(1).unwrap_or(&"").to_string();
    let tournament_info_vars = json!({
      "slug": tournament_slug,
      "slug_event": event_slug
    });

    let result_tournament_info = graphql_query(
        query_client.clone(),
        query_tournament_info,
        tournament_info_vars.clone(),
    )
    .await;
    let tournament_name = result_tournament_info["tournament"]["name"]
        .as_str()
        .unwrap(); // ---> result
    println!("successfully scraped data for {}", tournament_name);

    let tournament_entrants_var = json!({
      "eventId": result_tournament_info["event"].get("id").unwrap().to_string(),
    });
    let result_entrant_count = graphql_query(
        query_client.clone(),
        query_tournament_entrants,
        tournament_entrants_var,
    )
    .await;
    println!("successfully scraped entrants for {}", tournament_name);

    let featured_players_vars = json!({
        "slug_event": event_slug
    });
    let result_featured_players =
        graphql_query(query_client, query_featured_players, featured_players_vars)
            .await
            .to_string();
    println!(
        "successfully scraped top eight players for {}",
        tournament_name
    );

    let featured_players_top_eight = featured_players_json
        .as_array()
        .unwrap()
        .iter()
        .filter(|player| {
            result_featured_players.contains(&(player.as_str().unwrap().to_owned() + "\""))
        })
        .take(8)
        .map(|player| player.as_str().unwrap())
        .pad_using(8, |_| "TBD")
        .collect::<Vec<&str>>();

    let entrant_count = result_entrant_count["event"]["numEntrants"].as_number();
    let entrant_count_string = match entrant_count {
        Some(entrant_count) => entrant_count.to_string(),
        None => "TBD".to_string(),
    }; // ---> result

    let timezone: Tz = result_tournament_info["tournament"]["timezone"]
        .as_str()
        .unwrap()
        .parse()
        .expect("Invalid timezone");

    let start_date =
        unix_timestamp_to_readable_date(&result_tournament_info["tournament"]["startAt"], timezone);

    let end_date =
        unix_timestamp_to_readable_date(&result_tournament_info["tournament"]["endAt"], timezone);

    let date = format!("{start_date} - {end_date}"); // ---> result

    let city = result_tournament_info["tournament"]["city"]
        .as_str()
        .unwrap();
    let state = result_tournament_info["tournament"]["addrState"]
        .as_str()
        .unwrap();
    let city_and_state = format!("{}, {}", city, state); // ---> result

    let address = result_tournament_info["tournament"]["venueAddress"]
        .as_str()
        .unwrap();
    let google_maps_link = format!(
        "https://www.google.com/maps/search/?api=1&query={}",
        encode(address)
    ); // ---> result

    let banner_images = result_tournament_info["tournament"]["images"]
        .as_array()
        .unwrap();
    let banner_image_largest = banner_images
        .iter()
        .max_by_key(|img| img["width"].as_u64().unwrap())
        .unwrap();
    let banner_image_largest_url = banner_image_largest["url"].as_str().unwrap();
    let banner_url = Regex::new(r"\?.*")
        .unwrap()
        .replace(banner_image_largest_url, "");

    let name_camel = kebab_to_camel(&tournament_slug);
    let stream_url = tournament["stream-url"].as_str().unwrap();
    let schedule_url = tournament["schedule-url"].as_str().unwrap();

    download_tournament_image(&banner_url, &name_camel);

    json!({
        "start.gg-tournament-name": name_camel,
        "name": tournament_name,
        "date": date,
        "start-unix-timestamp": result_tournament_info["tournament"]["startAt"],
        "end-unix-timestamp": result_tournament_info["tournament"]["endAt"],
        "player0": featured_players_top_eight[0],
        "player1": featured_players_top_eight[1],
        "player2": featured_players_top_eight[2],
        "player3": featured_players_top_eight[3],
        "player4": featured_players_top_eight[4],
        "player5": featured_players_top_eight[5],
        "player6": featured_players_top_eight[6],
        "player7": featured_players_top_eight[7],
        "entrants": entrant_count_string,
        "city-and-state": city_and_state,
        "maps-link": google_maps_link,
        "full-address": address,
        "start.gg-url": melee_singles_url,
        "stream-url": stream_url,
        "schedule-url": schedule_url
    })
}

async fn graphql_query(client: Client, query: &str, vars: Value) -> Value {
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

fn unix_timestamp_to_readable_date(date: &Value, timezone: Tz) -> String {
    DateTime::from_timestamp(date.as_i64().unwrap(), 0)
        .unwrap()
        .with_timezone(&timezone)
        .format("%B %d")
        .to_string()
}

fn download_tournament_image(url: &str, name: &str) {
    // ffmpeg -i "image_url" -vf "scale=-1:340" "tournament_name".webp
    println!("[ffmpeg] downloading {url}");
    FfmpegCommand::new()
        .input(url)
        .args(["-vf", "scale=-1:340"])
        .output(format!("cards/{}.webp", name))
        .spawn()
        .unwrap()
        .iter()
        .unwrap()
        .for_each(|event| match event {
            FfmpegEvent::Log(LogLevel::Error | LogLevel::Fatal, msg) => {
                eprintln!("[ffmpeg] {:?}", msg)
            }
            FfmpegEvent::Progress(progress) => println!("[ffmpeg] {:?}", progress),
            FfmpegEvent::Done => println!("[ffmpeg] downloaded {url}"),
            _ => {}
        });
}

fn generate_calendar(tournament_data: Value, calendar_ics: &mut Calendar) -> Calendar {
    return calendar_ics
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

    let tournament_card = template_card
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
    tournament_card
}

fn make_site(index_html: &str) {
    fs::write("../../site/index.html", index_html).unwrap();
    fs::remove_dir_all("../../site/assets/cards").unwrap();

    Command::new("cp")
        .args(["-r", "cards", "../../site/assets"])
        .output()
        .unwrap();

    Command::new("rm").args(["-rf", "cards"]).output().unwrap();

    Command::new("mkdir").arg("cards").output().unwrap();
}

fn make_calendar(calendar_ics: Calendar) {
    fs::write("../../site/calendar.ics", calendar_ics.to_string()).unwrap();
}
