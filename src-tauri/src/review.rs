
use tauri;
use tauri::State;

use std::{
    sync::{
        Mutex, 
        Arc
    },
    iter::Rev,
    ops::Range,
    cmp::Reverse,
    path::PathBuf,
    fs::{File, OpenOptions},
    io::{BufReader, BufRead, BufWriter, Write},
};

use serde::{
    Serialize, 
    Deserialize
};
use rand::Rng;
use priority_queue::PriorityQueue as PQ;

use crate::utils::{
    Card,
    AppDataDirState,
    get_root_path,
    get_child_decks, 
    read_num_boxes,
    string_to_datetime, 
    read_quotas_file,
    write_quotas_file, 
    get_days_to_go,
    redistribute_quotas, 
    path2fname, 
    get_deck_idx
};
use chrono::Local;

/**
 * Structs
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct Quotas {
    new_left: i32,
    review_left: i32,
    num_progressed: i32,
    deck_name: String
}

#[derive(Serialize, Deserialize)]
pub struct SummedQuotas {
    new_left: i32,
    review_left: i32,
    num_progressed: i32,
}

#[derive(Serialize, Deserialize)]
pub struct DrawnItems {
    card: Option<Card>,
    quotas: SummedQuotas
}


// leitner box of a deck (deck name is attribute of each card)
#[derive(Debug)]
pub struct LeitnerBoxSystem {
    lboxes: Vec<PQ<Card, Reverse<i64>>>,
    new_ids: Vec<usize>
}

pub struct ReviewSessionState {
    pub systems: Arc<Mutex<Vec<LeitnerBoxSystem>>>,
    pub quotas: Arc<Mutex<Vec<Quotas>>>,
    pub deck_paths: Arc<Mutex<Vec<PathBuf>>>,
}

 // ===== Initializing =====

/**
 * Sets up leiter box systems and quotas
 */
#[tauri::command] 
pub fn init_review_session(state: tauri::State<ReviewSessionState>, 
    data_dir: tauri::State<AppDataDirState>, entry_name: String) {

    // get paths to decks that are children of state
    let root = get_root_path(data_dir);
    let deck_paths = get_child_decks(&root, &entry_name);

    // initialize quotas and leitner systems for each deck
    let quotas= get_todays_quotas(&deck_paths);
    let systems = init_leitner_systems(&deck_paths, &quotas);

    // put quotas and leitner systems on app state
    let mut quotas_state = state.quotas.lock().unwrap();
    *quotas_state = quotas;

    let mut systems_state= state.systems.lock().unwrap();
    *systems_state = systems;

    // save deck paths in state
    let mut path_state = state.deck_paths.lock().unwrap();
    *path_state = deck_paths;

}



fn get_todays_quotas(deck_paths: &Vec<PathBuf>) -> Vec<Quotas> {

    // entry of deck_paths corresponds to entry in quotas_set
    let mut records_set= Vec::new();
    // days to go until deadline for each deck
    let mut dtg: Vec<usize> = Vec::new();
    for deck_path in deck_paths {
        // update quotas file to account for missed days
        handle_missed_days(&deck_path); 

        // read quotas file
        let quotas_path = deck_path.join("quotas.csv");
        records_set.push(read_quotas_file(&quotas_path));

        // read days to go
        dtg.push(get_days_to_go(&deck_path) as usize);
    }

    // extract today's quotas for each deck
    let mut quotas_set: Vec<Quotas> = Vec::new();
    for (deck_idx, records) in records_set.iter().enumerate() {

        let (new_left, review_left, num_progressed);
        let deck_name = path2fname(&deck_paths[deck_idx]);

        if records.len() == 0 {
            // quotas are zero if deck is empty
            new_left = 0; 
            review_left = 0; 
            num_progressed = 0;
        } else {
            // derive amount to review from quotas
            let records = &records[dtg[deck_idx]];
            new_left = records.nq - records.nqp;
            review_left = records.rq - records.rqp;
            num_progressed = records.nqp + records.rqp;
        }
        
        quotas_set.push(Quotas {
            new_left,
            review_left,
            num_progressed,
            deck_name 
        });
    }
    quotas_set

}


