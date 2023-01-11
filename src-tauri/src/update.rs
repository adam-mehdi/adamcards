
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
    collections::HashMap,
    path::PathBuf, 
    io::{
      BufReader,
      prelude::*
    },
    fs::{ OpenOptions, File },
};
// use chrono::Local;
// use priority_queue::PriorityQueue as PQ;
// use rand::prelude::*;
// use std::fs::OpenOptions;


use crate::utils::{
    AppDataDirState,
    get_days_to_go, 
    read_num_boxes,
    get_root_path,
    get_child_decks,
    QuotasRecord,
    read_quotas_file,
    write_quotas_file,
    redistribute_quotas
};


// card with info about frontend
#[derive(Serialize, Deserialize, Debug)]
pub struct EditCard {
  id: usize,
  box_pos: usize,
  last_review: String,
  front: String,
  back: String,
  deck_name: String,
  is_created: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EntryChildren {
  cards: Vec<EditCard>,
  deck_names: Vec<String>
}




fn path2name(deck_path: &PathBuf, root: &PathBuf) -> String {
  deck_path.strip_prefix(&root).unwrap()
      .to_str().unwrap().to_owned()
}

#[tauri::command] 
pub fn read_decks(state: State<AppDataDirState>, entry: String) -> EntryChildren {
  let root = get_root_path(state);
  let deck_paths = get_child_decks(&root, &entry);
  let mut cards: Vec<EditCard> = Vec::new();


  let mut deck_names = Vec::new();
  for deck_path in deck_paths {
    let cards_path = deck_path.join("cards.csv");
    if !cards_path.exists() {
      panic!("cards file not in decks folder. problem: create cards on home
        Will create deck dir skeleton later");
    }
    let deck_name = path2name(&deck_path, &root);
    if !deck_names.contains(&deck_name) {
      deck_names.push(deck_name.to_string());
    }

    let file = File::open(cards_path).expect("file not found");
    let file = BufReader::new(file);

    for line in file.lines().skip(1) {
      let line = line.expect("failed to read line");
      let mut field_it = line.split(" >> ");

      let id = field_it.next().unwrap().trim().parse::<usize>().unwrap();
      let box_pos = field_it.next().unwrap().parse::<usize>().unwrap();
      let last_review = field_it.next().unwrap().to_string();
      let front = field_it.next().unwrap().to_owned();
      let back = field_it.next().unwrap().to_owned();


      cards.push( EditCard {
        id, 
        box_pos, 
        last_review, 
        front,
        back,
        deck_name: deck_name.to_string(),
        is_created: false
      });
    }
  }
  EntryChildren { cards, deck_names }
}

#[tauri::command] 
pub fn write_decks(state: State<AppDataDirState>, cards: Vec<EditCard>) {
  // need num_created for each deck!!
  let root = get_root_path(state);

  // Restructuring: Vec<EditCard> to {deck_name: card}
  let mut deck_map: HashMap<String, Vec<EditCard>> = HashMap::new();
  cards.into_iter().for_each(|card| {
    let deck = deck_map.entry(card.deck_name.to_string()).or_insert(vec![]);
    deck.push(card);
  });


  for (deck_name, deck_cards) in deck_map {
    let num_created = deck_cards.iter()
      .fold(0, |acc, x| acc + (x.is_created as i32));
    let deck_path = root.join(deck_name);
    write_cards_to_deck(&deck_path, deck_cards);
    update_quotas(&deck_path, num_created);
  }
}

/**
 * Writes cards in `deck_cards` into `cards.csv` file in the `deck_path` dir
 */
fn write_cards_to_deck(deck_path: &PathBuf, deck_cards: Vec<EditCard>) {

  let deck_path = deck_path.join("cards.csv");
  let mut file = OpenOptions::new().write(true).open(deck_path).unwrap();
  let mut contents = 
        "CardId >> BoxPosition >> LastReview >> Front >> Back\n".to_string();
  
  for card in deck_cards {
    contents.push_str(
      &format!( 
          "{} >> {} >> {} >> {} >> {}\n", 
          card.id, card.box_pos, card.last_review, &card.front, &card.back
        )
    );
  }

  file.write_all(contents.as_bytes())
        .expect("failed to save deck into buffer");
}



// computes quotas given `num_cards`, `num_boxes`, and `deck.days_to_go`, and 
// adds them to the nq and rq files on the quotas file
fn update_quotas(deck_path: &PathBuf, num_cards: i32) {
    // no quotas to update if no new cards
    if num_cards == 0 {
      return;
    }

    // get boxes from deck path config
    let num_boxes = read_num_boxes(&deck_path)
      .expect("failed to read num_boxes from config");
    
    // get days to go using deadline from config
    let days_to_go = get_days_to_go(&deck_path);

    // get existing quotas
    let quotas_path = deck_path.join("quotas.csv");
    assert!(quotas_path.is_file());
    let mut quotas = read_quotas_file(&quotas_path);

    // compute new quotas, with new and existing quotas both min sorted by dtg (nq, rq)
    let mut new_quotas = compute_quotas(num_cards, days_to_go, num_boxes);    

    // update quotas if there are existing quotas
    if quotas.len() != 0 {
      assert!(quotas.len() >= new_quotas.len(), "days to go cannot increase");

      for i in 0..new_quotas.len() {
          // update new quotas then review quotas
          new_quotas[i].nq += quotas[i].nq;
          new_quotas[i].rq += quotas[i].rq;
      }

      redistribute_quotas(&mut new_quotas);

      for i in 0..quotas.len() {
          // update new quotas then review quotas
          quotas[i].nq = new_quotas[i].nq;
          quotas[i].rq = new_quotas[i].rq;
      }
    } else {
      quotas = new_quotas;
    }
    
    // save new quotas
    write_quotas_file(&quotas, &quotas_path)


}





// computes quotas for `num_cards` given `days_to_go` and `num_boxes`
pub fn compute_quotas(num_cards: i32, days_to_go: i32, num_boxes: i32) -> Vec<QuotasRecord> {
    assert!(num_cards > 0);

    let n = num_cards;          // number of cards                                                           
    let t = days_to_go;         // days until deadline                                          
    let b = num_boxes;          // number of boxes
                                                                                
    let sum: i32 = (0..t).sum();
 
    // compute new card quota vector
    let mut nq: Vec<i32> = (0..t).rev().map(|x| x * n / sum).collect();
    let nq_sum = nq.iter().sum::<i32>();

    // enforce sum of NQ equals number of cards by adding remainder
    if let Some(first) = nq.get_mut(0) {
        *first += n - nq_sum;
    }
                                                                                
 
    // compute review card quota vector
    let mut rq: Vec<i32> = (0..t).map(|x| x * n * (b - 2) / sum).collect();                             
    let rq_sum = rq.iter().sum::<i32>();

    // enforce sum of RQ equals number of cards times number of bins minus 2
    if let Some(last) = rq.last_mut() {
        *last += (n * (b - 2)) - rq_sum;
    }
    // user reviews all cards the day of exam
    nq.push(0);
    rq.push(n);

    let mut quotas = Vec::new();
    for i in 0..nq.len() {
      let dtg = (nq.len() - 1 - i) as i32; // days to go
      quotas.push(QuotasRecord { dtg, nq: nq[i], rq: rq[i], nqp: 0, rqp: 0 });
    }

    // min sort by dtg: make index i correspond to days_to_go = i
    quotas.sort_by(|a, b| a.dtg.cmp(&b.dtg));
    assert!(quotas[0].dtg == 0);
    assert!(quotas.iter().fold(0, |acc, x| acc + x.nq) == n);
    assert!(quotas.iter().fold(0, |acc, x| acc + x.rq) == n * (b - 1));

    quotas
}






#[derive(Serialize, Deserialize, Debug)]
pub struct FieldPair {
  front: String,
  back: String,
}

#[tauri::command] 
pub fn parse_textfield(textfield: String) -> Vec<FieldPair> {
  let mut cards: Vec<FieldPair> = Vec::new();

  for line in textfield.lines() {
    // ensure this line contains a valid card
    if line.matches(">>").count() != 1 {
        continue;
    }
    let mut field_it = line.split(">>");

    let front = process_field(field_it.next().unwrap());
    let back = process_field(field_it.next().unwrap());
    cards.push(FieldPair { front, back });
  }

  cards
}

fn process_field(field: &str) -> String {
    let mut field = field.trim();
    let ch = field.chars().next().unwrap();
    if ch == '-' || ch == '*' {
      field =  field.strip_prefix(ch).unwrap().trim();
    }
    field.to_string()
}