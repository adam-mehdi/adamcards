use std::{ 
    fs::File, 
    path::PathBuf, 
    cmp::{ Reverse, self }, 
    io::BufReader,
    io::prelude::*,
};
use chrono::Local;
use priority_queue::PriorityQueue as PQ;
use rand::prelude::*;
use std::fs::OpenOptions;

use crate::{
    mio_deck::DeckEntry, 
    utils::{
        redistribute_quotas, 
        read_from_cfg,
        read_quotas_file,
        write_quotas_file,
        path2string
    }
};

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct Card {
    id: u64,
    box_pos: usize,
    last_review: u64,
    front: String,
    back: String,
}

pub fn run_session(deck_name: String) {
    let deck = DeckEntry::new_from_name(&deck_name).unwrap();

    // build the lboxes
    let mut lboxes = init_lboxes(
        &path2string(&deck.deck_path_buf)
    );

    handle_missed_days(&deck); // update quotas file to account for missed days
    // new_left and review_left are the amount of cards left to review 
    // (distinct from quotas, which are amount total to review)
    let (mut new_left, mut review_left) = get_quotas(&deck);
    let num_boxes = read_from_cfg::<i32>(&deck_name, "num_boxes")
        .expect("'num_boxes' not in config");
    let (new_quota, review_quota) = (new_left, review_left);
    
    let mut review_num = 0;
    let mut is_cleaned = false;

    // ids of cards to be introduced in this session
    let ids_to_introduce = get_ids_to_introduce(
        &mut lboxes[0], new_left);
    
    while new_left > 0 || review_left > 0 {
        // draw_card will return reviewing_new
        let mut card = draw_card(&mut lboxes, review_num, 
            new_left, review_left, &ids_to_introduce).unwrap();

        println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
        println!("==================================");
        println!("New: {new_left} | Review: {review_left}");
        println!("==================================");

        // response to front of card
        println!("{}\n", card.front);
        if let None = get_response(true) {
            is_cleaned = cleanup(
                &mut lboxes, &deck, new_left, review_left, Some(card)
            );
            break;
        }

        // response to back of card
        println!("{}\n", card.back);
        match get_response(false) {
            // user is quitting
            None => {
                is_cleaned = cleanup(
                    &mut lboxes, &deck, new_left, review_left, Some(card)
                );
                break;
            },
            // user gives a valid response
            Some(score) => {
                // update left, not incrementing NQ if "-1" on new card
                if card.box_pos == 0 && score == 1{
                    new_left = (new_left as i32 - score) as u32;
                } else if card.box_pos > 0 {
                    review_left = (review_left as i32 - score) as u32;
                }

                // update box number
                let new_box_pos = get_new_pos(score, card.box_pos, num_boxes) as usize;
                card.box_pos = new_box_pos;

                // update last review
                let current_utc: u64 = Local::now().timestamp().try_into().unwrap();
                card.last_review = current_utc;

                // put card back into leitner box system
                let score = get_queue_score(current_utc);
                lboxes[new_box_pos].push(card, Reverse(score));
            }
        }
        review_num += 1;
    }

    println!("\n\n\n==================================");
    println!("You have completed {} new cards and {} reviewed cards today. Well done.",
        new_quota, review_quota);
    println!("==================================\n\n\n");
    if !is_cleaned {
        cleanup(&mut lboxes, &deck, new_left, review_left, None);
    }
}



// initializes leiter box system for specified deck
fn init_lboxes(deck_path: &String) -> Vec<PQ<Card, Reverse<u64>>> {
    // Hardcoded number of leitner boxes
    let num_boxes: u8 = 5; // Later we will infer the number of boxes

    // Instantiate the Vector of Priority Queues
    let mut lboxes = Vec::new();
    for _ in 0..num_boxes {
        lboxes.push(PQ::new());
    }

    // Prepare to Read
    let file = File::open(deck_path).expect("file not found");
    let reader = BufReader::new(file);

    // Read in each card in the deck file, skipping header line
    for line in reader.lines().skip(1) {
        let line = line.expect("failed to read line");
        let mut field_it = line.split(" >> ");

        let id = field_it.next().unwrap().trim().parse::<u64>().unwrap();
        let box_pos = field_it.next().unwrap().parse::<usize>().unwrap();
        assert!(box_pos < lboxes.len());

        let last_review = field_it.next().unwrap().to_string();
        let last_review = last_review.trim().parse::<u64>().unwrap();
        let queue_score = get_queue_score(last_review);

        let front = field_it.next().unwrap().to_owned();
        let back = field_it.next().unwrap().to_owned();

        // Push card to appropriate lbox
        lboxes[box_pos].push(
            Card {
                id,
                box_pos,
                last_review,
                front,
                back,
            },
            Reverse(queue_score)
        );
    }

    lboxes
}