// count days in past where quota is not fulfilled, add unfilfilled progressions
// to today's quota, and redistribute quotas to even out study cost over days
fn handle_missed_days(deck_path: &PathBuf) {
    let quotas_path = deck_path.join("quotas.csv");
    let mut quotas = read_quotas_file(&quotas_path);
    let curr_idx = get_days_to_go(deck_path) as usize;

    // return if no previous days
    if curr_idx + 1 == quotas.len() {
        return;
    }

    let (mut nq_missed, mut rq_missed) = (0, 0);
    for i in (curr_idx + 1)..quotas.len() {
        // count up number of progressions missed in the past
        nq_missed += quotas[i].nq - quotas[i].nqp;
        rq_missed += quotas[i].rq - quotas[i].rqp;

        // set past quota to the amount that was practiced
        quotas[i].nq = quotas[i].nqp;
        quotas[i].rq = quotas[i].rqp;
    }

    // return if no missed days
    if nq_missed == 0 && rq_missed == 0 {
        return;
    }


    // add missed cards to current day
    quotas[curr_idx].nq += nq_missed;
    quotas[curr_idx].rq += rq_missed;

    // redistribute quotas to even out amount of studying over days
    // let mut new_quotas = records2quotas(&quotas, curr_idx);
    let new_quotas= &mut quotas[0..=curr_idx];
    redistribute_quotas(new_quotas);

    // write redistributed quotas to file system
    write_quotas_file(&quotas, &quotas_path);
}


fn init_leitner_systems(deck_paths: &Vec<PathBuf>, quotas: &Vec<Quotas>) -> Vec<LeitnerBoxSystem> {
    let mut systems = Vec::new();
    for i in 0..deck_paths.len() {
        systems.push(
            init_leitner_system(&deck_paths[i],&quotas[i])
        );
    }
    systems
}

fn init_leitner_system(deck_path: &PathBuf, quotas: &Quotas) -> LeitnerBoxSystem {
    
    let num_boxes = read_num_boxes(&deck_path)
        .expect("failed to read num_boxes from config"); 

    // Instantiate the Vector of Priority Queues
    let mut lboxes = Vec::new();
    for _ in 0..num_boxes {
        lboxes.push(PQ::new());
    }

    // Prepare to Read
    let cards_path = deck_path.join("cards.csv");
    let file = File::open(cards_path).expect("file not found");
    let reader = BufReader::new(file);

    // Read in each card in the deck file, skipping header line
    for line in reader.lines().skip(1) {
        let line = line.expect("failed to read line");
        let mut field_it = line.split(" >> ");

        let id = field_it.next().unwrap().trim().parse::<usize>().unwrap();
        let box_pos = field_it.next().unwrap().parse::<usize>().unwrap();
        assert!(box_pos < lboxes.len());

        let last_review = field_it.next().unwrap().trim().to_string();
        let queue_score = get_queue_score(&last_review);

        let front = field_it.next().unwrap().to_owned();
        let back = field_it.next().unwrap().to_owned();
        let deck_name = path2fname(deck_path);

        // Push card to appropriate lbox
        lboxes[box_pos].push(
            Card { id, box_pos, last_review, front, back, deck_name },
            Reverse(queue_score)
        );
    }

    assert!(lboxes.len() > 0, "num_boxes must be greater than 0");

    // get ids to introduce
    let mut new_ids = Vec::new();
    let mut card_cache: Vec<Card> = Vec::new();
    for _ in 0..quotas.new_left {
        let drawn_card = lboxes[0].pop().unwrap().0;
        new_ids.push(drawn_card.id);
        card_cache.push(drawn_card);
    }

    while !card_cache.is_empty() {
        lboxes[0].push(
            card_cache.pop().unwrap(), 
            Reverse(0)
        );
    }

    LeitnerBoxSystem { lboxes, new_ids}
}

