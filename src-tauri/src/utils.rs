
use tauri;
use std::{ 
    str::FromStr,
    sync::{
      Mutex, 
      Arc
    },
    io::{BufReader, BufRead},
    path::{ PathBuf },
    collections::hash_map::DefaultHasher,
    hash::{ Hash, Hasher },
    fmt::Debug,
    fs::OpenOptions
};
use serde::{
    Serialize, 
    Deserialize
};
use chrono::{ 
  DateTime,
  prelude::*,
};

  
  
/*
 * Structs
 */

// want ReviewSession to be Vec<LeitnerBoxSystem>
#[allow(dead_code)]
pub struct ReviewSessionState {
  pub cards_arc: Arc<Mutex<Option<LeitnerBoxSystem>>>,
}
#[allow(dead_code)]
pub struct LeitnerBoxSystem {
  pub deck_name: String
}
// Path to folder with app data
#[allow(dead_code)]
pub struct AppDataDirState{
  pub path: Option<PathBuf>
}

// info about a deck
#[derive(Serialize, Deserialize)]
struct DeckEntry {
  pub id: usize,
  pub name: String,
  pub deadline_string: String,
}

/*
 * Date Helpers
 */

 pub fn get_days_to_go(deck_path: &PathBuf) -> i32 {
    let deadline = read_from_config(&deck_path, "deadline")
      .expect("deadline not found in config");
    
    let datetime = deadline_to_datetime(deadline);
    let days_to_go = days_until_datetime(datetime) as i32;
    days_to_go 
 }

pub fn deadline_to_datetime(deadline_string: String) -> DateTime<FixedOffset> {
    if deadline_string.chars().count() == 25 {
        return DateTime::parse_from_rfc3339(&deadline_string)
            .expect("failed to parse datetime in the rfc3339 format");
    } else {
        panic!(
            "deadline string must have form or rfc3339 but got: {}", 
            deadline_string);
    }
}

pub fn days_until_datetime(datetime: DateTime<FixedOffset>) -> i64 {
    datetime.signed_duration_since(Local::now()).num_days()
}

 // id of a card is hash of its deck name, front, and back fields concatenated
#[tauri::command] 
pub fn calculate_hash(deck_name: String, front: String, back: String) -> u64 {
    let t = deck_name + &front + &back;
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}





/*
 * fs helpers 
 */

// given path to deck dir, returns value of field_name if found (otherwise None)
pub fn read_from_config<T: FromStr>(deck_path: &PathBuf, field_name: &str) -> Option<T>
    where <T as FromStr>::Err: Debug 
    {
    
    let config_path = deck_path.join("config.toml");

    let file = OpenOptions::new()
        .read(true)
        .open(config_path)
        .expect("failed to open deck cfg");
    let file = BufReader::new(file);

    for line in file.lines() {
        let line = line.expect("failed to read line from cfg");
        let mut it = line.split("=");
        let name = it.next().unwrap().trim();
        if name == field_name {
            let data = it.next()
                .expect("trying to retrieve empty field").trim();
            let data = T::from_str(data).expect("failed to extract value");
            return Some(data);
        }
    }
    None
  }