#![allow(unused_imports)]
#![allow(dead_code)]
use tauri;
use tauri::State;

use std::{
    sync::{
        Mutex, 
        Arc
    },
    iter::Rev,
    ops::Range,
    cmp::{Reverse, max},
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
    MetaData,
    FrontendCard,
    AppDataDirState,
    QuotasRecord,
    get_root_path,
    get_child_decks, 
    read_num_boxes,
    string_to_datetime, 
    read_quotas_file,
    write_quotas_file, 
    get_days_to_go,
    redistribute_quotas, 
    path2fname, 
    path2string,
    get_deck_idx, 
    read_from_cfg, 
    append_val_cfg
};
use chrono::Local;

/**
 * Structs
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct Quotas {
    pub new_left: i32,
    pub review_left: i32,
    pub num_progressed: i32,
    pub deck_name: String
}

#[derive(Serialize, Deserialize, Debug)]
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
    pub dtg: Arc<Mutex<Vec<usize>>>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReviewSessionCard {
    pub card: Card,
    pub stack_before: String,
    pub stack_after: Option<String>,
    pub box_pos_delta:  Option<i32>,
    pub user_response: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct CardBuffer {
    data: Vec<ReviewSessionCard>,
    idx: i32,
}

 // ===== Initializing =====

/**
 * Sets up leiter box systems and quotas
 */
#[tauri::command] 
pub fn init_review_session(state: tauri::State<ReviewSessionState>, 
    data_dir: tauri::State<AppDataDirState>, entry_name: String) -> SummedQuotas {

    // get paths to decks that are children of state
    let root = get_root_path(data_dir);
    let deck_paths = get_child_decks(&root, &entry_name);

    // remove decks whose deadlines have passed
    // let mut valid_deck_paths = Vec::new();
    // for deck_path in deck_paths {
    //     let dtg = get_days_to_go(&deck_path);
    //     if dtg >= 0 {
    //         valid_deck_paths.push(deck_path);
    //     }
    // }
    // let deck_paths = valid_deck_paths;

    // get days to go at current time
    let mut dtg: Vec<usize> = Vec::new();
    for deck_path in &deck_paths {
        dtg.push(get_days_to_go(&deck_path) as usize);
    }
    let mut days_state = state.dtg.lock().unwrap();
    *days_state = dtg;

    // initialize quotas and leitner systems for each deck
    let quotas= get_todays_quotas(&*days_state, &deck_paths);
    let summed_quotas = sum_quotas(&quotas);
    let systems = init_leitner_systems(&deck_paths, &quotas);

    // (&summed_quotas);
    // put quotas and leitner systems on app state
    let mut quotas_state = state.quotas.lock().unwrap();
    *quotas_state = quotas;

    let mut systems_state= state.systems.lock().unwrap();
    *systems_state = systems;

    // save deck paths in state
    let mut path_state = state.deck_paths.lock().unwrap();
    *path_state = deck_paths;
    




    if summed_quotas.num_progressed < 0 {
        println!("num_progressed: {}", summed_quotas.num_progressed);
    }

    summed_quotas
}


