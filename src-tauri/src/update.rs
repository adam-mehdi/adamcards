
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
    iter::zip
};
// use chrono::Local;
// use priority_queue::PriorityQueue as PQ;
// use rand::prelude::*;
// use std::fs::OpenOptions;


use crate::utils::{
    AppDataDirState,
    read_from_config,
    get_days_to_go,
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
        deck_name: deck_name.to_string()
      });
    }
  }
  EntryChildren { cards, deck_names }
}

#[tauri::command] 
pub fn write_decks(state: State<AppDataDirState>, cards: Vec<Card>, num_created: i32) {
  eprintln!("NOT IMPLEMENTED. RETURNING");
  return;
  // let root = get_root_path(state);


  // Restructuring: Vec<Card> to {deck_name: card}
  // let mut deck_map: HashMap<String, Vec<Card>> = HashMap::new();
  // cards.into_iter().for_each(|card| {
  //   let deck = deck_map.entry(card.deck_name.to_string()).or_insert(vec![]);
  //   deck.push(card);
  // });

  // for (deck_name, deck_cards) in deck_map {
  //   let deck_path = root.join(deck_name);
  //   write_cards_to_deck(&deck_path, deck_cards);
  //   update_quotas(&deck_path, num_created)

    // LEFT OFF HERE

  // }
}

// writes cards in `deck_cards` into `cards.csv` file in the `deck_path` dir
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
// fn update_quotas(deck_path: &PathBuf, num_cards: i32) {
//     // get boxes from deck path config
//     let num_boxes = read_from_config(&deck_path, "num_boxes")
//       .unwrap_or_else(|| {eprintln!("COMPUTING DEFUALT NUM_BOXES"); 4});
    
//     let days_to_go = get_days_to_go(&deck_path);


//     let quotas_path = deck_path.join("quotas.csv");
//     assert!(quotas_path.is_file());
    

    
    

//     // get quotas for the new cards being put into the deck (nq, rq)
//     let mut new_quotas = compute_quotas(num_cards, days_to_go, num_boxes);    
//     // make index i correspond to days_to_go = i
//     new_quotas.reverse();


//     // form: [[dtg, nq, rq, nqp, rqp], ...]
//     let mut quotas = read_quotas_file(&quotas_path);

//     assert!(quotas.len() >= new_quotas.len(), "days to go must decrease");

//     for i in 0..new_quotas.len() {
//         // update new quotas then review quotas
//         new_quotas[i].0 += quotas[i][1];
//         new_quotas[i].1 += quotas[i][2];
//     }

//     redistribute_quotas(&mut new_quotas);

//     for i in 0..quotas.len() {
//         // update new quotas then review quotas
//         quotas[i][1] = new_quotas[i].0;
//         quotas[i][2] = new_quotas[i].1;
//     }
    
//     write_quotas_file(&quotas, &quotas_path)


// }


pub struct QuotasRecord {
  dtg: i32, // days_to_go
  nq: i32,  // new_quota
  rq: i32,  // review_quota
  nqp: i32, // new_quota_practiced
  rqp: i32  // review_quota_practiced
}


// returns lines of quotas in vector form [[dtg, nq, rq, nqp, rqp], ...]
pub fn read_quotas_file(quotas_path: &PathBuf) -> Vec<Vec<i32>> {

    let reader = OpenOptions::new().read(true).open(quotas_path).unwrap();
    let reader = BufReader::new(reader);

    let mut quotas: Vec<Vec<i32>> = Vec::new();
    for line in reader.lines().skip(1) {
        let line = line.expect("failed to read line");
        let msg = "quotas file is improperly formatted. must be csv.";
        quotas.push(
            line.split(',').map(|x| x.parse::<i32>().expect(msg)).collect()
        );
    }
    quotas
}

// write [[dtg, nq, rq, nqp, rqp], ...] to ./decks/quotas/deckname.csv
pub fn write_quotas_file(quotas: &Vec<Vec<i32>>, quotas_path: &PathBuf) {
  // compute (nq, rq) doubles for each day
  let buf = OpenOptions::new().write(true).open(quotas_path)
      .expect("failed to create quota file");
  let mut buf = BufWriter::new(buf);

  let header = "DaysToGo,NewQuota,ReviewQuota,NewPracticed,ReviewPracticed\n";
  buf.write_all(header.as_bytes()).expect("Unable to write data");

  for d in 0..quotas.len() {
      buf.write_fmt(format_args!(
          "{},{},{},{},{}\n",
           quotas[d][0], 
           quotas[d][1], 
           quotas[d][2], 
           quotas[d][3], 
           quotas[d][4]
          )
      ).expect("failed to write");
  }
}

