
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

use std::{ 
    fs::File, 
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
struct Card {
  id: usize,
  box_pos: usize,
  last_review: String,
  front: String,
  back: String,
}

#[derive(Serialize, Deserialize)]
pub struct Deck {
    cards: Vec<Card>,
    deck_name: String
}

// pub fn read_decks(state: State<AppDataDirState>, fs_name: String) -> Vec<Deck> {
// fs_name is the name of either a deck or folder
// }

fn get_deck_path(data_dir: State<AppDataDirState>, deck_name: &String) -> PathBuf {
  data_dir.path.as_ref().unwrap().join("decks").join(deck_name)
}

#[tauri::command] 
pub fn read_deck(state: State<AppDataDirState>, deck_name: String) -> Deck {
  let deck_path = get_deck_path(state, &deck_name);


  let file = File::open(deck_path.join("cards.csv"))
    .expect("file not found");
  let reader = BufReader::new(file);

  let mut cards: Vec<Card> = Vec::new();

  for line in reader.lines().skip(1) {
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
      back
    });
  }
  Deck { cards, deck_name }
}