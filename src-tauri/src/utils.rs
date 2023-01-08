
use std::{ 
    sync::{
      Mutex, 
      Arc
    },
    path::PathBuf
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