// computes quotas for `num_cards` given `days_to_go` and `num_boxes`
pub fn compute_quotas(num_cards: i32, days_to_go: i32, num_boxes: i32) -> Vec<QuotasRecord> {

    let n = num_cards;          // number of cards                                                           
    let t = days_to_go;         // days until deadline                                          
    let b = num_boxes;          // number of boxes
                                                                                   
    let sum: i32 = (0..t).sum();
    
    // compute new card quota vector
    let mut nq: Vec<i32> = (0..t).rev().map(|x| x * n / sum).collect();
    let nq_sum = n;

    // enforce sum of NQ equals number of cards by adding remainder
    if let Some(first) = nq.get_mut(0) {
        *first += n % nq_sum;
    }
                                                                                   
    
    // compute review card quota vector
    let mut rq: Vec<i32> = (0..t).map(|x| x * n * (b - 2) / sum).collect();                             
    let rq_sum = n * (b - 2);

    // enforce sum of RQ equals number of cards times number of bins minus 2
    if let Some(last) = rq.last_mut() {
        *last += (n * (b - 2)) % rq_sum;
    }
    // user reviews all cards the day of exam
    nq.push(0);
    rq.push(n);

    let mut quotas = Vec::new();
    for i in 0..nq.len() {
      let dtg = (nq.len() - 1 - i) as i32; // days to go
      quotas.push(QuotasRecord { dtg, nq: nq[i], rq: rq[i], nqp: 0, rqp: 0 });
    }
    quotas
}


pub fn get_num_boxes(days_to_go: i64) -> i32 {
  let t = days_to_go as i32;
  // bins generated by recursive equation x_n = x_{n-1} + 2^n + 1 
  // applied 5 times to (2, 6)
let bins = vec![
      (0, 1), (2, 6), (7, 15), (16, 32), (33, 65), (66, 130), (131, 259)];
let mut i = 0;
  let mut found_bin = false;
for (a, b) in bins {
  if a <= t && t <= b {
          found_bin = true;
    break;
  }
  i += 1;
}
  assert!(found_bin, "deadline must be between 0 and 259 days in the future");
let num_boxes = 2 + i;
num_boxes
}

fn reverse_order<T: Copy>(coll: &mut Vec<T>) {
  let mut j = coll.len() - 1;
  let mut i = 0;
  while i <= j {
      let temp = coll[i];
      coll[i] = coll[j];
      coll[j] = temp;

      j -= 1;
      i += 1;
  }
}

// redistributes quotas, based on assuming new reviews are twice as expensive
// quotas has form (nq, rq); i days to go can correspond either to index
// quotas.len() - 1 - i or index i
pub fn redistribute_quotas(quotas: &mut Vec<(i32, i32)>){
  let reversed = quotas[0].0 != 0;
  if reversed {
      reverse_order(quotas);
  }

  //  a quantification of effort, score S := 2NQ + RQ
  let score_tot: i32 = compute_study_cost(&quotas).iter().sum();
  let score_avg = score_tot / quotas.len() as i32;

  // don't want to modify index 0, so temporarily give it avg score
  assert!(quotas[0].0 == 0, "day 0 is allotted a new card");
  let n_cards = quotas[0].1;
  quotas[0].1 = score_avg;

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
              if quotas[i].0 > 0 {
                  quotas[i].0 -= 1;
                  quotas[min_idx].0 += 1;
              }

              // give one RQ away to the min twice
              let scores = compute_study_cost(&quotas);
              let min_idx = argmin(&scores);
              if quotas[i].1 > 0 {
                  quotas[i].1 -= 1;
                  quotas[min_idx].1 += 1;
              }

              let scores = compute_study_cost(&quotas);
              let min_idx = argmin(&scores);
              if quotas[i].1 > 0 {
                  quotas[i].1 -= 1;
                  quotas[min_idx].1 += 1;
              }
          }
      }
  }

  // reset index 0
  quotas[0].1 = n_cards;

  if reversed {
      reverse_order(quotas);
  }
}

fn compute_study_cost(quotas: &Vec<(i32, i32)>) -> Vec<i32> {
    let scores: Vec<i32> = quotas
        .iter()
        .map(|x| 2*x.0 + x.1) //"new cards are twice as hard as review cards"
        .collect();
    scores
}

fn argmin(collection: &Vec<i32>) -> usize {
    let min = collection.iter().min().unwrap();
    let argmin = collection.iter().position(|x| x == min)
        .expect("failed to find argmin");
    argmin
} 