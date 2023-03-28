use chrono::Utc;
use crate::review_db::{ReviewCard, get_queue_score};
use crate::models::Card;


use diesel::update;
use diesel::prelude::*;
// use diesel::serialize::ToSql;


pub fn pop_review_anki_card(conn: &mut SqliteConnection, deck_id: i32) -> ReviewCard {
    use crate::schema::{cards, entries};

    // get all cards that are due today
    let today = chrono::Local::now().date_naive();
    let popped_card = cards::table
        .filter(cards::next_practice.le(today).and(cards::deck_id.eq(deck_id))) // filter cards whose next_practice is in the past
        .order(cards::queue_score.asc())
        .select((cards::id, cards::front, cards::back))
        .first::<(i32, String, String)>(conn)
        .expect("Error choosing card to review");

    let deck_name = entries::table
        .filter(entries::id.eq(deck_id))
        .select(entries::name)
        .get_result::<String>(conn)
        .expect("failed to get deck name");

    ReviewCard { 
        stack_before: String::from("review"), 
        deck_name, 
        card: Card { 
            id: popped_card.0,
            front: popped_card.1,
            back: popped_card.2
        }
    }
}


pub struct SmResponse {
    interval: i32, // The interval between repetitions (in days).
    repetitions: i32,// The number of times a user has seen the flashcard.
    ease_factor: f32,// The easiness factor for the flashcard.
}

// quality is score, and it ranges 1-5. A score of 3 or more is a success
// again hard okay good easy
pub fn calculate_sm(
    quality: i32,
    repetitions: i32,
    previous_interval: i32,
    previous_ease_factor: f32,
) -> SmResponse {
    let (interval, repetitions, ease_factor) = if quality >= 3 {
        let interval = match repetitions {
            0 => 1,
            1 => 6,
            _ => (previous_interval as f32 * previous_ease_factor).round() as i32,
        };

        let repetitions = repetitions + 1;
        let ease_factor =
            previous_ease_factor + (0.1 - (5 - quality) as f32 * (0.08 + (5 - quality) as f32 * 0.02));

        (interval, repetitions, ease_factor)
    } else if quality == 1 {
        // repeat if pressing again
        (0, repetitions, previous_ease_factor)
    } else {
        (1, repetitions, previous_ease_factor)
    };

    let ease_factor = if ease_factor < 1.4 { 1.4 } else { ease_factor };

    SmResponse {
        interval,
        repetitions,
        ease_factor,
    }
}



pub fn update_card_anki(conn: &mut SqliteConnection, card: &ReviewCard, score: i32) -> String {
    use crate::schema::{cards, ankiquotas};

    // get card's current stats
    let (ease_factor, interval, repetitions) = cards::table
        .filter(cards::id.eq(card.card.id))
        .select((cards::easiness, cards::interval, cards::repetitions))
        .get_result::<(Option<f32>, Option<i32>, Option<i32>)>(conn)
        .expect("failed to get current card stats");

    let new_stats = calculate_sm( 
            score,
            repetitions.unwrap(),
            interval.unwrap(),
            ease_factor.unwrap()
    );

    let next_practice = Utc::now() + chrono::Duration::days(i64::from(new_stats.interval));

    // update card's contents and box pos, returning new box pos
    update(cards::table)
        .filter(cards::id.eq(card.card.id))
        .set((
            cards::front.eq(&card.card.front), 
            cards::back.eq(&card.card.back), 
            cards::queue_score.eq(get_queue_score()),
            cards::next_practice.eq(next_practice.date_naive()),
            cards::easiness.eq(new_stats.ease_factor),
            cards::interval.eq(new_stats.interval),
            cards::repetitions.eq(new_stats.repetitions)
        ))
        .execute(conn)
        .expect("failed to update card box pos");

    // get deck id
    let deck_id = cards::table
        .filter(cards::id.eq(card.card.id))
        .select(cards::deck_id)
        .get_result::<i32>(conn)
        .expect("failed to get deck id");

    if new_stats.interval > 0 { 
        // get current day in anki quota
    
        let today = chrono::Local::now().date_naive();
        let prac_new = (&card.stack_before == "new") as i32;
        let prac_review = (&card.stack_before == "review") as i32;

        update(ankiquotas::table)
            .filter(ankiquotas::date_practiced.eq(today).and(ankiquotas::deck_id.eq(deck_id)))
            .set((
                ankiquotas::new_practiced.eq(ankiquotas::new_practiced + prac_new), 
                ankiquotas::review_practiced.eq(ankiquotas::review_practiced + prac_review), 
            ))
            .execute(conn)
            .unwrap();
     }

    let stack_after = if new_stats.interval > 0 { 
        String::from("done") 
    } else { 
        String::from(&card.stack_before)
    };

    stack_after

}
