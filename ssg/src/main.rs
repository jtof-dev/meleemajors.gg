extern crate dotenv;

use case_converter::kebab_to_camel;
use chrono::DateTime;
use chrono_tz::Tz;
use dotenv::dotenv;
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::event::{FfmpegEvent, LogLevel};
use fs_extra::{copy_items, dir};
use gql_client::{Client, ClientConfig};
use icalendar::{Calendar, Class, Component, Event, EventLike};
use itertools::Itertools;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use std::{env, fs};
use tokio::time::sleep;
use urlencoding::encode;
use utils::{
    absolute_path, log_error, log_green, log_grey, log_heading, log_info, log_red, log_skip,
    log_success, log_warn, read_file, replace_placeholder_values,
};

mod generate_gql;
mod mailing_list;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok(); // Read vars from .env file if present
    let api_token = env::var("STARTGGAPI").expect("STARTGGAPI environmental variable not found!");

    let args: Vec<String> = std::env::args().collect();
    if args.contains(&String::from("--generate")) {
        generate_gql::main();
        return;
    }

    // Whether to exit early for debug after a single iteration without writing
    // Usage: `cargo run -- --bail`
    let bail = args.contains(&String::from("--bail"));

    let mut query_headers = HashMap::new();
    query_headers.insert("authorization".to_string(), format!("Bearer {}", api_token));
    let query_config = ClientConfig {
        endpoint: "https://api.start.gg/gql/alpha".to_string(),
        timeout: Some(60),
        headers: Some(query_headers),
        proxy: None,
    };
    let query_client = Client::new_with_config(query_config);
    let query_tournament_info = read_file("graphql/getTournamentInfo.gql");
    let query_tournament_entrants = read_file("graphql/getTournamentEntrants.gql");
    let query_featured_players = read_file("graphql/getFeaturedPlayers.gql");
    let json_featured_players: Value = serde_json::from_str(&read_file("topPlayers.json")).unwrap();
    let template_header_html = read_file("html/header.html");
    let mut index_html: String = "".to_string();
    let template_card = read_file("html/templateCard.html");
    let index_footer_html = read_file("html/footer.html");
    let mut calendar_ics = Calendar::new().name("upcoming melee majors").done();
    let tournaments = read_file("tournaments.json");
    let json_tournaments: Value = serde_json::from_str(&tournaments).unwrap();
    let mut all_images: HashSet<String> = HashSet::new();

    let mut mailing_list = mailing_list::MailingListService::new()
        .inspect_err(|e| {
            log_warn("email", "Mailing list service init failed");
            log_warn("email", "Mailing list service init failed");
            log_warn("email", &format!("{:?}", e));
        })
        .ok();

    match json_tournaments {
        Value::Array(vec) => {
            for tournament in vec.iter().enumerate() {
                let tournament_data = scrape_data(
                    tournament.1,
                    query_client.clone(),
                    &query_tournament_info,
                    &query_tournament_entrants,
                    &query_featured_players,
                    &json_featured_players,
                    &mut all_images,
                )
                .await;

                // println!("{}", tournament_data);

                if tournament.0 == 0 {
                index_html =
                    replace_placeholder_values(&tournament_data, &template_header_html);
                }

                index_html.push_str(&replace_placeholder_values(
                    &tournament_data,
                    &template_card,
                ));

                calendar_ics = generate_calendar(tournament_data.clone(), &mut calendar_ics);
                log_success("calendar", "generated ICS event");

                if let Some(ref mut service) = mailing_list {
                    service
                        .schedule_reminder_broadcast(&tournament_data)
                        .await
                        .or_else(|e| {
                            log_error("email", "Failed to schedule reminder broadcast");
                            log_red(&e.to_string());
                            Err(e)
                        })
                        .ok();
                    service
                        .schedule_top8_broadcast(&tournament_data)
                        .await
                        .or_else(|e| {
                            log_error("email", "Failed to schedule top-8 broadcast");
                            log_red(&e.to_string());
                            Err(e)
                        })
                        .ok();
                } else {
                    log_warn("email", "Skipping email scheduling");
                }

                if bail {
                    std::process::exit(0)
                }
            }
        }
        _ => panic!("root must be an array"),
    }
    index_html.push_str(&format!("\n{}", index_footer_html));
    make_site(&index_html);
    make_calendar(calendar_ics);
    cleanup_images(all_images);
    next_steps();
}

