#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use chrono::{NaiveDateTime, NaiveDate, DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::sync::Mutex;

fn main() {
  let the_state = MyState {
    info: Mutex::new("Hello".to_string()),
  };

  tauri::Builder::default()
    .manage(the_state)
    .invoke_handler(tauri::generate_handler![greet, get_decks, pde, update_special_info])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

struct MyState {
  info: Mutex<String>,
}

#[tauri::command]
fn greet(state: tauri::State<MyState>, name: &str) -> String {
  format!("Hello, {}! From Rust!! The special message is {}", name, state.info.lock().unwrap())
}

#[tauri::command]
fn update_special_info(state: tauri::State<MyState>, special_info: &str) {
  *state.info.lock().unwrap() = special_info.to_string();
}

// #[tauri::command]
// fn my_custom_command(state: tauri::State<MyState>) {
//   assert_eq!(state.0 == "some state value", true);
// }

#[derive(Serialize, Deserialize)]
struct DeckEntry {
  id: usize,
  name: String,
  deadline_string: String,
}

#[tauri::command]
fn get_decks() -> Vec<DeckEntry> {

  let deadline = NaiveDate::from_ymd_opt(2014, 11, 28).unwrap().and_hms_opt(12, 0, 0).unwrap();

  let mut entries: Vec<DeckEntry> = Vec::new();

  entries.push(DeckEntry {
    id: 1,
    name: "German".to_string(),
    deadline_string: deadline.to_string(),
  });

  entries
}

#[derive(Serialize, Deserialize)]
struct ExperimentalDeckEntry {
  name: String,
  ndt: NaiveDateTime,
}

#[tauri::command]
async fn pde(deck_entry: ExperimentalDeckEntry)  {
  println!("Hello! From Rust {}, {}", deck_entry.name, deck_entry.ndt);

}