// returns queue score (epoch time in seconds plus or minus 15 minutes)
fn get_queue_score(last_review: &str) -> i64 {
    let dt: i64;
    if last_review == "None" {
        dt = 0;
    } else {
        dt = string_to_datetime(last_review).timestamp();
    }
    let mut range = rand::thread_rng();
    let noise = range.gen_range(-900..900); // +-15 min in secs
    let queue_score = dt + noise;
    queue_score
}




 // ===== Drawing next card =====

/**
 * Draws card from Leitner Box system in the app state `state`.
 * 
 * Returns DrawnItems, which contains a card if the quota today is not satisfied
 * and None if the review session is finished.
 */
#[tauri::command] 
pub fn draw_card(state: tauri::State<ReviewSessionState>) -> DrawnItems {

    let systems_state= &mut *state.systems.lock().unwrap();
    let quotas_state = &mut *state.quotas.lock().unwrap();
    let deck_state = &mut *state.deck_paths.lock().unwrap();

    // Determine if drawing new or review: 5 new then 10 review
    let num_progressed = quotas_state.iter()
        .fold(0, |acc, x| acc + x.num_progressed);
    let is_new: Option<bool> = is_drawing_new(&quotas_state);

    // if quotas are fulfilled
    if let None = is_new {
        // card == None triggers cleanup on frontend for finishing review session
        return DrawnItems { 
            card: None, 
            quotas: SummedQuotas { new_left: 0, review_left: 0, num_progressed }
        };
    };
    

    let is_new = is_new.unwrap();
    let deck_idx = choose_deck(&quotas_state, is_new);

    // extract objects from state for easy reference
    let quotas = quotas_state.get_mut(deck_idx).unwrap();
    let leitner_system = systems_state.get_mut(deck_idx).unwrap();

    
    // decrement number of cards left
    if is_new {
        quotas.new_left -= 1
    } else {
        quotas.review_left -= 1
    }

    // compute summed quotas for the frontend's display
    let new_left = quotas_state.iter().fold(0, |acc, x| acc + x.new_left);
    let review_left = quotas_state.iter().fold(0, |acc, x| acc + x.review_left);
    let summed_quotas = SummedQuotas { new_left, review_left, num_progressed };

    // draw new card: pop until finding a new card in `new_ids`
    let card =  if is_new { 
        pop_new_card(leitner_system)
    } else {
        // Warning: never reaches last day with +1 on get_days_to_go_naive
        let is_last_day = get_days_to_go(&deck_state[deck_idx]) == 0;
        pop_review_card(leitner_system, is_last_day)
    };

    return DrawnItems {
        card: Some(card),
        quotas: summed_quotas
    };
}

    

fn is_drawing_new(quotas_state: &Vec<Quotas>) -> Option<bool> {
    let num_progressed = quotas_state.iter()
        .fold(0, |acc, x| acc + x.num_progressed);
    let in_new_interval = num_progressed % 15 < 5;
    let new_exists = quotas_state.iter()
        .fold(0, |acc, x| acc + x.new_left) > 0;
    let review_exists = quotas_state.iter()
        .fold(0, |acc, x| acc + x.review_left) > 0;
    
    // completed review session
    if !new_exists && !review_exists {
        return None;
    }

    let is_new = (in_new_interval && new_exists) || !review_exists;
    Some(is_new)

}


fn choose_deck(quotas: &Vec<Quotas>, is_new: bool) -> usize {

    let mut range = rand::thread_rng();
    let mut deck_idx = range.gen_range(0..quotas.len()) as usize;

    // repeatedly sample until we get a valid card
    let mut counter = 0;
    while (quotas[deck_idx].new_left == 0 && is_new) ||
          (quotas[deck_idx].review_left == 0 && !is_new) {

        deck_idx = range.gen_range(0..quotas.len()) as usize;

        counter += 1;
        assert!(counter < 10000, "infinite loop probably initiated");
    }

    deck_idx
}

