use std::fs;

use crate::read_file;

pub fn main() {
    let mut get_featured_players_query =
        "query getFeaturedPlayers($slug_event: String!) {".to_owned();

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
    serde_json::from_str::<Vec<String>>(&top_players)
        .unwrap()
        .iter()
        .enumerate()
        .for_each(|(index, item)| {
            let new_query = get_featured_players_contents
                .replace("{{entrant-name}}", item)
                .replace("{{number}}", index.to_string().as_str());
            get_featured_players_query.push_str(&new_query);
        });

    get_featured_players_query.push_str(get_featured_players_footer);

    fs::write(
        "src/graphql/getFeaturedPlayers.gql",
        get_featured_players_query,
    )
    .unwrap();
}
