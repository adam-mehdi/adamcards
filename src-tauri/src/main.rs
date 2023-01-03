#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use chrono::{NaiveDateTime, NaiveDate, DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::sync::{Mutex, Arc};

struct ReviewSessionState {
  cards_arc: Arc<Mutex<Vec<FrontendCard>>>,
}
fn main() {
  let mut cards = Vec::new(); 
  cards.push(
    FrontendCard{
      id: 0,
      front: "Front 0".into(),
      back: "Back 0".into(),
    }
  );
  cards.push(
    FrontendCard{
      id: 1,
      front: "Front 1".into(),
      back: "Back 1".into(),
    }
  );
   cards.push(
    FrontendCard{
      id: 2,
      front: "Front 2".into(),
      back: "Back 2".into(),
    }
  ); 
  cards.push(
    FrontendCard{
      id: 3,
      front: "Front 3".into(),
      back: "Back 3".into(),
    }
  );

  let review_session_state = ReviewSessionState {
    cards_arc: Arc::new(Mutex::new(cards)),
  };

  tauri::Builder::default()
    .manage(review_session_state)
    // commands before greet implement actual mio2 functionality, everything else is for
    // illustrative purposes only and will be removed in the future.
    .invoke_handler(tauri::generate_handler![get_next_card, create_card_from_csv, get_decks, post_review])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}



// #[tauri::command]
// fn greet(state: tauri::State<ReviewSessionState>, name: &str) -> String {
//   format!("Hello, {}! From Rust!! The special message is {}", name, state.info.lock().unwrap())
// }

// #[tauri::command]
// fn update_special_info(state: tauri::State<ReviewSessionState>, special_info: &str) {
//   *state.info.lock().unwrap() = special_info.to_string();
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
async fn pde(deck_entry: ExperimentalDeckEntry) -> String {
  println!("Hello! From Rust {}, {}", deck_entry.name, deck_entry.ndt);
  "TEST".to_string()

}

// THE FUNCTIONS FROM HERE ON ARE MIO


#[tauri::command]
async fn create_card_from_csv(card_path: &str) -> Result<String, String> {
  println!("create_card_from_csv received: {}", card_path);

  // Try to create the card from the CSV.
  // Return any meaningful errors to the frontend if it fails 
  // Otherwise let the user know it was a success

  // Ok("Card Created".to_string());
  Err("Not actually an error, create_card_from_csv not implemented".to_string())
}


#[derive(Serialize, Deserialize)]
struct FrontendCard {
  id: usize,
  front: String,
  back: String,
}

#[tauri::command]
fn get_next_card(state: tauri::State<ReviewSessionState>, deck_id: usize) -> Result<FrontendCard, String> {
  // Get next card.
  // if there are no cards to review, return 'NoCardsToReview'
  println!("Get next card for deck {}", deck_id);

  // Rewrite this later to handle not being able to get the lock
  let mut cards = state.cards_arc.lock().unwrap();
  
  match cards.pop() {
    Some(card) => Ok(card),
    None => Err("NoCardsToReview".to_string()),
  }
}

#[tauri::command]
fn post_review(state: tauri::State<ReviewSessionState>, deck_id: usize, review_score: u8) -> Result<String, String> {
  println!("Review Score: {}", review_score);
  Ok("Success".to_string())
}