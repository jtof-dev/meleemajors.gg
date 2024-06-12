use std::env;
use std::fs::File;
use serde_json::Value;

#[tokio::main]
async fn main() {
  let mut tournament_data: Vec<String> = Vec::new();
  let token = env::var("STARTGGAPI").unwrap();

  // iterate through tournaments in json array
  let tournaments = read_file("tournaments.json");

  let v: Value = serde_json::from_str(&tournaments).unwrap();

  match v {
    Value::Array(vec) => {
      for tournament in vec {
        let tournament_slug = tournament["start.gg-tournament-name"].as_str().unwrap();
        scrape_data(tournament_slug, &token).await;
      }
    },
    _ => panic!("root must be an array")
  }
}

/** Returns string contents of file with given path or panics otherwise */
fn read_file(path: &str) -> String {
  let file = File::open(path).unwrap();
  let file_contents = std::io::read_to_string(file).unwrap();
  return file_contents;
}

async fn scrape_data(tournament_slug: &str, token: &str) {
  // let tournament_info = ggapi::get_tournament_info(tournament_slug, token).await;

  let query = read_file("graphql/getTournamentName.gql");

  let vars = ggapi::Vars { id: ggapi::GGID::Int(0), slug: tournament_slug.to_string(), page: 1, per_page: 100 };

  let tournament_data = ggapi::execute_query(token, &query, vars).await;
  match tournament_data {
    ggapi::GGResponse::Data(data) => {
      println!("name: {}", data.tournament().name());
    },
    ggapi::GGResponse::Error(e) => panic!("error: {}", e),
  }
  // println!("name: {}", tournament_data);

}

fn generate_card() {

}

fn make_site() {

}