use anyhow::{Context, Result};
use reqwest;
use scraper::{Html, Selector};
use serde_json;
use std::collections::HashSet;
use std::fs;

use crate::utils::{log_green, log_info, log_success, log_warn};

pub async fn main() -> Result<()> {
    log_info("rankings", "Fetching latest SSBMRank from Liquipedia...");

    // Fetch the SSBMRank page
    let url = "https://liquipedia.net/smash/SSBMRank";
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; MeleeMajorsBot/1.0)")
        .build()?;

    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to fetch SSBMRank page")?;

    let html_content = response
        .text()
        .await
        .context("Failed to read response body")?;

    // Parse HTML
    let document = Html::parse_document(&html_content);

    // Try to find the most recent ranking table
    // SSBMRank pages typically have tables with class "wikitable"
    let table_selector = Selector::parse("table.wikitable").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let cell_selector = Selector::parse("td, th").unwrap();

    let mut top_50_players: Vec<String> = Vec::new();

    // Find the most recent ranking table
    // Tables are in chronological order, so we iterate in reverse to get the newest first
    let tables: Vec<_> = document.select(&table_selector).collect();

    for table in tables.iter().rev() {
        for (_row_idx, row) in table.select(&row_selector).enumerate() {
            let cells: Vec<_> = row.select(&cell_selector).collect();

            // Skip empty rows
            if cells.is_empty() {
                continue;
            }

            // First check if this row looks like it contains a rank number
            let first_cell_text = cells[0].text().collect::<String>().trim().to_string();

            // Skip header rows or rows without numeric rank
            if first_cell_text.parse::<i32>().is_err() {
                continue;
            }

            if cells.len() >= 2 {
                // Usually: Rank | Player | Character | etc.
                // Get the player name (typically in second column)
                let player_cell = &cells[1];

                // Try to extract player name from the cell
                // Player names are often in <a> tags
                // Note: Some cells have multiple links, skip empty ones
                let a_selector = Selector::parse("a").unwrap();

                let player_name = player_cell
                    .select(&a_selector)
                    .map(|link| link.text().collect::<String>().trim().to_string())
                    .find(|name| !name.is_empty())
                    .unwrap_or_else(|| player_cell.text().collect::<String>().trim().to_string());

                if !player_name.is_empty() && player_name != "-" && !player_name.contains("TBD") {
                    top_50_players.push(player_name);
                    if top_50_players.len() >= 50 {
                        break;
                    }
                }
            }
        }

        if top_50_players.len() >= 50 {
            break; // We found our top 50
        }

        // If we found some players but not enough, this might not be the right table
        if top_50_players.len() < 10 {
            top_50_players.clear();
        } else {
            
            break;
        }
    }

    if top_50_players.is_empty() {
        log_warn("rankings", "Could not parse rankings from Liquipedia");
        log_warn(
            "rankings",
            "Please manually update topPlayers.json or check the page structure",
        );
        return Ok(());
    }

    log_success("rankings", &format!("Parsed {} players", top_50_players.len()));

    // Use path relative to ssg/ directory (where cargo runs from)
    let top_players_path = "src/topPlayers.json";
    let existing_players: Vec<String> =
        serde_json::from_str(&fs::read_to_string(top_players_path)?)
            .unwrap_or_else(|_| Vec::new());

    // Create a set of top 50 for quick lookup o(1) i believe
    let top_50_set: HashSet<String> = top_50_players.iter().cloned().collect();

    // Keep legacy players i dont want to mess with the existing gods like leffen but happy to delete this
    let legacy_players: Vec<String> = existing_players
        .into_iter()
        .filter(|p| !top_50_set.contains(p))
        .collect();

    log_info(
        "rankings",
        &format!("Keeping {} legacy players", legacy_players.len()),
    );

    // Combine: top 50 first, then legacy players
    let mut final_players = top_50_players;
    final_players.extend(legacy_players);

    // Write to file
    let json_output = serde_json::to_string_pretty(&final_players)?;
    fs::write(top_players_path, json_output)?;

    log_success(
        "rankings",
        &format!("Updated topPlayers.json with {} total players", final_players.len()),
    );
    log_green("Run 'cargo run -- --generate' to update the GraphQL query");

    Ok(())
}
