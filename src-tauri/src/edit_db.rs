// #![allow(unused_imports)]
// #![allow(unused_variables)]
// #![allow(dead_code)]

use diesel::{insert_into, delete, update};
use diesel::prelude::*;

use chrono::prelude::*;

use tauri;
use serde::{
    Serialize, 
    Deserialize
};

use crate::home_db::DatabaseState;
use crate::models::{Card, NewCard};

#[derive(Serialize, Deserialize, Debug)]
pub struct DeckContents {
    pub deck_id: i32,
    pub deck_name: String,
    pub cards: Vec<Card>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeckNewContents {
    pub deck_id: i32,
    pub deck_name: String,
    pub cards: Vec<NewCard>
}

use crate::home_db::{
    compute_num_boxes_from_id, naive_to_localoffset
};


#[tauri::command] 
pub fn read_deadline_contents(state: tauri::State<DatabaseState>, deadline_id: i32) -> Vec<DeckContents> {
    use crate::schema::{parents, deckitems, cards, entries};
    
    let conn= &mut *state.conn.lock().unwrap();

    // get deck ids and names of deadline
    let deck_ids = parents::table
        .filter(parents::parent_id.eq(deadline_id))
        .select(parents::child_id)
        .load::<i32>(conn)
        .expect("failed to get decks");


    let mut deadline_contents: Vec<DeckContents> = Vec::new();
    // get card ids for each deck
    for deck_id in deck_ids {
        let item_ids = deckitems::table
            .filter(deckitems::deck_id.eq(deck_id))
            .select(deckitems::item_id)
            .load::<i32>(conn)
            .expect("failed to get item ids");

        let card_data = cards::table
            .filter(cards::id.eq_any(item_ids))
            .select((cards::id, cards::front, cards::back))
            .get_results::<(i32, String, String)>(conn)
            .expect("failed to get card contents");

        let mut cards: Vec<Card> = Vec::new();
        for card in card_data {
            let (id, front, back) = card;
            cards.push( Card { id, front, back });
        }


        let deck_name = entries::table
            .filter(entries::id.eq(deck_id))
            .select(entries::name)
            .get_result::<String>(conn)
            .expect("failed to get deck name");

        deadline_contents.push( DeckContents { deck_id, deck_name, cards } );
    }


    // get card contents for each card id
    deadline_contents
}


#[tauri::command]
pub fn delete_card(state: tauri::State<DatabaseState>, card_id: i32) {
    use crate::schema::{cards, deckitems, parents, quotas};

    let conn= &mut *state.conn.lock().unwrap();

    let deck_id = deckitems::table
        .filter(deckitems::item_id.eq(card_id))
        .select(deckitems::deck_id)
        .get_result::<i32>(conn)
        .expect("failed to retrieve deck id");

    let deadline_id = parents::table
        .filter(parents::child_id.eq(deck_id))
        .select(parents::parent_id)
        .get_result::<i32>(conn)
        .expect("failed to retrieve deadline id");

    let box_pos = delete(cards::table.filter(cards::id.eq(card_id)))
        .returning(cards::box_position)
        .get_result::<i32>(conn)
        .expect("failed to delete deck item");
    
    delete(deckitems::table.filter(deckitems::item_id.eq(card_id)))
        .execute(conn)
        .expect("failed to delete deck item");

    let days_to_go = get_days_to_go(conn, deadline_id);
    let num_boxes = compute_num_boxes_from_id(conn, deadline_id);

    let mut quota_records = quotas::table
        .filter(quotas::id.eq(deck_id).and(quotas::days_to_go.lt(days_to_go + 1)))
        .select((quotas::days_to_go, quotas::new_assigned, quotas::review_assigned))
        .get_results::<(i32, i32, i32)>(conn)
        .expect("failed to get quota records");

    quota_records.sort_by_key(|&record| record.0);

    let mut q_to_sub = num_boxes - box_pos;
    if q_to_sub == 0 { return; }

    // subtract from final day quotas
    quota_records[0].2 -= 1;
    q_to_sub -= 1;

    // subtract one from new quota
    if q_to_sub == 0 { return; }
    if box_pos == 0 {
        let mut i = days_to_go as usize;
        loop {
            if quota_records[i].1 > 0 {
                quota_records[i].1 -= 1;
                q_to_sub = q_to_sub - 1;
                break;
            }
            i -= 1;
        }
    }

    // subtract rest from review quotas
    let mut i = 0;
    loop {
        if quota_records[i].2 > 0 {
            let sub_amt = std::cmp::min(quota_records[i].2, q_to_sub);
            quota_records[i].2 -= sub_amt;
            q_to_sub = q_to_sub - sub_amt;
        }

        if q_to_sub > 0 {
            i = i + 1;
        } else {
            break;
        }
        
    }

    for quota_record in quota_records {
        update(quotas::table)
            .filter(quotas::id.eq(deck_id).and(quotas::days_to_go.eq(quota_record.0)))
            .set((quotas::new_assigned.eq(quota_record.1), quotas::review_assigned.eq(quota_record.2)))
            .execute(conn)
            .expect("failed to write quota back");
    }

}


// create cards, returning ids of returned cards
/**
 * Creates cards in deck_contents into the `cards` table associated with the proper deck 
 */
#[tauri::command]
pub fn create_cards(state: tauri::State<DatabaseState>, deadline_id: i32, deck_new_contents: DeckNewContents) -> Vec<i32> {
    use crate::schema::{cards, deckitems};
    
    let conn= &mut *state.conn.lock().unwrap();

    // add new cards to `cards` database

    let mut card_ids = Vec::new();
    for new_card in deck_new_contents.cards {
        let card_id = insert_into(deckitems::table)
            .values(deckitems::deck_id.eq(deck_new_contents.deck_id))
            .returning(deckitems::item_id)
            .get_result::<i32>(conn)
            .expect("failed to assign deck to card");

        insert_into(cards::table)
            .values((cards::front.eq(new_card.front), cards::back.eq(new_card.back), cards::id.eq(card_id), cards::box_position.eq(0)))
            .execute(conn)
            .expect("failed to insert new cards");

        card_ids.push(card_id);
    }

    // account for quotas
    write_quotas(conn, deadline_id, deck_new_contents.deck_id, card_ids.len() as i32);

    // return ids of new cards
    card_ids

}


/**
 * Updates an existing card in the file system, called on the `onChange` event. 
 * Allows a user to change the contents of a card with the changes saving
 */
#[tauri::command]
pub fn update_card(state: tauri::State<DatabaseState>, card: Card) {
    use crate::schema::cards;
    
    let conn= &mut *state.conn.lock().unwrap();

    // add new cards to `cards` database
    update(cards::table)
        .filter(cards::id.eq(card.id))
        .set((cards::front.eq(card.front), cards::back.eq(card.back)))
        .execute(conn)
        .expect("failed to insert quota record");
}



////////////////// HELPERS FOR QUOTAS //////////////////


// quotas helper
use crate::models::QuotaRecord;
use crate::utils_db::days_until_deadline;


pub fn get_days_to_go(conn: &mut PgConnection, deadline_id: i32) -> i32 {
    use crate::schema::deadlines;

    let deadline_date = deadlines::table
        .filter(deadlines::id.eq(deadline_id))
        .select(deadlines::deadline_date)
        .get_result::<NaiveDateTime>(conn)
        .expect("failed to get deadline date");

    let fixed_offset_date_time = naive_to_localoffset(deadline_date);

    let days_to_go = days_until_deadline(
        fixed_offset_date_time,
        2,
        14
    ) as i32;

    days_to_go
}

pub fn write_quotas(conn: &mut PgConnection, deadline_id: i32, deck_id: i32, num_cards: i32) {

    use crate::schema::quotas;
    // write quotas for `num_new` new cards
    let days_to_go = get_days_to_go(conn, deadline_id);
    if num_cards > 0 {
        let num_boxes = compute_num_boxes_from_id(conn, deadline_id);
        let mut quota_records = compute_quotas(num_cards, days_to_go, num_boxes);
        discount_past_progressions(conn, &mut quota_records, deck_id);

        for new_quota_record in quota_records {
            let existing_quota = quotas::table
                .filter(quotas::id.eq(deck_id).and(quotas::days_to_go.eq(new_quota_record.days_to_go)))
                .select((quotas::days_to_go, quotas::new_assigned, quotas::review_assigned, quotas::new_quota_initial, quotas::review_quota_initial, quotas::new_practiced, quotas::review_practiced))
                .get_result::<(i32, i32, i32, i32, i32, i32, i32)>(conn)
                .optional()
                .expect("failed to retrieve existing quota record");

            match existing_quota {
                Some(q) => {
                    let combined_quota_record = add_quota_records(
                        new_quota_record,
                        QuotaRecord { days_to_go: q.0, new_assigned: q.1, review_assigned: q.2, new_quota_initial: q.3, review_quota_initial: q.4, new_practiced: q.5, review_practiced: q.6 }
                    );
    
                    update(quotas::table)
                        .filter(quotas::days_to_go.eq(combined_quota_record.days_to_go))
                        .set(combined_quota_record)
                        .execute(conn)
                        .expect("failed to insert quota record");
                },

                None => {
                    insert_into(quotas::table)
                        .values((new_quota_record, quotas::id.eq(deck_id)))
                        .execute(conn)
                        .expect("failed to insert new quota record into empty quotas");
                }
                
            }

        }

    }
}



// computes quotas for `num_cards` given `days_to_go` and `num_boxes`
pub fn compute_quotas(num_cards: i32, days_to_go: i32, num_boxes: i32)  -> Vec<QuotaRecord> {
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
        let dtg = nq.len() - 1 - i; // days to go
        quotas.push(
            QuotaRecord {
                days_to_go: dtg as i32,
                new_assigned: nq[i],
                review_assigned: rq[i],
                new_quota_initial: nq[i],
                review_quota_initial: rq[i],
                new_practiced: 0,
                review_practiced: 0
            }
        );
    }