fn pop_new_card(leitner_system: &mut LeitnerBoxSystem) -> Card {
    // if id is not in new_ids, then draw again
    let mut temp_cards = Vec::new();
    let new_card = loop {
        let card = leitner_system.lboxes[0].pop().expect("no new cards").0;
        if leitner_system.new_ids.contains(&card.id) {
            break card;
        }
        temp_cards.push(card);
    };

    while !temp_cards.is_empty() {
        leitner_system.lboxes[0].push(temp_cards.pop().unwrap(), Reverse(0));
    }

    new_card
}

fn pop_review_card(leitner_system: &mut LeitnerBoxSystem, is_last_day: bool) -> Card {
    // draw review card: 
    //    sample from boxes with probability linearly decreasing with box index
    let num_boxes = leitner_system.lboxes.len() - 1;

    // in Mathematica this is FoldList[Sum, 0, Range[num_boxes, 1]]
    let mut bins: Vec<usize> = Vec::new();
    let mut acc: usize = 0;
    let day_range = get_day_range(num_boxes, is_last_day);
    for i in day_range {
        acc += i * (!leitner_system.lboxes[i].is_empty() as usize);
        bins.push(acc);
    }

    let mut rng = rand::thread_rng();
    let max = *bins.last().unwrap();
    let r = rng.gen_range(0..=max);

    // return box index i if the random number r falls between bins i-1 and i
    for i in 0..num_boxes {
        if r <= bins[i] {
            if let Some((card, _)) = leitner_system.lboxes[i].pop() {
                return card;
            } else {
                panic!("no cards in chosen box: error with choosing box");
            }
        }
    }
    panic!("no bins found; random number generated incorrectly");
}

fn get_day_range(num_boxes: usize, is_last_day: bool) -> Rev<Range<usize>> {
    if is_last_day {
        return (1..(num_boxes + 1)).rev();
    } 
    // Note that the probability of sampling from the final box is 0; this is 
    // meant to make it impossible to graduate a card until the final day
    (0..num_boxes).rev()
}


 // ===== Handling user response =====

#[tauri::command] 
pub fn handle_response(state: State<ReviewSessionState>, mut card: Card, response: i32) {

    let systems_state= &mut *state.systems.lock().unwrap();
    let quotas_state = &mut *state.quotas.lock().unwrap();
    let deck_state = & *state.deck_paths.lock().unwrap();

    let deck_idx = get_deck_idx(&card.deck_name, deck_state)
        .expect("deck not found in deck_state");

    let quotas = &mut quotas_state[deck_idx];
    let system = &mut systems_state[deck_idx];

    // response of 1, 2, or 3, is now a score of -1, 0, 1
    let score = response - 2;

    // update quotas and box position based on score and set last review to now
    update_quotas_on_response(score, quotas, &card);
    update_pos_on_response(score, &mut card.box_pos, system.lboxes.len());
    update_last_review(&mut card.last_review);

    // put card back into leitner box system
    let queue_score = get_queue_score(&card.last_review);
    system.lboxes[card.box_pos].push(card, Reverse(queue_score));

}


// put card back into proper stack
fn update_quotas_on_response(score: i32, quotas: &mut Quotas, card: &Card) {
    // pregress if correct on new card
    // score == 1 ==> one to progressed and one 
    if card.box_pos == 0 {
        if score == 1 {
            quotas.num_progressed += score;
        } else {
            quotas.new_left += 1;
        }
    }
    // progress or retract on review card
    else if card.box_pos > 0 {
        quotas.review_left -= score;
        quotas.num_progressed += score;
    }
}