pub fn get_todays_quotas(dtg: &Vec<usize>, deck_paths: &Vec<PathBuf>) -> Vec<Quotas> {


    // entry of deck_paths corresponds to entry in quotas_set
    let mut records_set= Vec::new();

    // days to go until deadline for each deck
    for deck_path in deck_paths {

        // read quotas file
        let quotas_path = deck_path.join("quotas.csv");
        let mut records = read_quotas_file(&quotas_path);

        // update quotas records to account for missed days
        handle_missed_days(&mut records, deck_path); 
        
        // redistribute quotas to even out amount of studying over days
        write_quotas_file(&records, &quotas_path);

        records_set.push(records);
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
            // get days to go, assume 0 days if deadline is passed
            let mut days = dtg[deck_idx];
            if days > 10446744073709551615  { days = 0; }
            let records = &records[days];

            // derive amount to review from quotas
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


/**
 * Count days in past where quota is not fulfilled, add unfilfilled progressions
 * to today's quota, and redistribute quotas to even out study cost over days
 */
fn handle_missed_days(quotas: &mut Vec<QuotasRecord>, deck_path: &PathBuf) {
    let mut curr_idx = get_days_to_go(deck_path) as usize;

    // if deadline has passed, act as if last day
    if (curr_idx as i32) < 0 {
        curr_idx = 0;
    }

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

    // add missed cards to today if it is the last day
    if curr_idx == 0 {
        quotas[curr_idx].nq += nq_missed;
        quotas[curr_idx].rq += rq_missed;
        return;
    }



    // add missed cards to days up to and including current day, without deadline day
    let num_days = curr_idx as i32;
    let new_per_day = nq_missed / num_days;
    let new_rmdr = nq_missed - new_per_day * num_days;
    let review_per_day = rq_missed / num_days;
    let review_rmdr = rq_missed - review_per_day * num_days;

    // distribute cards up to and including current day, skipping day of exam
    for dtg in 1..=curr_idx {

        // distribute burden for missed quotas on past days
        quotas[dtg].nq += new_per_day;
        quotas[dtg].rq += review_per_day;

        // add remainder to proper days (semi-arbirarily chosen)
        if dtg == 1 {
            quotas[dtg].rq += review_rmdr;
        } else if dtg == curr_idx {
            quotas[dtg].nq += new_rmdr;
        }

    }

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
    for _ in 0..num_boxes + 1 {
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
            Card { 
                fcard: FrontendCard {id, front, back, deck_name}, 
                md: MetaData { box_pos, last_review } 
            },
            Reverse(queue_score)
        );
    }

    assert!(lboxes.len() > 1, "num_boxes + 1 must be greater than 1");
    assert!(lboxes[0].len() >= quotas.new_left as usize, "likely failed to save on review");

    // get ids to introduce
    let mut new_ids = Vec::new();
    let mut card_cache: Vec<Card> = Vec::new();

    for _ in 0..quotas.new_left {
        let drawn_card = lboxes[0].pop().unwrap().0;
        new_ids.push(drawn_card.fcard.id);
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

fn sum_quotas(quotas: &Vec<Quotas>) -> SummedQuotas {
    let new_left = quotas.iter().fold(0, |acc, x| acc + x.new_left);
    let review_left = quotas.iter().fold(0, |acc, x| acc + x.review_left);
    let num_progressed = quotas.iter().fold(0, |acc, x| acc + x.num_progressed);
    SummedQuotas { new_left, review_left, num_progressed }
}




 // ===== Drawing next card =====

/**
 * Draws N cards from Leitner Box system in the app state `state`.
 * 
 * Returns DrawnItems, which contains a card if the quota today is not satisfied
 * and None if the review session is finished.
 */
#[tauri::command] 
pub fn draw_cards(state: State<ReviewSessionState>, num_cards: i32) -> Vec<Card> {

    let systems_state= &mut *state.systems.lock().unwrap();
    let quotas_state = &mut *state.quotas.lock().unwrap();
    let deck_state = &mut *state.deck_paths.lock().unwrap();

    
    let mut cards: Vec<Card> = Vec::new();
    let mut new_drawn = vec![0; deck_state.len()];
    let mut review_drawn =  vec![0; deck_state.len()];
    for _ in 0..num_cards {
        let card = draw_card(
            systems_state, 
            quotas_state, 
            deck_state, 
            &mut new_drawn, 
            &mut review_drawn
        );

        match card {
            // if quotas are fulfilled and no more cards to draw, stop drawing cards
            None => break,
            // otherwise, add to drawn cards
            Some(c) => cards.push(c)
        };

    }

    cards
}

fn draw_card(
    systems_state: &mut Vec<LeitnerBoxSystem>, 
    quotas_state: &mut Vec<Quotas>, 
    deck_state: &mut Vec<PathBuf>,
    new_drawn: &mut Vec<i32>,
    review_drawn: &mut Vec<i32>) -> Option<Card> {

    let num_new_drawn = &new_drawn.iter().sum::<i32>();
    let num_review_drawn = &review_drawn.iter().sum::<i32>();

    let is_new: Option<bool> = is_drawing_new(&quotas_state, num_new_drawn, num_review_drawn);
    if let None = is_new {
        return None;
    }

    let is_new = is_new.unwrap();
    let deck_idx = choose_deck(&quotas_state, is_new, new_drawn, review_drawn);

    // extract objects from state for easy reference
    let leitner_system = &mut systems_state[deck_idx];



    // draw new card: pop until finding a new card in `new_ids`
    let card =  if is_new { 
        new_drawn[deck_idx] += 1;
        Some(pop_new_card(leitner_system))
    } else {
        review_drawn[deck_idx] += 1;
        let is_last_day = get_days_to_go(&deck_state[deck_idx]) <= 0;
        pop_review_card(leitner_system, is_last_day)
    };

    card
}
    

// returns `is_new` if there are cards to review; otherwise None if finished session
fn is_drawing_new(
    quotas_state: &Vec<Quotas>, new_drawn: &i32, review_drawn: &i32
) -> Option<bool> {

    let num_progressed = quotas_state.iter()
        .fold(0, |acc, x| acc + x.num_progressed);
    let in_new_interval = num_progressed % 15 < 5;
    let new_exists = quotas_state.iter()
        .fold(0, |acc, x| acc + x.new_left) 
        - *new_drawn > 0;
    let review_exists = quotas_state.iter()
        .fold(0, |acc, x| acc + x.review_left) 
        - *review_drawn > 0;
    
    // completed review session
    if !new_exists && !review_exists {
        return None;
    }

    let is_new = (in_new_interval && new_exists) || !review_exists;
    Some(is_new)

}


// chooses deck to sample from
fn choose_deck(
    quotas: &Vec<Quotas>, 
    is_new: bool, 
    new_drawn: &mut Vec<i32>, 
    review_drawn: &mut Vec<i32>
) -> usize {

    // initial deck_idx is sampled 
    let mut range = rand::thread_rng();
    let mut deck_idx = range.gen_range(0..quotas.len()) as usize;


    // sample from a different deck if chosen deck has no new/review card quota
    let mut counter = 0;
    while (quotas[deck_idx].new_left - new_drawn[deck_idx] == 0 && is_new) ||
          (quotas[deck_idx].review_left - review_drawn[deck_idx] == 0 && !is_new) {

        // repeatedly sample until we get a valid card
        deck_idx = range.gen_range(0..quotas.len()) as usize;

        counter += 1;
        assert!(counter < 10000, "infinite loop probably initiated");
    }

    deck_idx
}

// draw a new card from the first box in chosen leitner system
fn pop_new_card(leitner_system: &mut LeitnerBoxSystem) -> Card {
    // if id is not in new_ids, then draw again
    let mut temp_cards = Vec::new();
    assert!(!leitner_system.lboxes[0].is_empty(), "no new cards to draw from");

    let new_card = loop {
        let card = leitner_system.lboxes[0].pop().expect("no new cards").0;
        if leitner_system.new_ids.contains(&card.fcard.id) {
            break card;
        }
        temp_cards.push(card);
    };

    while !temp_cards.is_empty() {
        leitner_system.lboxes[0].push(temp_cards.pop().unwrap(), Reverse(0));
    }

    new_card
}

fn pop_review_card(leitner_system: &mut LeitnerBoxSystem, is_last_day: bool) -> Option<Card> {
    // draw review card: 
    //    sample from boxes with probability linearly decreasing with box index
    let num_boxes = leitner_system.lboxes.len() - 1;

    // in Mathematica this is FoldList[Sum, 0, Range[num_boxes, 1]]
    let mut bins: Vec<usize> = Vec::new();
    let mut acc: usize = 0;
    let box_range = get_box_range(num_boxes, is_last_day);
    for (idx, b) in box_range.enumerate() {
        acc += b * (!leitner_system.lboxes[idx].is_empty() as usize);
        bins.push(acc);
    }

    // return if no cards to draw
    if bins.iter().sum::<usize>() == 0 {
        return None;
    }

    let mut rng = rand::thread_rng();
    let max = *bins.last().unwrap();
    let r = rng.gen_range(0..=max);


    // return box index i if the random number r falls between bins i-1 and i
    for i in 0..bins.len() {
        if r <= bins[i] {
            if let Some((card, _)) = leitner_system.lboxes[i].pop() {
                return Some(card);
            }         
        } 
    }
    panic!("no bins found; random number generated incorrectly");
}

fn get_box_range(num_boxes: usize, is_last_day: bool) -> Rev<Range<usize>> {
    // choose box to sample from, giving probability to last box as well
    if is_last_day {
        return (1..(num_boxes+1)).rev();
    } 
    // Note that the probability of sampling from the final box is 0; this is 
    // meant to make it impossible to graduate a card until the final day
    (0..num_boxes).rev()
}


 // ===== Handling user response =====



 /**
  * Updates backend state with buffer cards and their metadata. Pushes the reviewed
  * cards back into the Leitner box system.
  */
#[tauri::command] 
pub fn save_card_buffer(state: State<ReviewSessionState>, rcards: Vec<ReviewSessionCard>, 
    squotas: SummedQuotas) {

    // retrieve state
    let systems_state= &mut *state.systems.lock().unwrap();
    let quotas_state = &mut *state.quotas.lock().unwrap();
    let deck_state = & *state.deck_paths.lock().unwrap();

    // update quotas for each card in buffer
    for rcard in rcards {
        push_card_to_lbox(rcard, systems_state, quotas_state, deck_state, true)
    }

    let new_summed_quotas = sum_quotas(&quotas_state);

    assert!(new_summed_quotas.new_left == squotas.new_left, "{} {}", 
        new_summed_quotas.new_left, squotas.new_left);
    assert!(new_summed_quotas.review_left == squotas.review_left, "{} {}", 
        new_summed_quotas.review_left, squotas.review_left);
    assert!(new_summed_quotas.num_progressed == squotas.num_progressed, "{} {}",
        new_summed_quotas.num_progressed, squotas.num_progressed);

}

fn push_card_to_lbox(
    mut rcard: ReviewSessionCard, 
    systems_state: &mut Vec<LeitnerBoxSystem>,
    quotas_state: &mut Vec<Quotas>,
    deck_state: &Vec<PathBuf>,
    is_responded: bool) {

    let deck_idx = get_deck_idx(&rcard.card.fcard.deck_name, deck_state)
        .expect("failed to find deck name");

    if is_responded {
        update_quotas_on_response(&mut quotas_state[deck_idx], &rcard);
        update_review_time(&mut rcard.card.md.last_review);
    }

    let queue_score = get_queue_score(&rcard.card.md.last_review);
    systems_state[deck_idx].lboxes[rcard.card.md.box_pos]
        .push(rcard.card, Reverse(queue_score));

}

// updates quotas based on where cards drawn into buffer ended up
fn update_quotas_on_response(quotas: &mut Quotas, rcard: &ReviewSessionCard) {
    let stack_before = &rcard.stack_before;
    if let Some(stack_after) = rcard.stack_after.clone() {
        if stack_before == "new" && stack_after == "done" {
            quotas.new_left -= 1;
            quotas.num_progressed += 1;
        } else if stack_before == "review" && stack_after == "done" {
            quotas.review_left -= 1;
            quotas.num_progressed += 1;
        }
    } 
}

fn update_review_time(last_review: &mut String) {
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
pub fn cleanup(state: State<ReviewSessionState>, mut card_buffer: CardBuffer) {
    let systems_state= &mut *state.systems.lock().unwrap();
    let quotas_state = &mut *state.quotas.lock().unwrap();
    let deck_state = & *state.deck_paths.lock().unwrap();


    // put cards back into the queue, handling responses of those responded ones
    for card_idx in (0..card_buffer.data.len()).rev() {
        let is_responded = card_idx < card_buffer.idx as usize;
        let rcard = card_buffer.data.pop().unwrap();
        push_card_to_lbox(
            rcard, systems_state, quotas_state, deck_state, is_responded,
        );
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
                            card.fcard.id, 
                            card.md.box_pos,
                            card.md.last_review, 
                            &card.fcard.front, 
                            &card.fcard.back
                        )
                    ).expect("Failed to write deck data");
                }
            }
        }

    }

	// graceful cleanup complete for decks
}


// read an updated quotas_state into the file system
fn cleanup_quotas(quotas_state: &mut Vec<Quotas>, deck_state: &Vec<PathBuf>) {
    assert!(quotas_state.len() == deck_state.len());

    for deck_idx in 0..quotas_state.len() {
        let quotas_path = deck_state[deck_idx].join("quotas.csv");
        let mut quotas_vec = read_quotas_file(&quotas_path);

        let dtg = max(get_days_to_go(&deck_state[deck_idx]), 0);

        // new_quota is amount left to be practiced; get number of cards done
        for record_idx in 0..quotas_vec.len() {
            let mut quotas = &mut quotas_vec[record_idx];
            if quotas.dtg == dtg {
                // record what was practiced into quotas file
                let new_practiced = quotas.nq - quotas.nqp - quotas_state[deck_idx].new_left;
                quotas.nqp += new_practiced;

                let review_practiced = quotas.rq - quotas.rqp - quotas_state[deck_idx].review_left;
                quotas.rqp += review_practiced;
            }
        }

        write_quotas_file(&quotas_vec, &quotas_path);

    }

    // completed writing quotas
}
