
use tauri;
use std::{ 
    sync::{
      Mutex, 
      Arc
    },
    path::PathBuf,
    collections::hash_map::DefaultHasher,
    hash::{ Hash, Hasher },
};
use serde::{
    Serialize, 
    Deserialize
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
 * Helpers
 */


 // id of a card is hash of its deck name, front, and back fields concatenated
#[tauri::command] 
pub fn calculate_hash(deck_name: String, front: String, back: String) -> u64 {
    let t = deck_name + &front + &back;
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}