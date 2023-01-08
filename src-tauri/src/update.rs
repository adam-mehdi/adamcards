
use tauri::{
  // api::path::resolve_path, 
//   Manager, 
  // App,
  State
};
use serde::{
    Serialize, 
    Deserialize
};

use std::fs::{
  read_dir,
  File,
};
use std::{ 
    path::PathBuf, 
    io::BufReader,
    io::prelude::*,
};
// use chrono::Local;
// use priority_queue::PriorityQueue as PQ;
// use rand::prelude::*;
// use std::fs::OpenOptions;

// use crate::mio0::{
//     mio_deck::DeckEntry, 
//     utils::{
//         redistribute_quotas, 
//         read_from_cfg,
//         read_quotas_file,
//         write_quotas_file,
//         path2string
//     }
// };

// mod mio0;


use crate::utils::{
    AppDataDirState
};
// card with info about frontend
#[derive(Serialize, Deserialize)]
pub struct Card {
  id: usize,
  box_pos: usize,
  last_review: String,
  front: String,
  back: String,
  deck_name: String,
}

// pub fn read_decks(state: State<AppDataDirState>, fs_name: String) -> Vec<Deck> {
// fs_name is the name of either a deck or folder
// }

// get immutable path to app data
fn get_root_path(data_dir: State<AppDataDirState>) -> PathBuf {
  // entry refers to the name of either a deck or a directory
  data_dir.path.as_ref().unwrap().join("decks")
}

// returns vector of deck directory paths if they are children of `entry`
fn get_child_decks(root: &PathBuf, entry: &String) -> Vec<PathBuf> {
  read_dir(root).expect("wrong root to appdata")
    .filter_map(Result::ok)
    .filter(|f| f.path().is_dir() && f.path().starts_with(entry))
    .map(|x| x.path())
    .collect::<Vec<PathBuf>>()
}

pub fn path2string(path: &PathBuf) -> String {
  path.clone().into_os_string().into_string().unwrap()
}


#[tauri::command] 
pub fn read_decks(state: State<AppDataDirState>, entry: String) -> Vec<Card> {
  let root = get_root_path(state);
  let deck_names = get_child_decks(&root, &entry);
  let mut cards: Vec<Card> = Vec::new();


  for deck_name in deck_names {
    let file = File::open(root.join("cards.csv")).expect("file not found");
    let file = BufReader::new(file);

    for line in file.lines().skip(1) {
      let line = line.expect("failed to read line");
      let mut field_it = line.split(" >> ");

      let id = field_it.next().unwrap().trim().parse::<usize>().unwrap();
      let box_pos = field_it.next().unwrap().parse::<usize>().unwrap();
      let last_review = field_it.next().unwrap().to_string();
      let front = field_it.next().unwrap().to_owned();
      let back = field_it.next().unwrap().to_owned();

      cards.push( Card {
        id, 
        box_pos, 
        last_review, 
        front, 
        back,
        deck_name: path2string(&deck_name)
      });
    }
  }
  cards
}