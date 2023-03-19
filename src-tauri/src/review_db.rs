
// #![allow(unused_imports)]
// #![allow(unused_variables)]
// #![allow(dead_code)]

use diesel::update;
use diesel::prelude::*;

use tauri;
use tauri::State;

use crate::home_db::DatabaseState;
use crate::models::Card;

use chrono::Local;

use serde::{
    Serialize, 
    Deserialize
};
use std::{
    sync::{
        Mutex, 
        Arc
    }
};
use rand::distributions::Distribution;
use rand::{Rng, distributions::WeightedIndex};


use crate::edit_db::get_days_to_go;


#[derive(Clone)]
pub struct UserResponse {
    pub card_id: i32,
    pub box_pos_delta: Option<i32>,
    pub user_answer: String,
    pub stack_after: Option<String>,
    pub stack_before: String,
    pub deck_id: i32
}
pub struct ReviewSessionState {
    pub response_stack: Arc<Mutex<Vec<UserResponse>>>,
    pub undo_response_stack: Arc<Mutex<Vec<UserResponse>>>,
    pub curr_card: Arc<Mutex<Option<UserResponse>>>,
    pub new_ids: Arc<Mutex<Vec<i32>>>,
    pub days_to_go: Arc<Mutex<Option<i32>>>,
    pub deadline_id: Arc<Mutex<Option<i32>>>
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Quota {
    pub new_left: i32,
    pub review_left: i32,
    pub num_progressed: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReviewCard {
    pub stack_before: String,
    pub deck_name: String,
    pub card: Card
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CardResults {
    pub stack_after: Option<String>,
    pub user_answer: String,
    pub card: ReviewCard
}



#[tauri::command] 
pub fn init_review_session(
    state: State<DatabaseState>, 
    review_state: State<ReviewSessionState>, 
    deadline_id: i32) -> Quota 
{ 
    use crate::schema::{cards, deckitems};

    let conn= &mut *state.conn.lock().unwrap();
    let days_to_go= &mut *review_state.days_to_go.lock().unwrap();
    let id= &mut *review_state.deadline_id.lock().unwrap();
    let new_ids= &mut *review_state.new_ids.lock().unwrap();

    // record deadline id
    *id = Some(deadline_id);
    
    // record days_to_go
    let dtg = get_days_to_go(conn, deadline_id);
    *days_to_go = Some(dtg);

    let deck_ids = get_deck_ids(conn, deadline_id);
    
    for deck_id in &deck_ids {
        handle_missed_days(conn, *deck_id, &dtg);
    }

    // specify new_ids
    let quotas = get_deadline_quotas(conn, &deck_ids, &dtg);

    // select which new cards to memorize today
    for i in 0..quotas.len() {
        if quotas[i].new_left == 0 {
            continue;
        }

        let new_ids_deck: Vec<i32> = cards::table
            .inner_join(deckitems::table.on(cards::id.eq(deckitems::item_id)))
            .filter(deckitems::deck_id.eq(deck_ids[i]).and(cards::box_position.eq(0)))
            .select(cards::id)
            .limit(quotas[i].new_left as i64)
            .get_results::<i32>(conn)
            .expect("failed to get new ids");

        new_ids.extend_from_slice(&new_ids_deck);
    }
    

    get_deadline_summed_quota(conn, &deck_ids, &dtg)


}


/**
 * Count days in past where quota is not fulfilled, add unfilfilled progressions
 * to today's quota, and redistribute quotas to even out study cost over days
 */
#[derive(Debug)]
pub struct QuotaTempRecord {
  pub dtg: i32, // days_to_go
  pub nq: i32,  // new_quota
  pub rq: i32,  // review_quota
  pub nqp: i32, // new_quota_practiced
  pub rqp: i32  // review_quota_practiced
}

pub fn handle_missed_days(conn: &mut SqliteConnection, deck_id: i32, days_to_go: &i32) {
    use crate::schema::quotas;
    let mut curr_idx = *days_to_go as usize;

    // read quotas 
    let quota_tuples = quotas::table
        .filter(quotas::id.eq(deck_id).and(quotas::days_to_go.lt(days_to_go + 1)))
        .select((quotas::days_to_go, quotas::new_assigned, quotas::review_assigned, quotas::review_practiced, quotas::new_practiced))
        .get_results::<(i32, i32, i32, i32, i32)>(conn)
        .expect("failed to get quotas");

    let mut quotas = Vec::new();
    for t in quota_tuples {
        quotas.push(
            QuotaTempRecord { dtg: t.0, nq: t.1, rq: t.2, nqp: t.3, rqp: t.4 }
        )
    }

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

    // update quotas database with new quotas
    for quota in quotas {
        update(quotas::table)
            .filter(quotas::id.eq(deck_id).and(quotas::days_to_go.eq(quota.dtg)))
            .set((quotas::new_assigned.eq(quota.nq), quotas::review_assigned.eq(quota.rq)))
            .execute(conn)
            .expect("failed to update quota");
    }

}



fn get_deck_ids(conn: &mut SqliteConnection, deadline_id: i32) -> Vec<i32> {
    use crate::schema::parents;
    parents::table
        .filter(parents::parent_id.eq(deadline_id))
        .select(parents::child_id)
        .get_results::<i32>(conn)
        .expect("failed to get deck ids")
}

fn get_deadline_quotas(conn: &mut SqliteConnection, deck_ids: &Vec<i32>, days_to_go: &i32) -> Vec<Quota> {
    use crate::schema::quotas;

    let mut quotas = Vec::new();
    for deck_id in deck_ids {
        let quota = quotas::table
            .filter(quotas::id.eq(deck_id).and(quotas::days_to_go.eq(days_to_go)))
            .select((quotas::new_assigned, quotas::review_assigned, quotas::new_practiced, quotas::review_practiced))
            .get_result::<(i32, i32, i32, i32)>(conn)
            .optional()
            .expect("failed to retrieve quota records");

        match quota {
            Some(q) => quotas.push(
                    Quota {
                        new_left: q.0,
                        review_left: q.1,
                        num_progressed: q.2 + q.3
                    }
                ),
            None => quotas.push( 
                Quota { 
                    new_left: 0, 
                    review_left: 0, 
                    num_progressed: 0 
                }
            )
        }

    }

    quotas
}

fn get_deadline_summed_quota(conn: &mut SqliteConnection, deck_ids: &Vec<i32>, days_to_go: &i32) -> Quota {
    use crate::schema::quotas;

    let quotas = quotas::table
        .filter(quotas::id.eq_any(deck_ids).and(quotas::days_to_go.eq(days_to_go)))
        .select((quotas::new_assigned, quotas::review_assigned, quotas::new_practiced, quotas::review_practiced))
        .get_results::<(i32, i32, i32, i32)>(conn)
        .expect("failed to retrieve quota records");

    let mut summed_quota = Quota { new_left: 0, review_left: 0, num_progressed: 0 };
    for quota in quotas {
        summed_quota.new_left += quota.0;
        summed_quota.review_left += quota.1;
        summed_quota.num_progressed += quota.2;
    }

    summed_quota

}

// fn get_deck_quota(conn: &mut SqliteConnection, deck_id: i32, days_to_go: &i32) -> Quota {
//     use crate::schema::quotas;

//     let quotas = quotas::table
//         .filter(quotas::id.eq(deck_id).and(quotas::days_to_go.eq(days_to_go)))
//         .select((quotas::new_assigned, quotas::review_assigned, quotas::new_practiced, quotas::review_practiced))
//         .get_results::<(i32, i32, i32, i32)>(conn)
//         .expect("failed to retrieve quota records");

//     let mut summed_quota = Quota { new_left: 0, review_left: 0, num_progressed: 0 };
//     for quota in quotas {
//         summed_quota.new_left += quota.0;
//         summed_quota.review_left += quota.1;
//         summed_quota.num_progressed += quota.2;
//     }

//     summed_quota

// }

#[tauri::command] 
pub fn get_next_card(state: State<DatabaseState>, review_state: State<ReviewSessionState>) -> Option<ReviewCard> { 

    let conn= &mut *state.conn.lock().unwrap();
    let deadline_id= &*review_state.deadline_id.lock().unwrap();
    let new_ids = &*review_state.new_ids.lock().unwrap();
    let days_to_go= &*review_state.days_to_go.lock().unwrap();
    let curr_card = &mut *review_state.curr_card.lock().unwrap();

    // get deck ids and quotas
    let deck_ids = get_deck_ids(conn, deadline_id.unwrap());
    let quotas = get_deadline_quotas(conn, &deck_ids, &days_to_go.unwrap());

    // determine if drawing new card
    let is_new: Option<bool> = is_drawing_new(&quotas);
    if let None = is_new {
        return None;
    }
    let is_new = is_new.unwrap();

    // choose deck
    let deck_idx = choose_deck(&quotas, is_new);
    let deck_id = deck_ids[deck_idx];

    // choose box
    let new_card;
    if is_new {
        new_card = pop_new_card(conn, new_ids, deck_id);

    } else {
        new_card = pop_review_card(conn, deck_id);
    }

    *curr_card = Some(UserResponse {
        card_id: new_card.card.id.clone(),
        box_pos_delta: None,
        user_answer: String::from(""),
        stack_before: new_card.stack_before.clone(),
        stack_after: None,
        deck_id
    });

    Some(new_card)

 }

fn pop_new_card(conn: &mut SqliteConnection, new_ids: &Vec<i32>, deck_id: i32) -> ReviewCard { 
    use crate::schema::{cards, deckitems, entries};
    use diesel::prelude::*;

    // get the first card in the chosen deck whose id is in new_ids
    let new_card = cards::table
        .inner_join(deckitems::table.on(cards::id.eq(deckitems::item_id)))
        .filter(cards::id.eq_any(new_ids).and(deckitems::deck_id.eq(deck_id)))
        .select((cards::id, cards::front, cards::back))
        .order(cards::queue_score.asc())
        // .order(cards::queue_score.asc().nulls_first()) // nulls_first means nulls come first with ascending order
        .first::<(i32, String, String)>(conn)
        .expect("failed to pop new card");

    let deck_name = entries::table
        .filter(entries::id.eq(deck_id))
        .select(entries::name)
        .get_result::<String>(conn)
        .expect("failed to get deck name");


    ReviewCard {
        deck_name,
        stack_before: String::from("new"),
        card: Card {
            id: new_card.0,
            front: new_card.1,
            back: new_card.2
        }
    }

    

}

fn pop_review_card(conn: &mut SqliteConnection, deck_id: i32) -> ReviewCard { 
    use crate::schema::{cards, deckitems, entries};


    let card_ids = deckitems::table
        .filter(deckitems::deck_id.eq(deck_id))
        .select(deckitems::item_id)
        .get_results::<i32>(conn)
        .expect("failed to get deck ids");

    // in terms of SQL
    // let box_counts = diesel::sql_query("SELECT box_position, COUNT ( * ) FROM quotas GROUP BY box_position")
    //     .load(conn)
    //     .expect("failed to load box counts");

    let box_counts = cards::table
        .select((cards::box_position, diesel::dsl::sql::<diesel::sql_types::BigInt>("count(*)"))) // https://github.com/diesel-rs/diesel/issues/1781#issuecomment-633174958
        .group_by(cards::box_position)
        .get_results::<(i32, i64)>(conn)
        .expect("failed to get distribution of boxes");


    // choose box with probability weighted by number of cards in the box
    let box_pos = choose_weighted_index(&box_counts);

    let card = cards::table
        .filter(cards::id.eq_any(card_ids).and(cards::box_position.eq(box_pos)))
        .select((cards::id, cards::front, cards::back))
        .order(cards::queue_score.asc())
        .first::<(i32, String, String)>(conn)
        .expect("failed to order cards");

    let deck_name = entries::table
        .filter(entries::id.eq(deck_id))
        .select(entries::name)
        .get_result::<String>(conn)
        .expect("failed to get deck name");

    ReviewCard {
        stack_before: String::from("review"),
        deck_name: deck_name,
        card: Card {
            id: card.0,
            front: card.1,
            back:card.2
        }
    }

}

// returns box position
fn choose_weighted_index(pos_weights: &Vec<(i32, i64)>) -> i32 {
    let mut v = Vec::new();
    for w in pos_weights {
        v.push(w.1 as i32);
    }

    let n = v.len();
    let weights = (0..n).map(|i| 1.0 / (i as f32 + 1.0)).collect::<Vec<_>>();
    let dist = WeightedIndex::new(&weights).unwrap();
    let mut rng = rand::thread_rng();
    let idx = dist.sample(&mut rng);

    let box_pos = pos_weights[idx].0;
    box_pos
}

// returns `is_new` if there are cards to review; otherwise None if finished session
fn is_drawing_new(quotas_state: &Vec<Quota>) -> Option<bool> {

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

// chooses deck to sample from
fn choose_deck(quotas: &Vec<Quota>, is_new: bool) -> usize {

    // initial deck_idx is sampled 
    let mut range = rand::thread_rng();

    let mut deck_idx;

    // sample from a different deck if chosen deck has no new/review card quota
    let mut counter = 0;
    loop {
        // repeatedly sample until we get a valid card
        deck_idx = range.gen_range(0..quotas.len()) as usize;

        if quotas[deck_idx].new_left > 0 && is_new || quotas[deck_idx].review_left > 0 && !is_new {
            break;
        }

        counter += 1;
        assert!(counter < 10000, "infinite loop probably initiated");
    }

    deck_idx
}


// returns stack_after; score is -1, 0, or 1
#[tauri::command] 
pub fn record_response(
    state: State<DatabaseState>, 
    review_state: State<ReviewSessionState>,
    score: i32, 
    user_answer: String, 
    card: ReviewCard
) -> String {

    use crate::schema::{cards, quotas, deckitems};
    let conn= &mut *state.conn.lock().unwrap();
    let days_to_go = *review_state.days_to_go.lock().unwrap();
    let response_stack = &mut *review_state.response_stack.lock().unwrap();
    let curr_card = &mut *review_state.curr_card.lock().unwrap();

    // update card box_pos
    let box_pos_delta = get_box_pos_delta(conn, score, &card.card.id);

    // update card's contents and box pos, returning new box pos
    update(cards::table)
        .filter(cards::id.eq(card.card.id))
        .set((cards::box_position.eq(cards::box_position + box_pos_delta), cards::front.eq(card.card.front), cards::back.eq(card.card.back), cards::queue_score.eq(get_queue_score())))
        .execute(conn)
        .expect("failed to update card box pos");


    // get deck id
    let deck_id = deckitems::table
        .filter(deckitems::item_id.eq(card.card.id))
        .select(deckitems::deck_id)
        .get_result::<i32>(conn)
        .expect("failed to get deck id");

    // update quota
    if card.stack_before == "new" {
        update(quotas::table)
            .filter(quotas::id.eq(deck_id).and(quotas::days_to_go.eq(days_to_go.unwrap())))
            .set((quotas::new_assigned.eq(quotas::new_assigned - box_pos_delta), quotas::new_practiced.eq(quotas::new_practiced + box_pos_delta)))
            .execute(conn)
            .expect("failed to update new quota");

    } else {
        update(quotas::table)
            .filter(quotas::id.eq(deck_id).and(quotas::days_to_go.eq(days_to_go.unwrap())))
            .set((quotas::review_assigned.eq(quotas::review_assigned - box_pos_delta), quotas::review_practiced.eq(quotas::review_practiced + box_pos_delta)))
            .execute(conn)
            .expect("failed to update review quota");
    }

    // append card to responseStack
    let stack_after;
    if score == 1 {
        stack_after = "done";
    } else {
        stack_after = &card.stack_before;
    }
    let stack_after = String::from(stack_after);

    // return stack after
    *curr_card = None;
    let response = UserResponse {
        card_id: card.card.id,
        user_answer,
        stack_before: card.stack_before.clone(),
        stack_after: Some(stack_after.clone()),
        box_pos_delta: Some(box_pos_delta),
        deck_id
    };
    response_stack.push(response);

    stack_after
    
}

fn get_box_pos_delta(conn: &mut SqliteConnection, score: i32, card_id: &i32) -> i32 {
    use crate::schema::cards;

    let box_pos = cards::table
        .filter(cards::id.eq(card_id))
        .select(cards::box_position)
        .get_result::<i32>(conn)
        .expect("failed to get box pos");

    let mut box_pos_delta = 0;
    if score == -1 && box_pos > 1 {
        box_pos_delta = -1;
    } else if score == 1 {
        box_pos_delta = 1;
    }
    box_pos_delta
}

// returns queue score (epoch time in seconds plus or minus 15 minutes)
fn get_queue_score() -> Option<i32> {
    let dt = Local::now().timestamp();
    let mut range = rand::thread_rng();
    let noise = range.gen_range(-30..30); // +-30 secs
    let queue_score = dt + noise;
    Some(queue_score as i32)
}


// returns previous card; String indicates stack_after
#[tauri::command] 
pub fn get_last_card(state: State<DatabaseState>, review_state: State<ReviewSessionState>) -> Option<CardResults> { 
    use crate::schema::{cards, quotas, entries};
    let conn= &mut *state.conn.lock().unwrap();
    let days_to_go = *review_state.days_to_go.lock().unwrap();
    let response_stack = &mut *review_state.response_stack.lock().unwrap();
    let curr_card = &mut *review_state.curr_card.lock().unwrap();
    let undo_response_stack = &mut *review_state.undo_response_stack.lock().unwrap();

    if let None = curr_card {
        return None;
    }

    match response_stack.pop() {
        None => return None,
        Some(response) => {
            undo_response_stack.push(curr_card.clone().unwrap());

            // update quotas
            if &response.stack_before == "new" {
                update(quotas::table)
                    .filter(quotas::id.eq(response.deck_id).and(quotas::days_to_go.eq(days_to_go.unwrap())))
                    .set((quotas::new_assigned.eq(quotas::new_assigned + response.box_pos_delta.unwrap()), quotas::new_practiced.eq(quotas::new_practiced - response.box_pos_delta.unwrap())))
                    .execute(conn)
                    .expect("failed to update quotas");
            } else {
                update(quotas::table)
                    .filter(quotas::id.eq(response.deck_id).and(quotas::days_to_go.eq(days_to_go.unwrap())))
                    .set((quotas::review_assigned.eq(quotas::review_assigned + response.box_pos_delta.unwrap()), quotas::review_practiced.eq(quotas::review_practiced - response.box_pos_delta.unwrap())))
                    .execute(conn)
                    .expect("failed to update quotas");
            }

            let deck_name = entries::table
                .filter(entries::id.eq(response.deck_id))
                .select(entries::name)
                .get_result::<String>(conn)
                .expect("failed to get deck name");

            // update box position of card and return its contents
            update(cards::table)
                .filter(cards::id.eq(response.card_id))
                .set(cards::box_position.eq(cards::box_position - response.box_pos_delta.unwrap()))
                .execute(conn)
                .expect("failed to update box pos");
            let card = cards::table
                .filter(cards::id.eq(response.card_id))
                .select((cards::id, cards::front, cards::back))
                .get_result::<(i32, String, String)>(conn)
                .expect("failed to get card contents");
                
            let card_results = Some(CardResults {
                stack_after: response.stack_after.clone(),
                user_answer: response.user_answer.clone(),
                card: ReviewCard { 
                    stack_before: response.stack_before.clone(), 
                    deck_name: deck_name, 
                    card: Card { id: card.0, front: card.1, back: card.2 }
                }
            });
            *curr_card = Some(response);
            return card_results;
        }
    }

 }

// #[tauri::command] 
// pub fn undo_get_last_card(state: State<DatabaseState>) -> Option<CardResults> { None }