// returns new box position of card according to `score`
fn update_pos_on_response(score: i32, box_pos: &mut usize, num_boxes: usize) {
    let mut new_pos = *box_pos as i32 + score;
    if new_pos < 0 {
        new_pos = 0;
    } else if new_pos > num_boxes as i32 - 1{
        new_pos = num_boxes as i32 - 1;
    }
    *box_pos = new_pos as usize;
}

fn update_last_review(last_review: &mut String) {
    // update last review to YYYY-MM-DDThh:mm:ss+ZZ:ZZ format
    *last_review = Local::now()
    // SecondsFormat records just seconds instead of the default microseconds
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
}



 // ===== Saving review session =====

 /**
  * Saves review session data to cards and quotas files. Puts `drawn_card` back  
  * into proper leitner system if the session is not completed, in which case 
  * `card` is None.
  */
#[tauri::command] 
pub fn cleanup(state: tauri::State<ReviewSessionState>, drawn_card: Option<Card>) {
    let systems_state= &mut *state.systems.lock().unwrap();
    let quotas_state = &mut *state.quotas.lock().unwrap();
    let deck_state = & *state.deck_paths.lock().unwrap();
    
    // put card back into queue
    if let Some(card) = drawn_card {
        let queue_score = get_queue_score(&card.last_review);
        let i = get_deck_idx(&card.deck_name, deck_state)
            .expect("did not find deck index in deck state");
        systems_state[i].lboxes[card.box_pos].push(card, Reverse(queue_score));
    }

    // write data to deck and quotas csvs, respectively
    cleanup_decks(systems_state, &deck_state);
    cleanup_quotas(quotas_state, &deck_state);

    // graceful cleanup successful
}

fn cleanup_decks(systems_state: &mut Vec<LeitnerBoxSystem>, deck_state: &Vec<PathBuf>) {

    assert!(systems_state.len() == deck_state.len());
    for deck_idx in 0..systems_state.len() {
        let deck_path = deck_state[deck_idx].join("cards.csv");

        // write cards into csv
        let file = OpenOptions::new().truncate(true).write(true).open(deck_path)
            .expect("failed to open path to deck when saving");
        let mut file = BufWriter::new(file);


        let header = b"CardId >> BoxPosition >> LastReview >> Front >> Back\n";
        file.write_all(header).expect("Failed to write deck data");
        let lboxes = &mut systems_state[deck_idx].lboxes;

        for box_idx in 0..lboxes.len() {
            while !lboxes[box_idx].is_empty() {
                if let Some((card, _)) = &mut lboxes[box_idx].pop() {
                    file.write_fmt(format_args!( 
                            "{} >> {} >> {} >> {} >> {}\n", 
                            card.id, card.box_pos, card.last_review, &card.front, &card.back
                        )
                    ).expect("Failed to write deck data");
                }
            }
        }

    }

	// graceful cleanup complete for decks
}


fn cleanup_quotas(quotas_state: &mut Vec<Quotas>, deck_state: &Vec<PathBuf>) {
    assert!(quotas_state.len() == deck_state.len());

    for deck_idx in 0..quotas_state.len() {
        let quotas_path = deck_state[deck_idx].join("quotas.csv");
        let mut quotas_vec = read_quotas_file(&quotas_path);

        let dtg = get_days_to_go(&deck_state[deck_idx]);

        // new_quota is amount left to be practiced; get number of cards done
        for record_idx in 0..quotas_vec.len() {
            let mut quotas = &mut quotas_vec[record_idx];
            if quotas.dtg == dtg {
                let new_practicable = quotas.nq - quotas.nqp;
                let new_practiced = 
                    new_practicable - quotas_state[deck_idx].new_left;
                quotas.nqp += new_practiced;

                let review_practicable = quotas.rq - quotas.rqp;
                let review_practiced = 
                    review_practicable - quotas_state[deck_idx].review_left;
                quotas.rqp += review_practiced;
            }
        }

        write_quotas_file(&quotas_vec, &quotas_path);

    }

    // completed writing quotas
}