async fn scrape_data(
    tournament: &Value,
    query_client: Client,
    query_tournament_info: &str,
    query_tournament_entrants: &str,
    query_featured_players: &str,
    featured_players_json: &Value,
    all_images: &mut HashSet<String>,
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

    let tournament_info = &result_tournament_info["tournament"];
    // println!("{tournament_info}");

    let name = tournament_info["name"].as_str().unwrap(); // ---> result

    log_heading(name);
    log_success("start.gg", "scraped tournaments");

    let tournament_entrants_var = json!({
      "eventId": result_tournament_info["event"].get("id").unwrap().to_string(),
    });
    let result_entrant_count = graphql_query(
        query_client.clone(),
        query_tournament_entrants,
        tournament_entrants_var,
    )
    .await;
    log_success("start.gg", "scraped entrants");

    let featured_players_vars = json!({
        "slug_event": event_slug
    });
    let result_featured_players =
        graphql_query(query_client, query_featured_players, featured_players_vars)
            .await
            .to_string();
    log_success("start.gg", "scraped top 8 players");

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

    let timezone: Tz = tournament_info["timezone"]
        .as_str()
        .unwrap()
        .parse()
        .expect("Invalid timezone");

    let start_date = unix_timestamp_to_readable_date(&tournament_info["startAt"], timezone);

    let end_date = unix_timestamp_to_readable_date(&tournament_info["endAt"], timezone);

    let date = format!("{start_date} - {end_date}"); // ---> result

    // let city = tournament_info["city"].as_str().unwrap();
    let city = tournament_info["city"].as_str().unwrap_or("Unknown");
    let state = tournament_info["addrState"].as_str().unwrap();

    let city_and_state = format!("{}, {}", city, state); // ---> result

    let address = tournament_info["venueAddress"].as_str().unwrap();

    let banner_images = tournament_info["images"].as_array().unwrap();
    let banner_image_largest = banner_images
        .iter()
        .max_by_key(|img| img["width"].as_u64().unwrap())
        .unwrap();
    let banner_image_largest_url = banner_image_largest["url"].as_str().unwrap();
    let banner_url = Regex::new(r"\?.*")
        .unwrap()
        .replace(banner_image_largest_url, "");

    let name_camel = kebab_to_camel(&tournament_slug);

    download_tournament_image(&banner_url, &name_camel, all_images);

    let stream_url = tournament["stream-url"].as_str().unwrap();
    let schedule_url = tournament["schedule-url"].as_str().unwrap();

    json!({
        "start.gg-tournament-name": name_camel,
        "name": check_override(tournament, name.to_string(), "name"),
        "date": date,
        "start-unix-timestamp": tournament_info["startAt"],
        "end-unix-timestamp": tournament_info["endAt"],
        "timezone": tournament_info["timezone"],
        "player0": check_override(tournament, featured_players_top_eight[0].to_string(), "player0"),
        "player1": check_override(tournament, featured_players_top_eight[1].to_string(), "player1"),
        "player2": check_override(tournament, featured_players_top_eight[2].to_string(), "player2"),
        "player3": check_override(tournament, featured_players_top_eight[3].to_string(), "player3"),
        "player4": check_override(tournament, featured_players_top_eight[4].to_string(), "player4"),
        "player5": check_override(tournament, featured_players_top_eight[5].to_string(), "player5"),
        "player6": check_override(tournament, featured_players_top_eight[6].to_string(), "player6"),
        "player7": check_override(tournament, featured_players_top_eight[7].to_string(), "player7"),
        "entrants": entrant_count_string,
        "city-and-state": check_override(tournament, city_and_state.to_string(), "city-and-state"),
        "maps-link": format!("https://www.google.com/maps/search/?api=1&query={}", encode(address)),
        "full-address": address,
        "start.gg-url": melee_singles_url,
        "stream-url": stream_url,
        "schedule-url": schedule_url,
        "schedule-link-class": if schedule_url.is_empty() {" hidden"} else {""},
        "stream-link-class": if stream_url.is_empty() {" hidden"} else {""},
        "top8-start-time": tournament["top8-start-time"],
    })
}