    quotas
}

// fn subtract_quota_records(q1: QuotaRecord, q2: QuotaRecord) -> QuotaRecord{
//     QuotaRecord {
//         days_to_go: q1.days_to_go,
//         new_assigned: q1.new_assigned - q2.new_assigned,
//         review_assigned: q1.review_assigned - q2.review_assigned,
//         new_quota_initial: q1.new_quota_initial - q2.new_quota_initial,
//         review_quota_initial: q1.review_quota_initial - q2.review_quota_initial,
//         new_practiced: q1.new_practiced - q2.new_practiced,
//         review_practiced: q1.review_practiced - q2.review_practiced
//     }
// }

fn add_quota_records(q1: QuotaRecord, q2: QuotaRecord) -> QuotaRecord{
    QuotaRecord {
        days_to_go: q1.days_to_go,
        new_assigned: q1.new_assigned + q2.new_assigned,
        review_assigned: q1.review_assigned + q2.review_assigned,
        new_quota_initial: q1.new_quota_initial + q2.new_quota_initial,
        review_quota_initial: q1.review_quota_initial + q2.review_quota_initial,
        new_practiced: q1.new_practiced + q2.new_practiced,
        review_practiced: q1.review_practiced + q2.review_practiced
    }
}



#[tauri::command] 
pub fn write_text_field(state: tauri::State<DatabaseState>, is_text_field: bool) { 
    use crate::schema::userconfig;
    
    let conn= &mut *state.conn.lock().unwrap();
    update(userconfig::table)
        .set(userconfig::is_text_field.eq(is_text_field))
        .execute(conn)
        .expect("failed to set dark mode");
}