fn get_ids_to_introduce(new_cards: &mut PQ<Card, Reverse<u64>>, num_to_introduce: u32)
    -> Vec<u64> {
    assert!(new_cards.len() > num_to_introduce as usize, "too few new cards");
    let mut ids: Vec<u64> = Vec::new();

    let mut card_cache: Vec<Card> = Vec::new();
    for _ in 0..num_to_introduce {
        let drawn_card = new_cards.pop().unwrap().0;
        ids.push(drawn_card.id);
        card_cache.push(drawn_card);
    }

    for card in card_cache {
        new_cards.push(card, Reverse(0));
    }

    ids
}

// reads quota file and returns (new_quota, review_quota) for current day
fn get_quotas(deck: &DeckEntry) -> (u32, u32) {
    assert!(deck.days_to_go >= 0);
    // get path to quota file at ./decks/quotas/deckname.txt
    let quota_path = deck.deck_path_buf
        .parent().unwrap().join("quotas/".to_string() + &deck.deck_name + "-quotas.csv");

    let mut contents = String::new();
    let mut file = File::open(quota_path)
        .expect("quotas file not found");
    file.read_to_string(&mut contents)
        .expect("failed to read quotas file");

    // read each line of quotas file, skipping header line
    for line in contents.lines().skip(1) {
        let mut quota_it = line.split(',');
        let dtg = quota_it.next().unwrap().parse::<i64>().expect("failed to parse");
        if dtg == deck.days_to_go {
            // total number of progressions allotted to the day
            let nq = quota_it.next().unwrap().parse::<u32>().expect("failed to parse");
            let rq = quota_it.next().unwrap().parse::<u32>().expect("failed to parse");
            // number of progressions already done
            let nqp = quota_it.next().unwrap().parse::<u32>().expect("failed to parse");
            let rqp = quota_it.next().unwrap().parse::<u32>().expect("failed to parse");
            return (nq - nqp, rq - rqp);
        }
    }
    panic!("failed to retrieve quota for days_to_go {}", deck.days_to_go)

}

// returns new box position of card according to `score`
fn get_new_pos(score: i32, box_pos: usize, num_boxes: i32) -> i32{
    let box_pos = box_pos as i32;
    let mut new_pos = box_pos + score;
    new_pos = cmp::min(num_boxes - 1, new_pos);
    new_pos = cmp::max(0, new_pos);
    new_pos
}

fn draw_card(lboxes: &mut Vec<PQ<Card, Reverse<u64>>>, review_num: i32, 
    new_quota: u32, review_quota: u32, new_ids: &Vec<u64>) -> Option<Card> {

    // introduce 5 new cards then 10 review cards, and if new or review quota 
    // are zero respectively, then introduce an old or new card respectively
    let is_new = (review_num % 15 < 5 && new_quota > 0) || review_quota == 0;
    
    if is_new {
        // if id is not in new_ids, then draw again
        let mut temp_cards = Vec::new();
        let next_card = loop {
            let card = lboxes[0].pop().expect("no new cards").0;
            if new_ids.contains(&card.id) {
                break card;
            }
            temp_cards.push(card);
        };

        while !temp_cards.is_empty() {
            lboxes[0].push(temp_cards.pop().unwrap(), Reverse(0));
        }

        return Some(next_card);
    }

    // number of boxes we are sampling from
    let num_boxes = lboxes.len() - 1;
    let mut bins: Vec<i32> = Vec::new();
    let mut acc = 2;
    for i in (0..num_boxes).rev() {
        acc += (i as i32) * (!lboxes[i].is_empty() as i32);
        bins.push(acc);
    }

    let mut rng = rand::thread_rng();
    let r = rng.gen_range(1..=bins[bins.len() - 1]);


    for i in 0..num_boxes {
        if r <= bins[i] {
            if let Some((card, _)) = lboxes[i].pop() {
                return Some(card);
            } else {
                panic!("no cards in chosen box: error with choosing box");
            }
        }
    }
    None
}

// executes graceful shutdown
fn cleanup(lboxes: &mut Vec<PQ<Card, Reverse<u64>>>, deck: &DeckEntry, 
    new_left: u32, review_left: u32, drawn_card: Option<Card>) -> bool {
    
    // put card back into queue
    if let Some(card) = drawn_card {
        let last_review = card.last_review;
        lboxes[card.box_pos].push(card, Reverse(last_review));
    }

    // write data to deck and quotas csvs, respectively
    cleanup_deck(lboxes, &deck.deck_path_buf);
    cleanup_quotas(deck, new_left, review_left);

    // graceful cleanup successful
    true
}

