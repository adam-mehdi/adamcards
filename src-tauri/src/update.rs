
#![allow(unused_imports)]
#![allow(dead_code)]
use tauri::State;

use serde::{
    Serialize, 
    Deserialize
};

use std::{ 
    collections::HashMap,
    path::PathBuf, 
    io::{
      BufReader,
      BufWriter,
      prelude::*
    },
    fs::{ OpenOptions, File },
};

use crate::{ReviewSessionState, review};

use crate::utils::{
    AppDataDirState,
    get_days_to_go, 
    read_num_boxes,
    get_root_path,
    get_child_decks,
    QuotasRecord,
    write_quotas_file,
    // redistribute_quotas,
    path2fname,
    get_deck_idx,
    Card,
    MetaData,
    FrontendCard
};



#[derive(Serialize, Deserialize, Debug)]
pub struct EntryChildren {
  cards: Vec<Card>,
  deck_names: Vec<String>
}






// ===== Loading past cards into edit session =====

#[tauri::command] 
pub fn read_decks(data_dir: State<AppDataDirState>, state: State<ReviewSessionState>,
  entry: String) -> EntryChildren {

  let root = get_root_path(data_dir);
  let deck_paths = get_child_decks(&root, &entry);
  

  let mut cards: Vec<Card> = Vec::new();
  let mut deck_names = Vec::new();
  for deck_path in &deck_paths {
    let cards_path = deck_path.join("cards.csv");
    if !cards_path.exists() {
      panic!("cards file not in decks folder. problem: create cards on home
        Will create deck dir skeleton later");
    }
    let deck_name = &path2fname(&deck_path);
    if !deck_names.contains(deck_name) {
      deck_names.push(deck_name.to_string());
    }

    let file = File::open(cards_path).expect("file not found");
    let file = BufReader::new(file);

    for line in file.lines().skip(1) {
      let line = line.expect("failed to read line");
      let mut field_it = line.split(" >> ");

      
      let id = field_it.next().unwrap().trim().parse::<usize>()
        .expect("failed to parse id when reading decks");
      let box_pos = field_it.next().unwrap().parse::<usize>()
        .expect("failed to parse box pos when reading decks");
      let last_review = field_it.next().unwrap().to_string();
      let front = field_it.next().unwrap().to_owned();
      let back = field_it.next().unwrap().to_owned();
      let deck_name = deck_name.to_owned();

      let fcard = FrontendCard { id, front, back, deck_name };
      let md = MetaData { box_pos, last_review };
      cards.push( Card { fcard, md });

    }
  }

  // save deck paths to state for step of saving edited cards
  let deck_state = &mut *state.deck_paths.lock().unwrap();
  *deck_state = deck_paths; 

  EntryChildren { cards, deck_names }
}


// ===== Save edits: write cards to cards.csv and re-compute quotas =====

#[tauri::command] 
pub fn write_decks(state: State<ReviewSessionState>, cards: Vec<Card>) {

  // deck paths to deck names
  let deck_state = & *state.deck_paths.lock().unwrap();
  let deck_names: Vec<String> = (*deck_state).iter()
    .map(|x| path2fname(x)).collect();

  // using deck names as keys, group cards by deck affiliation
  let mut deck_map: HashMap<String, Vec<Card>> = HashMap::new();
  for deck_name in deck_names {
    deck_map.entry(deck_name).or_insert(Vec::new());
  }
  for card in cards {
    deck_map.get_mut(&card.fcard.deck_name)
      .expect("deck name not in list of decks. a deck was created during update")
      .push(card);
  }

  // write cards for each deck into fs
  for (deck_name, deck_cards) in deck_map {
    let deck_idx = get_deck_idx(&deck_name, deck_state).unwrap();
    let deck_path = &deck_state[deck_idx];

    update_quotas(deck_path, &deck_cards);
    write_cards_to_deck(deck_path, deck_cards);
  }
}

/**
 * Writes cards in `deck_cards` into `cards.csv` file in the `deck_path` dir
 */
fn write_cards_to_deck(deck_path: &PathBuf, deck_cards: Vec<Card>) {

  let deck_path = deck_path.join("cards.csv");

  // Note: should not be necessary but fixes non-writing if `deck_card` is empty
  let file = OpenOptions::new()
    .write(true)
    .truncate(true) 
    .open(deck_path)
    .expect("failed to open path to deck when saving");
  let mut file = BufWriter::new(file);

  let header = b"CardId >> BoxPosition >> LastReview >> Front >> Back\n";
  file.write_all(header).expect("failed to write header to cards.csv");
  
  for card in deck_cards {
    file.write_fmt(
      format_args!(
        "{} >> {} >> {} >> {} >> {}\n", 
        card.fcard.id, 
        card.md.box_pos, 
        card.md.last_review, 
        &card.fcard.front, 
        &card.fcard.back
      )
    ).expect("failed to write cards to cards.csv");
  }

}


/**
 * Computes quotas anew, discounts progresssions on cards, and redistributes 
 * quotas. Rewriting quotas is easiest in order to handle deleted cards.
 */
fn update_quotas(deck_path: &PathBuf, deck_cards: &Vec<Card>) {

    let quotas_path = deck_path.join("quotas.csv");
    assert!(quotas_path.is_file(), "problem with home: must create quotas");

    // write empty quotas file with just header if no cards in deck
    if deck_cards.len() == 0 {
      write_quotas_file(&Vec::new(), &quotas_path);
      return;
    }

    // get boxes from deck path config
    let num_boxes = read_num_boxes(&deck_path)
      .expect("failed to read num_boxes from config");
    
    // get days to go using deadline from config
    let days_to_go = get_days_to_go(&deck_path);

    // compute new quotas, with new and existing quotas both min sorted by dtg (nq, rq)
    let num_cards = deck_cards.len() as i32;
    let mut new_quotas = compute_quotas(num_cards, days_to_go, num_boxes);    

    discount_past_progressions(&mut new_quotas, &deck_cards);
    // redistribute_quotas(&mut new_quotas);

    // save new quotas
    write_quotas_file(&new_quotas, &quotas_path)


}

/**
 * Decreases values in `new_quotas` to account for past reviews, encoded in 
 * box positions of cards in `cards`
 * 
 * This function arises from the scheme where quotas are computed only based on
 * number of cards and days until deadline
 */
fn discount_past_progressions(new_quotas: &mut Vec<QuotasRecord>, cards: &Vec<Card>) {
  if new_quotas.len() == 1 {
    return;
  }

  // get number of cards which are advanced from the initial box
  let tot_new_advanced: i32 = cards.iter()
    .map(|x| (x.md.box_pos > 0) as i32).sum();

  // get number of times cards are advanced, not counting initial advance
  let tot_review_advanced = cards.iter()
    .map(|x| (x.md.box_pos as i32 - 1) * ((x.md.box_pos > 0) as i32))
    .sum::<i32>();


  let days = new_quotas.len() as i32 - 1;
  let new_per_day = tot_new_advanced / days;
  let mut remainder = tot_new_advanced - days * new_per_day;
  for dtg in 1..new_quotas.len() {
    new_quotas[dtg].nq -= new_per_day;

    // subtract remainder to day furthest from deadline
    if remainder > 0 {
      let sub_value = std::cmp::min(new_quotas[dtg].nq, remainder);
      new_quotas[dtg].nq -= sub_value;
      remainder -= sub_value;
    }
  }
  assert!(remainder == 0);

  let review_per_day = tot_review_advanced / days;

  let mut remainder = tot_review_advanced - days * review_per_day;
  for dtg in (1..new_quotas.len()).rev() {
    new_quotas[dtg].rq -= review_per_day;

    // subtract remainder to before deadline day
    if remainder > 0 {
      let sub_value = std::cmp::min(new_quotas[dtg].rq, remainder);
      new_quotas[dtg].rq -= sub_value;
      remainder -= sub_value;
    }
  }

  assert!(remainder == 0);

}



// computes quotas for `num_cards` given `days_to_go` and `num_boxes`
pub fn compute_quotas(num_cards: i32, days_to_go: i32, num_boxes: i32) 
  -> Vec<QuotasRecord> {
    assert!(num_cards > 0);

    let n = num_cards;          // number of cards                                                           
    let t = days_to_go;         // days until deadline                                          
    let b = num_boxes;          // number of boxes
                                                                                
    let mut sum: i32 = (0..t).sum();
    // avoid division by zero error for when t = 0, 1
    if sum == 0 {
      sum += 1;
    }
 
    // compute new card quota vector
    let mut nq: Vec<i32> = (0..t).rev().map(|x| x * n / sum).collect();
    let nq_sum = nq.iter().sum::<i32>();

    // enforce sum of NQ equals number of cards by adding remainder
    if let Some(first) = nq.get_mut(0) {
        *first += n - nq_sum;
        // no new cards on last day
        nq.push(0);
    }
                                                                                
 
    // compute review card quota vector
    let mut rq: Vec<i32> = (0..t).map(|x| x * n * (b - 2) / sum).collect();                             
    let rq_sum = rq.iter().sum::<i32>();

    // enforce sum of RQ equals number of cards times number of bins minus 2
    if let Some(last) = rq.last_mut() {
        *last += (n * (b - 2)) - rq_sum;
        // user reviews all cards the day of exam
        rq.push(n);
    }
    
    // review cards if days_to_go == 0
    if days_to_go == 0 {
      nq.push(n);
      rq.push(n * (b - 1));
    }

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





// ===== Helpers to create cards: parse textfield into cards =====

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
    if field.len() < 1 {
      return "".to_string();
    }
    let ch = field.chars().next().unwrap();
    if ch == '-' || ch == '*' {
      field =  field.strip_prefix(ch).unwrap().trim();
    }
    field.to_string()
}
