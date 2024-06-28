use std::fs;

use crate::read_file;

pub fn main() {
    // define graphql parts
    let mut get_featured_players_query = "query getFeaturedPlayers($slug_event: String!) {".to_owned();

    let get_featured_players_contents = r#"
    event(slug: $slug_event) {
        _{{number}}: entrants(query: { page: 0, filter: { name: "{{entrant-name}}" } }) {
            nodes {
                name
            }
        }
    }"#;
    let get_featured_players_footer = "\n}";

    let top_players = read_file("topPlayers.json");
    let mut number = 0;
    for player in serde_json::from_str::<Vec<String>>(&top_players).unwrap() {
        let new_query = get_featured_players_contents
            .replace("{{entrant-name}}", &player)
            .replace("{{number}}", &number.to_string());
        get_featured_players_query.push_str(&new_query);
        number += 1;
    }

    get_featured_players_query.push_str(get_featured_players_footer);

    fs::write("graphql/getFeaturedPlayers.gql", get_featured_players_query).unwrap();
}