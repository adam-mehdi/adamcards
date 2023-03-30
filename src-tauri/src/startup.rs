use chrono::{Local, DateTime, Duration}; ///, Utc};


use diesel::insert_into;
use diesel::prelude::*;

use crate::edit::{write_quotas, insert_deck_contents, DeckNewContents};
use crate::models::NewCard;


use crate::home::{folder_system_is_empty, insert_entry, insert_deadline, insert_deck};


/**
 * Initializes root folder for first startup of app. Returns whether root folder was init
 */
pub fn init_root_folder(conn: &mut SqliteConnection) -> bool {
    if !folder_system_is_empty(conn) {
        return false;
    }
    use crate::schema::{entries, folders, userconfig};

    let entry_name = "My Trunk";
    let is_expanded: Option<bool> = Some(true);

    // create_entry(state, "My Trunk", None, md);
    insert_into(entries::table)
        .values((entries::name.eq(entry_name), entries::is_expanded.eq(is_expanded)))
        .execute(conn)
        .unwrap();
    let entry_id = entries::table
        .filter(entries::name.eq(entry_name))
        .order(entries::id.desc())
        .select(entries::id)
        .first::<i32>(conn)
        .unwrap();    

    // insert into specialized relation `Folder`/`Deadline`/`Deck` using id
    insert_into(folders::table)
        .values(folders::id.eq(entry_id))
        .execute(conn)
        .expect("failed to initialize root folder");


    insert_into(userconfig::table)
        .values((userconfig::is_dark_mode.eq(true), userconfig::is_text_field.eq(false)))
        .execute(conn)
        .expect("failed to initialize user config");

    true
}


pub fn init_getting_started(conn: &mut SqliteConnection) {
    use crate::schema::entries;

    let parent_id = entries::table
        .select(entries::id)
        .get_result::<i32>(conn)
        .expect("failed to get parent id");

    // in two days 
    let now: DateTime<Local> = Local::now();
    let next_week = now + Duration::days(1) + Duration::minutes(150);
    let formatted_date = next_week.format("%Y-%m-%d %H:%M:%S").to_string();

    // insert starter deadline
    let deadline_id = insert_entry(conn, Some(parent_id), "How to use Adam", "deadline");
    insert_deadline(conn, deadline_id, Some(formatted_date), Some(1), false);

    // insert deck 1: create (folder -> deadline -> deck), edit (create cards), review (until deadline)
    insert_starting_deck(conn, deadline_id, "1. Fundamentals");
    
    // insert deck 2: actions, textfield, prompt bar, reset deadline
    insert_starting_deck(conn, deadline_id, "2. Advanced Features");

    // insert deck 3: synthesis, rephrasing, explanations, instruction, upgrade, future
    insert_starting_deck(conn, deadline_id, "3. AI Magic");

}

fn insert_starting_deck(conn: &mut SqliteConnection, deadline_id: i32, deck_name: &str)  {
    let deck_id = insert_entry(conn, Some(deadline_id), deck_name, "deck");
    insert_deck(conn, deck_id, deadline_id);

    let deck_contents: DeckNewContents = get_starting_deck_contents(deck_id, deck_name.to_string());
    
    let ids = insert_deck_contents(conn, deck_contents, false);
    write_quotas(conn, deadline_id, deck_id, ids.len() as i32);
}

fn get_starting_deck_contents(deck_id: i32, deck_name: String) -> DeckNewContents {
    let cards: Vec<NewCard>;
    if deck_name.starts_with("1") {
        cards = vec![ 
            NewCard { front: String::from("Adam's folder system hierarchy consists of three organizational levels: Folders (for organization), Deadlines (housing various Decks), and Decks (containing decks). How many entries in the folder hierarchy are needed to create a card?"), back: String::from("3 (one folder, one deadline, one deck)") },
            NewCard { front: String::from("Using the action tray on the home screen, you can create, rename, move, or delete entries in the folder system. What icon opens the action tray?"), back: String::from("the vertical ellipsis ⋮") },
            NewCard { front: String::from("Once you set a deadline and create a deck, you can make cards. How does Adam make sure you learn those cards by your deadline?"), back: String::from("Adam assigns card reviews each day up to your deadline using the AM-1 algorithm. This allows you to learn and remember your cards guaranteed, with the minimum time and effort possible") },
        ]
    } else if deck_name.starts_with("2") {
        cards = vec![
            NewCard { front: String::from("What happens if you miss a day of reviews?"), back: String::from("Adam automatically adjusts card to spa") },
            NewCard { front: String::from("In addition to the standard Front/Back editor to create cards, Adam provides Textfield editor that create a card from each line with a double carrot like so: FRONT >> BACK. Why is this helpful?"), back: String::from("Allows you to create cards straight from your notes, saving time") },
            NewCard { front: String::from("Why does Adam prompt you to type out your answer to a card before revealing the back?"), back: String::from("The most effective way to study flashcards is to write your guess in your own words before revealing the card. It encourages active learning") },
            NewCard { front: String::from("Suppose you set a deadline for your midterm, and it has passed. How do you ensure you remember your cards for your final?"), back: String::from("reset the deadline on the home screen (a ⟳ button will appear on past deadlines to reset them)") },
        ]
    } else { 
        cards = vec![
            NewCard { front: String::from("Adam is a free and open-source application. However, it provides powerful AI features, which you can access by getting an OpenAI API key. How much will the AI features cost you?"), back: String::from("Exactly as much as OpenAI costs (.3 cents per thousand words). Adam takes absolutely none of it") },
            NewCard { front: String::from("What four AI features does Adam offer to accelerate your learning?"), back: String::from("synthesizer (source → cards), rephraser (front + back → newFront + newBack), explainer (front + back → explanation), instruction (front + back + your answer → instruction)") },
            NewCard { front: String::from("Adam allows you to use the power of GPT to create cards. How do you use this feature?"), back: String::from("enter your notes or source text in the edit page; you can see created cards") },
            NewCard { front: String::from("With Adam, you can be certain to learn the concept rather than memorize the card. What AI feature enables this?"), back: String::from("Adam rephrases the card question every time using GPT") },
            NewCard { front: String::from("You don't have to worry about when and where to apply Adam's AI features. It's done for you behind the scenes. What are the only things you have to worry about?"), back: String::from("Coming with material to learn and returning to review your cards") },

        ];
    }

    DeckNewContents { deck_id, deck_name, cards }
}

