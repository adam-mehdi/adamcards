#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

// use chrono::{NaiveDateTime, NaiveDate, DateTime, Utc, TimeZone};
use serde::{Serialize, Deserialize};
use tauri::{
  // api::path::resolve_path, 
  Manager, 
  // App,
  State
};
use std::{sync::{Mutex, Arc}, path::PathBuf};

// mod illustrations;


/*
 * Structs
 */

// want ReviewSession to be Vec<LeitnerBoxSystem>
#[allow(dead_code)]
struct ReviewSessionState {
  cards_arc: Arc<Mutex<Option<LeitnerBoxSystem>>>,
}
#[allow(dead_code)]
struct LeitnerBoxSystem {
  deck_name: String
}
// Path to folder with app data
#[allow(dead_code)]
struct AppDataDirState{
  path: Option<PathBuf>
}

// info about a deck
#[derive(Serialize, Deserialize)]
struct DeckEntry {
  id: usize,
  name: String,
  deadline_string: String,
}

// card with info about frontend
#[derive(Serialize, Deserialize)]
struct FrontendCard {
  id: usize,
  front: String,
  back: String,
}

/*
 * Run builder code
 */
fn main() {

  let review_session_state = ReviewSessionState {
    cards_arc: Arc::new(Mutex::new(None)),
  };


  tauri::Builder::default()
    .setup(|app| {
      let handle = app.handle();
      let data_dir = handle.path_resolver().app_data_dir().unwrap();

      let app_data_dir_state = AppDataDirState {
        path: Some(data_dir),
      };

      app.manage(app_data_dir_state);
      app.manage(review_session_state);

      Ok(())
    })
    // commands before greet implement actual mio2 functionality, everything else is for
    // illustrative purposes only and will be removed in the future.
    .invoke_handler(tauri::generate_handler![
      get_next_card, 
      post_review, 
      ])
    .run(tauri::generate_context!())
    
    .expect("error while running tauri application");

}


/*
 * MIO functions
 */

 #[allow(dead_code)]
#[tauri::command] 
fn get_next_card(_state: State<ReviewSessionState>, deck_id: usize) -> Result<i32, String> {
  // Get next card.
  // if there are no cards to review, return 'NoCardsToReview'
  println!("Get next card for deck {}", deck_id);

  // Rewrite this later to handle not being able to get the lock
  // let mut cards = lboxes.cards_arc.lock().unwrap();
  Ok(0)
}

// invoke('post_review', {deckId: usize, reviewScore: u8})

#[allow(dead_code)]
#[tauri::command]
fn post_review(_state: State<ReviewSessionState>, data_dir: State<AppDataDirState>,
  deck_id: usize, review_score: u8) -> Result<String, String> {

  println!("{} {}", review_score, deck_id);
  get_deck_path(data_dir, "test".to_owned());
  Ok("Success".to_string())
}

fn get_deck_path(data_dir: State<AppDataDirState>, deck_name: String) -> PathBuf {
  data_dir.path.as_ref().unwrap().join(deck_name)
}