fn cleanup_deck(lboxes: &mut Vec<PQ<Card, Reverse<u64>>>, deckpath: &PathBuf) {
	// write cards into csv
	let mut file = OpenOptions::new().write(true).open(deckpath).unwrap();
    // let mut file = BufWriter::new(file);
    let mut contents = 
        "CardId >> BoxPosition >> LastReviewTime >> Front >> Back\n".to_string();
	for i in 0..lboxes.len() {
        while !lboxes[i].is_empty() {
            if let Some((card, _)) = &mut lboxes[i].pop() {
                contents.push_str(
                    &format!( 
                        "{} >> {} >> {} >> {} >> {}\n", 
                        card.id, card.box_pos, card.last_review, &card.front, &card.back
                    )
                );
            }
        }
    }
    file.write_all(contents.as_bytes())
        .expect("failed to save deck into buffer");
	// graceful cleanup complete
}


fn cleanup_quotas(deck: &DeckEntry, new_quota: u32, review_quota: u32) {
    let quotas_path = deck.get_quotas_path();

    let mut quotas_vec = read_quotas_file(&quotas_path);

    // new_quota is amount left to be practiced; get number of cards done
    let dtg = deck.days_to_go as usize;
    let new_done = quotas_vec[dtg][1] - new_quota as i32;
    let review_done = quotas_vec[dtg][2] - review_quota as i32;

    // add number of cards done to quotas
    quotas_vec[dtg][3] += new_done;
    quotas_vec[dtg][4] += review_done;

    write_quotas_file(&quotas_vec, &quotas_path);
}


// count days in past where quota is not fulfilled, add unfilfilled progressions
// to today's quota, and redistribute quotas to even out study cost over days
fn handle_missed_days(deck: &DeckEntry) {
    let mut quotas = read_quotas_file(&deck.get_quotas_path());

    let curr_idx = deck.days_to_go as usize;
    // return if no previous days
    if curr_idx == quotas.len()-1 {
        return;
    }

    let (mut nq_missed, mut rq_missed) = (0, 0);
    for i in (curr_idx-1)..quotas.len() {
        // count up number of progressions missed in the past
        nq_missed += quotas[i][1] - quotas[i][3];
        rq_missed += quotas[i][2] - quotas[i][4];

        // set past quota to the amount that was practiced
        quotas[i][1] = quotas[i][3];
        quotas[i][2] = quotas[i][4];
    }

    // return if no missed days
    if nq_missed == 0 && rq_missed == 0 {
        return;
    }

    // add missed cards to current day
    quotas[curr_idx][1] += nq_missed;
    quotas[curr_idx][2] += rq_missed;

    // redistribute quotas to even out amount of studying over days
    let mut new_quotas = 
       records2quotas(&quotas, curr_idx);
    redistribute_quotas(&mut new_quotas);

    for i in 0..=curr_idx {
        quotas[i][1] = new_quotas[i].0;
        quotas[i][2] = new_quotas[i].1;
    }

    // write redistributed quotas to file system
    write_quotas_file(&quotas, &deck.get_quotas_path());
}

// restructures matrix form of quotas csv to a vector of tuples up to `curr_idx` inclusive
fn records2quotas(records: &Vec<Vec<i32>>, curr_idx: usize) -> Vec<(i32, i32)> {
    let mut new_quotas = Vec::new();
    for i in 0..=curr_idx {
        new_quotas.push((records[i][1], records[i][2]));
    }
    new_quotas
}

// returns queue score (epoch time in seconds plus or minus 15 minutes)
fn get_queue_score(current_utc: u64) -> u64 {
    let mut range = rand::thread_rng();
    let noise = range.gen_range(-900..900); // +-15 min in secs
    let queue_score = ((current_utc as i64) + noise) as u64;
    queue_score
}

// returns user response: -1, 0, 1, or 10 (newline); or None
fn get_response(is_front: bool) -> Option<i32> {
    let mut re = String::new();
    std::io::stdin().read_line(&mut re).expect("Failed to read line");

    if re == "\n" && is_front {
        // 10 is the ascii character for \n
        Some(10)
    } else if re == "q\n" {
        None
    } else if re == "3\n" || re == "2\n" || re == "1\n" {
        Some(re.trim().parse::<i32>().unwrap() - 2)
    } else {
        println!("Reponse must be ENTER, 1, 2, 3, or q");
        get_response(is_front)
    }
}