/**
 * Decreases values in `new_quotas` to account for past reviews, encoded in 
 * box positions of cards in `cards`
 * 
 * This function arises from the scheme where quotas are computed only based on
 * number of cards and days until deadline
 */
pub fn discount_past_progressions(conn: &mut PgConnection, new_quotas: &mut Vec<QuotaRecord>, deck_id: i32) {
     use crate::schema::{cards, deckitems};

    if new_quotas.len() == 1 {
        return;
    }

     // get array of card box positions in deck
     let box_positions = deckitems::table
        .inner_join(cards::table.on(cards::id.eq(deckitems::item_id)))
        .filter(deckitems::deck_id.eq(deck_id))
        .select(cards::box_position)
        .get_results::<i32>(conn)
        .expect("failed to get box positions");

    // return if all new cards
    if box_positions.iter().sum::<i32>() == 0 {
        return;
    }

    // get number of cards which are advanced from the initial box
    let tot_new_advanced: i32 = box_positions.iter()
        .map(|x| (*x > 0) as i32).sum();

    // get number of times cards are advanced, not counting initial advance
    let tot_review_advanced = box_positions.iter()
        .map(|x| (x - 1) * ((*x > 0) as i32))
        .sum::<i32>();


    let days = new_quotas.len() as i32 - 1;
    let new_per_day = tot_new_advanced / days;
    let mut remainder = tot_new_advanced - days * new_per_day;
    for dtg in 1..new_quotas.len() {
        new_quotas[dtg].new_assigned -= new_per_day;
        new_quotas[dtg].new_quota_initial -= new_per_day;

        // subtract remainder to day furthest from deadline
        if remainder > 0 {
            let sub_value = std::cmp::min(new_quotas[dtg].new_assigned, remainder);
            new_quotas[dtg].new_assigned -= sub_value;
            new_quotas[dtg].new_quota_initial -= sub_value;
            remainder -= sub_value;
        }
    }
    assert!(remainder == 0);

    let review_per_day = tot_review_advanced / days;

    let mut remainder = tot_review_advanced - days * review_per_day;
    for dtg in (1..new_quotas.len()).rev() {
            new_quotas[dtg].review_assigned -= review_per_day;
            new_quotas[dtg].review_quota_initial -= review_per_day;

            // subtract remainder to before deadline day
            if remainder > 0 {
            let sub_value = std::cmp::min(new_quotas[dtg].review_assigned, remainder);
            new_quotas[dtg].review_assigned -= sub_value;
            new_quotas[dtg].review_quota_initial -= sub_value;
            remainder -= sub_value;
        }
    }

    assert!(remainder == 0);

}