fn check_override(tournament: &Value, default_value: String, default_key: &str) -> String {
    let obj_tournament = tournament.as_object().unwrap();
    if obj_tournament.contains_key(default_key) {
        return format!("{}", obj_tournament[default_key].as_str().unwrap());
    }
    return format!("{}", default_value);
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

fn download_tournament_image(url: &str, name: &str, image_data: &mut HashSet<String>) {
    // ffmpeg -i "image_url" -vf "scale=-1:340" "tournament_name".webp

    let image_path = absolute_path(&format!("cards/{name}.webp"));

    image_data.insert(format!("{name}.webp"));

    if fs::metadata(&image_path).is_ok() {
        log_skip("ffmpeg", &format!("{name}.webp already exists"));
    } else {
        println!("[ffmpeg] downloading {url}");
        FfmpegCommand::new()
            .input(url)
            .args(["-vf", "scale=-1:340"])
            .overwrite()
            .output(image_path)
            .spawn()
            .unwrap()
            .iter()
            .unwrap()
            .for_each(|event| match event {
                FfmpegEvent::Log(LogLevel::Error | LogLevel::Fatal, msg) => {
                    log_error("ffmpeg", &format!("{:?}", msg));
                }
                FfmpegEvent::Progress(progress) => {
                    log_info("ffmpeg", &format!("{:?}", progress));
                }
                FfmpegEvent::Done => {
                    log_success("ffmpeg", &format!("{name}.webp downloaded"));
                }
                _ => {}
            });
    }
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
                .description(&replace_placeholder_values(
                    &tournament_data,
                    "{{start.gg-url}}\n\nattendees: {{entrants}}\n\nnotable entrants:\n\n{{player0}}\n{{player1}}\n{{player2}}\n{{player3}}\n{{player4}}\n{{player5}}\n{{player6}}\n{{player7}}\n",
                ))
                .class(Class::Public)
                .location(tournament_data["full-address"].as_str().unwrap())
                .done(),
        )
        .done();
}

fn make_site(index_html: &str) {
    fs::write(absolute_path("../../site/index.html"), index_html).unwrap();
    fs::remove_dir_all(absolute_path("../../site/assets/cards")).unwrap();

    copy_items(
        &[absolute_path("cards")],
        absolute_path("../../site/assets/"),
        &dir::CopyOptions::new().overwrite(true),
    )
    .unwrap();
}

fn make_calendar(calendar_ics: Calendar) {
    fs::write(
        absolute_path("../../site/calendar.ics"),
        calendar_ics.to_string(),
    )
    .unwrap();
}

fn cleanup_images(data: HashSet<String>) {
    let images = fs::read_dir(absolute_path("../../site/assets/cards")).unwrap();
    images.for_each(|image| {
        let image_str = image
            .unwrap()
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        if !data.contains(&image_str) {
            fs::remove_file(format!("cards/{image_str}")).ok();
        };
    })
}

fn next_steps() {
    log_green("\n🎉 Finished 🎉\n");
    log_grey("Next steps:");
    log_grey("1. preview locally w/ e.g. \"live-server site\"");
    log_grey("2. git commit & push to main to deploy site");
    log_grey("3. Review scheduled emails: https://app.kit.com/campaigns");
}
