#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::{
  Manager, 
  State
};
use std::sync::{
      Mutex, 
      Arc
    };

mod update;
use update::read_decks;

mod utils;
use utils::{
  ReviewSessionState,
  // LeitnerBoxSystem,
  AppDataDirState,
  calculate_hash
  // DeckEntry,
};

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
      // post_review, 
      read_decks,
      calculate_hash
      ])
    .run(tauri::generate_context!())
    
    .expect("error while running tauri application");

}


/*
 * MIO functions
 */

 // TODO: move to review.rs
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

// #[allow(dead_code)]
// #[tauri::command]
// fn post_review(_state: State<ReviewSessionState>, data_dir: State<AppDataDirState>,
//   deck_id: usize, review_score: u8) -> Result<String, String> {

//   println!("{} {}", review_score, deck_id);
//   get_deck_path(data_dir, "test".to_owned());
//   Ok("Success".to_string())
// }


