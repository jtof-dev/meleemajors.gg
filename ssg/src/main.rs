use std::env;

#[tokio::main]
async fn main() {
  // Parse the string of data into serde_json::Value.

  // todo: get `data` from the file
  //   let data = r#"
  //     [
  //       {
  //         "start.gg": "https://www.start.gg/tournament/tipped-off-15-connected-1/details",
  //         "schedule": "",
  //         "featured-players": [
  //           "TBD", "TBD", "TBD", "TBD", "TBD", "TBD", "TBD", "TBD"
  //         ],
  //         "stream": ""
  //       },
  //       {
  //         "start.gg": "https://www.start.gg/tournament/ceo-2024-6/details",
  //         "schedule": "",
  //         "featured-players": [
  //           "TBD", "TBD", "TBD", "TBD", "TBD", "TBD", "TBD", "TBD"
  //         ],
  //         "stream": ""
  //       }
  //     ]"#;

  //   let v: Value = serde_json::from_str(data).unwrap();
  //   println!("{}", v);

  //   match v {
  //     Value::Array(vec) => {
  //       for tournament in vec {
  //         println!("startgg: {}", tournament["start.gg"]);
  //       }
  //     },
  //     _ => panic!("root must be an array")
  //   }

  // Needs an async executor, not included here for brevity.
  // Explore libraries like `tokio` for async execution.
  let slug = "tipped-off-15-connected-1";

  let token = env::var("STARTGGAPI").unwrap();
  println!("API key: {}", token);

  let tournament_info = ggapi::get_tournament_info(slug, &token).await;
  match tournament_info {
    ggapi::GGResponse::Data(data) => {
      println!("asdfasdf: {}", data.tournament().name());
    },
    _ => {
      println!("No tournament found.");
    }
  }
}

// current_user: Option<Box<GGUser>>
// entrant: Option<Box<GGEntrant>>
// event: Option<Box<GGEvent>>
// participant: Option<Box<GGParticipant>>
// phase: Option<Box<GGPhase>>
// phase_group: Option<Box<GGPhaseGroup>>
// player: Option<Box<GGPlayer>>
// set: Option<Box<GGSet>>
// tournament: Option<Box<GGTournament>>
// tournaments: Option<Box<GGTournamentConnection>>
// user: Option<Box<GGUser>>
// videogame: Option<Box<GGVideogame>>
// videogames: Option<Box<GGVideogameConnection>>