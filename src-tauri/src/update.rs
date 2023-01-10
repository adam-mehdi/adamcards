
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
    collections::HashMap,
    path::PathBuf, 
    io::{
      BufReader,
      BufWriter,
      prelude::*
    },
    fs::OpenOptions,
};
// use chrono::Local;
// use priority_queue::PriorityQueue as PQ;
// use rand::prelude::*;
// use std::fs::OpenOptions;


use crate::utils::{
    AppDataDirState,
    get_days_to_go, 
    read_num_boxes,
};
// card with info about frontend
#[derive(Serialize, Deserialize, Debug)]
pub struct Card {
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
  cards: Vec<Card>,
  deck_names: Vec<String>
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
fn get_child_decks(root: &PathBuf, entry: &str) -> Vec<PathBuf> {
  read_dir(root).expect("wrong root to appdata")
    .filter_map(Result::ok)
    .filter(|f| f.path().is_dir() && f.file_name().into_string().unwrap().starts_with(entry))
    .map(|x| x.path())
    .collect::<Vec<PathBuf>>()
}

// pub fn path2string(path: &PathBuf) -> String {
//   path.clone().into_os_string().into_string().unwrap()
// }


#[tauri::command] 
pub fn read_decks(state: State<AppDataDirState>, entry: String) -> EntryChildren {
  let root = get_root_path(state);
  let deck_paths = get_child_decks(&root, &entry);
  let mut cards: Vec<Card> = Vec::new();


  let mut deck_names = Vec::new();
  for deck_path in deck_paths {
    let cards_path = deck_path.join("cards.csv");
    if !cards_path.exists() {
      panic!("cards file not in decks folder. problem: create cards on home
        Will create deck dir skeleton later");
    }
    let deck_name = deck_path.strip_prefix(&root).unwrap()
      .to_str().unwrap().to_owned();
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


      cards.push( Card {
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
pub fn write_decks(state: State<AppDataDirState>, cards: Vec<Card>) {
  // need num_created for each deck!!
  let root = get_root_path(state);

  // Restructuring: Vec<Card> to {deck_name: card}
  let mut deck_map: HashMap<String, Vec<Card>> = HashMap::new();
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
fn write_cards_to_deck(deck_path: &PathBuf, deck_cards: Vec<Card>) {

  let deck_path = deck_path.join("cards.csv");
  let mut file = OpenOptions::new().write(true).open(deck_path).unwrap();
  let mut contents = 
        "CardId >> BoxPosition >> LastReviewTime >> Front >> Back\n".to_string();
  
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


#[derive(Debug)]
pub struct QuotasRecord {
  dtg: i32, // days_to_go
  nq: i32,  // new_quota
  rq: i32,  // review_quota
  nqp: i32, // new_quota_practiced
  rqp: i32  // review_quota_practiced
}


// returns records of quotas, sorted with ascending dtg 
pub fn read_quotas_file(quotas_path: &PathBuf) -> Vec<QuotasRecord> {

    let reader = OpenOptions::new().read(true).open(quotas_path).unwrap();
    let reader = BufReader::new(reader);

    let mut quotas: Vec<QuotasRecord> = Vec::new();
    for line in reader.lines().skip(1) {
        let line = line.expect("failed to read line");
        let mut line_it = line.split(',')
          .map(|x| x.parse::<i32>()
          .expect("quotas file is improperly formatted. must be csv."));
     
        quotas.push(
          QuotasRecord {
            dtg: line_it.next().expect("failed to read quotas csv"),
            nq: line_it.next().expect("failed to read quotas csv"),
            rq: line_it.next().expect("failed to read quotas csv"),
            nqp: line_it.next().expect("failed to read quotas csv"),
            rqp: line_it.next().expect("failed to read quotas csv"),
        });
     
    }
    quotas
}

// write [[dtg, nq, rq, nqp, rqp], ...] to ./decks/quotas/deckname.csv
pub fn write_quotas_file(quotas: &Vec<QuotasRecord>, quotas_path: &PathBuf) {
  // compute (nq, rq) doubles for each day
  let buf = OpenOptions::new().write(true).open(quotas_path)
      .expect("failed to create quota file");
  let mut buf = BufWriter::new(buf);

  let header = "DaysToGo,NewQuota,ReviewQuota,NewPracticed,ReviewPracticed\n";
  buf.write_all(header.as_bytes()).expect("Unable to write data");

  for d in 0..quotas.len() {
      buf.write_fmt(format_args!(
          "{},{},{},{},{}\n",
           quotas[d].dtg, 
           quotas[d].nq, 
           quotas[d].rq, 
           quotas[d].nqp, 
           quotas[d].rqp
          )
      ).expect("failed to write");
  }
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



// redistributes quotas, based on assuming new reviews are twice as expensive
// quotas has form (nq, rq); i days to go can correspond either to index
// quotas.len() - 1 - i or index i
pub fn redistribute_quotas(quotas: &mut Vec<QuotasRecord>){

  assert!(quotas[0].dtg == 0);
  assert!(quotas[0].nq == 0, "day 0 is allotted a new card");

  //  a quantification of effort, score S := 2NQ + RQ
  let score_tot: i32 = compute_study_cost(&quotas).iter().sum();
  let score_avg = score_tot / quotas.len() as i32;

  // don't want to modify index 0, so temporarily give it avg score
  let n_cards = quotas[0].rq;
  quotas[0].rq = score_avg;

  loop {
      let scores: Vec<i32> = compute_study_cost(&quotas);
      if !scores.iter().any(|&x| x - score_avg >= 4) {
          break;
      }

      // if any days have a score greater than 3
      for i in (0..quotas.len()).rev() {
          let scores: Vec<i32> = compute_study_cost(&quotas);
          if scores[i] - score_avg >= 4 {
              // give one NQ away to the min
              let min_idx = argmin(&scores);
              if quotas[i].nq > 0 {
                  quotas[i].nq -= 1;
                  quotas[min_idx].nq += 1;
              }

              // give one RQ away to the min twice
              let scores = compute_study_cost(&quotas);
              let min_idx = argmin(&scores);
              if quotas[i].rq > 0 {
                  quotas[i].rq -= 1;
                  quotas[min_idx].rq += 1;
              }

              let scores = compute_study_cost(&quotas);
              let min_idx = argmin(&scores);
              if quotas[i].rq > 0 {
                  quotas[i].rq -= 1;
                  quotas[min_idx].rq += 1;
              }
          }
      }
  }

  // reset index 0
  quotas[0].rq = n_cards;
}

fn compute_study_cost(quotas: &Vec<QuotasRecord>) -> Vec<i32> {
    let scores: Vec<i32> = quotas
        .iter()
        .map(|x| 2*x.nq + x.rq) //"new cards are twice as hard as review cards"
        .collect();
    scores
}

fn argmin(collection: &Vec<i32>) -> usize {
    let min = collection.iter().min().unwrap();
    let argmin = collection.iter().position(|x| x == min)
        .expect("failed to find argmin");
    argmin
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