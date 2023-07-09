// #![allow(unused_imports)]
// #![allow(unused_variables)]
// #![allow(dead_code)]

use std::sync::{ Mutex, Arc };
use chrono::{prelude::*, Local, DateTime};

use tauri;
use serde::{
    Serialize, 
    Deserialize
};

use diesel::{insert_into, delete, update};
use diesel::prelude::*;
use diesel::result::Error;

use crate::utils::get_num_boxes;
use crate::utils::handle_missed_days;
use crate::edit::{get_days_to_go, write_quotas};

use tauri::api::dialog::FileDialogBuilder;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::io::BufReader;

use futures::channel::oneshot;


pub struct DatabaseState {
    pub conn: Arc<Mutex<SqliteConnection>>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct EntryMetadata {
    pub entry_type: String,
    pub deadline_date: Option<String>,
    pub study_intensity: Option<i32>,
    pub new_per_day: Option<i32>
}


#[derive(Serialize, Deserialize, Debug)]
pub struct FolderSystem {
    pub pairs: Vec<EntryPair>,
    pub data: Vec<EntryData>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EntryPair {
    pub parent_id: i32,
    pub child_id: i32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EntryData {
    pub entry_id: i32,
    pub entry_name: String,
    pub is_expanded: Option<bool>,
    pub entry_type: String,
    pub entry_quota: Option<Quota>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Quota {
    pub new_left: i32,
    pub review_left: i32,
    pub num_progressed: i32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    is_dark_mode: bool,
    is_textfield: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProgressData {
    curr_day: i32,
    tot_days: i32
}

pub fn folder_system_is_empty(conn: &mut SqliteConnection) -> bool {
    use crate::schema::folders;
    let all_folders = folders::table
        .select(folders::id)
        .load::<i32>(conn)
        .expect("Error loading folder");

    if all_folders.len() == 0 {
        return true;
    }
    false

}

#[tauri::command] 
pub fn read_user_config(state: tauri::State<DatabaseState>) -> AppConfig { 
    use crate::schema::userconfig;

    let conn= &mut *state.conn.lock().unwrap();
    let config = userconfig::table
        .select((userconfig::is_dark_mode, userconfig::is_text_field))
        .load::<(bool, bool)>(conn)
        .expect("failed to read config");

    if config.len() != 1 {
        eprintln!("multiple rows in user config")
    }

    AppConfig { 
        is_dark_mode: config[0].0,
        is_textfield: config[0].1
    }
}

#[tauri::command] 
pub fn write_dark_mode(state: tauri::State<DatabaseState>, is_dark_mode: bool) { 
    use crate::schema::userconfig;
    
    let conn= &mut *state.conn.lock().unwrap();
    update(userconfig::table)
        .set(userconfig::is_dark_mode.eq(is_dark_mode))
        .execute(conn)
        .expect("failed to set dark mode");
}

#[tauri::command] 
pub fn write_api_key(state: tauri::State<DatabaseState>, api_key: String) { 
    use crate::schema::userconfig;
    
    let conn= &mut *state.conn.lock().unwrap();
    update(userconfig::table)
        .set(userconfig::api_key.eq(api_key))
        .execute(conn)
        .expect("failed to set dark mode");
}

#[tauri::command] 
pub fn get_api_key(state: tauri::State<DatabaseState>) -> Option<String> { 
    use crate::schema::userconfig;
    
    let conn= &mut *state.conn.lock().unwrap();
    let api_key = userconfig::table
        .select(userconfig::api_key)
        .get_result::<Option<String>>(conn)
        .expect("failed to set dark mode");

    api_key
}



#[tauri::command] 
pub fn read_folder_system(state: tauri::State<DatabaseState>) -> Option<FolderSystem> {
    use crate::schema::{entries, parents};

    let conn= &mut *state.conn.lock().unwrap();

    // read parents
    let all_parents = parents::table
        .select((parents::parent_id, parents::child_id))
        .load::<(i32, i32)>(conn)
        .expect("Error loading parents");


    let mut pairs = Vec::new();
    for parent in all_parents {
        pairs.push(EntryPair {parent_id: parent.0, child_id: parent.1});
    }

    let all_entries = entries::table
        .select((entries::id, entries::name, entries::is_expanded))
        .load::<(i32, String, Option<bool>)>(conn)
        .expect("Error loading parents");

    let mut data: Vec<EntryData> = Vec::new();
    for entry in all_entries {
        let entry_id = entry.0;
        let type_result = get_entry_type(conn, entry_id);
        let entry_type;

        match type_result {
            Ok(t) => {
                entry_type = t;
            },

            Err(Error::NotFound) | Err(_) => {
                eprintln!("Database Integrity Error: failed to find entry id {} \
                in decks, deadlines, or folders", entry_id);
                return None;
            }
        }

        data.push(
            EntryData {
                entry_id,
                entry_name: entry.1,
                is_expanded: entry.2,
                entry_type,
                entry_quota: None // Some(Quota{new_left: 0, review_left:0, num_progressed: 0})
            }
        );
    }

    // write quotas into data in-place
    let mut folder_system = FolderSystem { pairs, data };
    folder_system = compute_quotas_(conn, folder_system);

    Some(folder_system)
}


fn compute_quotas_(conn: &mut SqliteConnection, mut folder_system: FolderSystem) -> FolderSystem {
    use crate::schema::parents;

    // first write the quotas of all of the decks
    let mut deck_ids = Vec::new();
    for entry in &mut folder_system.data {
        let entry_type = get_entry_type(conn, entry.entry_id)
            .expect("failed to retrieve entry type");
        if entry_type == String::from("deck") {

            entry.entry_quota = get_deck_quota(conn, entry.entry_id);

            deck_ids.push(entry.entry_id);

        }
    }

    // now get quotas of deadlines as as sum of quotas of children 
    
    let deadline_ids = parents::table
        .filter(parents::child_id.eq_any(deck_ids))
        .select(parents::parent_id)
        .load::<i32>(conn)
        .expect("failed to get children");

    for deadline_id in deadline_ids {
        let child_decks = parents::table
            .filter(parents::parent_id.eq(deadline_id))
            .select(parents::child_id)
            .load::<i32>(conn)
            .expect("failed to get decks of deadline");

        let child_quotas: Vec<&Option<Quota>> = folder_system.data.iter()
            .filter(|x| child_decks.contains(&x.entry_id))
            .map(|x| &x.entry_quota)
            .collect();

        // aggregate quotas
        let mut new_left = 0; 
        let mut review_left = 0; 
        let mut num_progressed = 0;
        for child_quota in child_quotas { 
            if let Some(quota) = child_quota {
                new_left += quota.new_left;
                review_left += quota.review_left;
                num_progressed += quota.num_progressed;
            }
        }

        for entry in &mut folder_system.data {
            if entry.entry_id == deadline_id {
                entry.entry_quota = Some(Quota { new_left, review_left, num_progressed });
            }
        }
    }

    folder_system
}



pub fn get_deck_quota(conn: &mut SqliteConnection, deck_id: i32) -> Option<Quota> {
    use crate::schema::{parents, deadlines, quotas, cards, decks, ankiquotas};

    // get days to go of this deck
    let deadline_id = parents::table
        .filter(parents::child_id.eq(deck_id))
        .select(parents::parent_id)
        .get_result::<i32>(conn)
        .expect("failed to get deadline parent");

    let is_anki = deadlines::table
        .find(deadline_id)
        .select(deadlines::is_anki)
        .first::<bool>(conn)
        .expect("failed to get is_anki");

    if is_anki {
        // forgetting to find whether next_practice is before today
        let today = chrono::Local::now().date_naive();
        let card_reps = cards::table
            .filter(cards::deck_id.eq(deck_id).and(cards::next_practice.le(today).or(cards::next_practice.is_null())))
            .select(cards::repetitions)
            .get_results::<Option<i32>>(conn)
            .expect("failed to get card repetitions");

        let mut new_per_day = decks::table
            .find(deck_id)
            .select(decks::new_per_day)
            .get_result::<Option<i32>>(conn)
            .expect("failed to get new_per_day")
            .expect("failed to unwrap new_per_day");

        let (mut num_new, mut num_review) = (0, 0);
        for rep in card_reps {
            if rep.unwrap() > 0 {
                num_review = num_review + 1;
            } else {
                num_new = num_new + 1;
            }
        }

        let today = chrono::Local::now().date_naive();
        let results: Option<(i32, i32)> = ankiquotas::table
            .filter(ankiquotas::date_practiced.eq(today).and(ankiquotas::deck_id.eq(deck_id)))
            .select((ankiquotas::new_practiced, ankiquotas::review_practiced))
            .get_result::<(i32, i32)>(conn)
            .optional()
            .unwrap();

            
    
        let mut num_progressed = 0;
        if let Some((new_prac, rev_prac)) = results {
            // num_new -= new_prac;
            // num_review -= rev_prac;
            
            num_progressed = new_prac + rev_prac;

            // for case where new_per_day is changed to be less than new_prac
            if new_prac > new_per_day {
                new_per_day = new_prac;
            }

            num_new = std::cmp::max(
                std::cmp::min(new_per_day - new_prac, num_new), 
                0
            );
            num_review = std::cmp::max( // FIX FOR REVIEW
                num_review,
                0
            );
        } else {
            insert_into(ankiquotas::table)
                .values((
                        ankiquotas::deck_id.eq(deck_id), 
                        ankiquotas::date_practiced.eq(today), 
                        ankiquotas::new_practiced.eq(0), 
                        ankiquotas::review_practiced.eq(0)
                ))
                .execute(conn)
                .expect("failed to insert new ankiquotas entry for today");
        }

        

        return Some(Quota {
            new_left: num_new,
            review_left: num_review,
            num_progressed
        })
    }

    let days_to_go = get_days_to_go(conn, deadline_id);
    handle_missed_days(conn, deck_id, &days_to_go);

    let entry_quota = quotas::table
        .filter(quotas::id.eq(deck_id).and(quotas::days_to_go.eq(days_to_go)))
        .select((quotas::new_assigned, quotas::review_assigned, quotas::new_practiced, quotas::review_practiced))
        .get_result::<(i32, i32, i32, i32)>(conn)
        .optional()
        .expect("failed to grab today's quota");

    if let Some((new_assigned, review_assigned, new_prac, review_prac)) = entry_quota {
        return Some(Quota { 
            new_left: new_assigned, 
            review_left: review_assigned,
            num_progressed: new_prac + review_prac
        });
    }
    None

}

/**
 * Given an id of an Entry, returns whether this id corresponds to a 
 * folder, deadline, or deck.
 * 
 * Args:
 *  conn: connection to diesel psql database
 *  entry_id: id of entry to give a type to
 *  
 */
fn get_entry_type(conn: &mut SqliteConnection, entry_id: i32) -> Result<String, Error> {
    use crate::schema::{folders, decks, deadlines};

    let folder_id = folders::table
        .select(folders::id)
        .filter(folders::id.eq(entry_id))
        .first::<i32>(conn);
    if let Ok(_) = folder_id {
        return Ok("folder".to_string());
    }

    let deadline_id = deadlines::table
        .filter(deadlines::id.eq(entry_id))
        .select(deadlines::id)
        .first::<i32>(conn);
    if let Ok(id) = deadline_id {
        let is_anki = deadlines::table
        .filter(deadlines::id.eq(id))
        .select(deadlines::is_anki)
        .get_result::<bool>(conn)
        .expect("failed to get is_anki");
        if is_anki {
            return Ok("ankibox".to_string())
        } else {
            return Ok("deadline".to_string());
        }
    }

    let deck_id = decks::table
        .select(decks::id)
        .filter(decks::id.eq(entry_id))
        .first::<i32>(conn);
    if let Ok(_) = deck_id {
        return Ok("deck".to_string());
    }

    Err(Error::NotFound)
}



/** 
 * Creates entry in database.
 * 
 * Args:
 *  conn: connection to diesel psql database
 *  entry_name: name of entry to be created
 *  parent_id: id of parent of this entry if not root folder
 *  entry_metadata: {entry_type: String, deadline_date: Option<String>, study_intensity: Option<String>}
 *  
 */
#[tauri::command] 
pub fn create_entry(state: tauri::State<DatabaseState>, entry_name: &str, parent_id: Option<i32>, md: EntryMetadata) {
    use crate::schema::folders;

    let conn= &mut *state.conn.lock().unwrap();
    
    let entry_id =  insert_entry(conn, parent_id, entry_name, &md.entry_type);

    let entry_type = md.entry_type.as_str();

    // insert into specialized relation `Folder`/`Deadline`/`Deck` using id
    match entry_type {
        "folder" => { insert_into(folders::table).values(folders::id.eq(entry_id)).execute(conn).unwrap(); },
        "deadline" | "ankibox" => insert_deadline(conn, entry_id, md.deadline_date, md.study_intensity, entry_type == "ankibox"),
        "deck" => insert_deck(conn, entry_id, parent_id.expect("no parent to deck"), md.new_per_day),
        _ => eprintln!("failed to create entry")

    }


}

// returns entry_id
pub fn insert_entry(conn: &mut SqliteConnection, parent_id: Option<i32>, entry_name: &str, entry_type: &str) -> i32 {
    use crate::schema::{entries, parents};

    let is_expanded = if entry_type == "deck" { None } else { Some(true) };

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

    if let Some(pid) = parent_id {
        insert_into(parents::table)
            .values((parents::child_id.eq(entry_id), parents::parent_id.eq(pid)))
            .execute(conn)
            .unwrap();
    }

    entry_id
}

pub fn insert_deadline(conn: &mut SqliteConnection, entry_id: i32, deadline_date: Option<String>, study_intensity: Option<i32>, is_anki: bool) {
    use crate::schema::deadlines;

    let (deadline_date, study_intensity, num_reset) = if !is_anki {
        (Some(string_to_chrono(&deadline_date.unwrap()).naive_local()), Some(study_intensity.unwrap()), Some(0))
    } else {
        (None, None, None)
    };
    
    insert_into(deadlines::table)
        .values((
            deadlines::id.eq(entry_id), 
            deadlines::deadline_date.eq(deadline_date),
            deadlines::study_intensity.eq(study_intensity),
            deadlines::num_reset.eq(num_reset),
            deadlines::is_anki.eq(is_anki)
        ))
        .execute(conn)
        .unwrap();

}

pub fn insert_deck(conn: &mut SqliteConnection, deck_id: i32, deadline_id: i32, new_per_day: Option<i32>) {
    use crate::schema::{decks, deadlines};

    let is_anki = deadlines::table
        .filter(deadlines::id.eq(deadline_id))
        .select(deadlines::is_anki)
        .get_result::<bool>(conn)
        .expect("failed to get parent deadline");

    let num_boxes = if !is_anki { 
        Some(compute_num_boxes_from_id(conn, deadline_id))
    } else {
        None // new_per_day is 5 by default 
    };

    insert_into(decks::table)
        .values((
            decks::id.eq(deck_id),
            decks::num_boxes.eq(num_boxes),
            decks::new_per_day.eq(new_per_day)
        ))
        .execute(conn)
        .unwrap();
}


// fn get_current_time() -> NaiveDateTime {
//     let current_time = Local::now().naive_local();
//     current_time
// }

/** 
 * Deletes from the database a file system entry, its children, and all their contents
 * 
 * Args:
 *  conn: connection to diesel psql database
 *  entry_id: id of entry to be deleted
 *  entry_type: type of entry to be deleted. is in "deadline", "folder", or "deck"
 */
#[tauri::command] 
pub fn delete_entry(state: tauri::State<DatabaseState>, entry_id: i32) {
    use crate::schema::{entries, parents};
    // TODO: decrement quota

    let conn= &mut *state.conn.lock().unwrap();

    // iteratively delete ids that descend from this id
    let mut parents = vec![entry_id];
    let mut i = 0;
    loop {
        let children = parents::table
            .filter(parents::parent_id.eq_any(parents))
            .select(parents::child_id)
            .get_results(conn)
            .expect("failed to select children");

        if children.len() == 0 && i == 0 {
            delete(parents::table.filter(parents::child_id.eq(entry_id)))
                .execute(conn)
                .expect("failed to delete deck pair");

        }

        if children.len() == 0 {
            break;
        }

        // delete all descending entries; deletions cascade to parents and deck contents
        delete(entries::table.filter(entries::id.eq_any(&children)))
            .execute(conn)
            .expect("failed to delete children");

        parents = children;
        i += 1;
    }

    // delete this entry
    delete(entries::table.filter(entries::id.eq(entry_id)))
        .execute(conn)
        .expect("failed to delete entry");

    

}



/** 
 * Moves an entry to a new folder. Supported entry types are folder and deadline
 * 
 */
#[tauri::command] 
pub fn move_entry(state: tauri::State<DatabaseState>, entry_id: i32, new_parent_id: i32) {
    use crate::schema::parents;
    let conn= &mut *state.conn.lock().unwrap();

    // update (parent_id, entry_id) in parents, setting parent_id to new_parent_id
    let update_count = update(parents::table)
        .filter(parents::child_id.eq(entry_id))
        .set(parents::parent_id.eq(new_parent_id))
        .execute(conn) 
        .expect("failed to set new parent");

    if update_count != 1 {
        eprintln!("Warning: renamed multiple entries");
    }
}


/** 
 * Renames an entry `entry_id` to `new_name`
 * 
 */
#[tauri::command] 
pub fn rename_entry(state: tauri::State<DatabaseState>, entry_id: i32, new_name: String) {
    use crate::schema::entries;
    let conn= &mut *state.conn.lock().unwrap();

    // update name attribute at entry_id from entries relation to new_name
    let update_count = update(entries::table)
        .filter(entries::id.eq(entry_id))
        .set(entries::name.eq(new_name))
        .execute(conn)
        .expect("failed to rename");

    if update_count != 1 {
        eprintln!("Warning: renamed multiple entries");
    }

}


#[tauri::command]
pub fn is_duplicate_name(state: tauri::State<DatabaseState>, parent_id: Option<i32>, new_name: String) -> bool {
    use crate::schema::{parents, entries};
    let conn= &mut *state.conn.lock().unwrap();

    if let None = parent_id {
        return false;
    }
    
    // select children of `parent_id`
    let parent_id = parent_id.unwrap();
    let children = parents::table
        .filter(parents::parent_id.eq(parent_id))
        .select(parents::child_id)
        .load::<i32>(conn)
        .expect("failed to load children of new parent");

    // select those children with a name equal to `new_name`
    let duplicate_exists = entries::table
        .filter(entries::id.eq_any(children).and(entries::name.eq(new_name)))
        .select(entries::id)
        .load::<i32>(conn)
        .expect("failed to grab children with duplicate names")
        .len() > 0;

    duplicate_exists
}





/**
 * Converts a string in the format YYYY-MM-DD HH:MM:SS to a NaiveDateTime taking
 * local timezone into account
 */
fn string_to_chrono(datetime: &str) -> DateTime<FixedOffset> {
    let format_str = "%Y-%m-%d %H:%M:%S";
    let naive_date_time = NaiveDateTime::parse_from_str(datetime, format_str)
        .expect("invalid deadline input");
    naive_to_localoffset(naive_date_time)
    
}

fn get_local_datetime() -> DateTime<FixedOffset> {
    let local_date_time = Local::now();
    let local_offset = local_date_time.offset();
    let fixed_offset_date_time = local_date_time.with_timezone(local_offset);
    return fixed_offset_date_time;
}

#[tauri::command]
pub fn entered_past_deadline(deadline: String) -> bool {
    let datetime = string_to_chrono(&deadline);
    let now = get_local_datetime();
    datetime < now
}


// returns deadline date in MMM dd mm:ss format and whether it is complete
#[tauri::command] 
pub fn get_deadline_date(state: tauri::State<DatabaseState>, deadline_id: i32) -> Option<(String, bool)> {
    use crate::schema::deadlines;
    let conn= &mut *state.conn.lock().unwrap();

    let (deadline_date, is_anki)  = deadlines::table
        .filter(deadlines::id.eq(deadline_id))
        .select((deadlines::deadline_date, deadlines::is_anki))
        .get_result::<(Option<NaiveDateTime>, bool)>(conn)
        .expect("failed to load deadline date");

    // deadline date represents UTC timezone; convert it to local
    if is_anki { 
        return None; 
    }

    let deadline_date = Local.from_local_datetime(&deadline_date.unwrap()).unwrap().naive_local();
    let formatted_date = deadline_date.format("%b %d %H:%M").to_string();

    let is_complete = naive_to_localoffset(deadline_date).timestamp() < Local::now().timestamp();

    Some((formatted_date, is_complete))

}

pub fn compute_num_boxes_from_id(conn: &mut SqliteConnection, parent_id: i32) -> i32 {
    use crate::schema::deadlines;

    let deadline_info = deadlines::table
        .filter(deadlines::id.eq(parent_id))
        .select((deadlines::study_intensity, deadlines::num_reset))
        .get_result::<(Option<i32>, Option<i32>)>(conn)
        .expect("failed to get parent deadline info");


    let study_intensity = deadline_info.0.expect("did not record study intensity");
    let num_reset = deadline_info.1.unwrap();

    let days_to_go = get_days_to_go(conn, parent_id);
    get_num_boxes(days_to_go as i32, study_intensity, num_reset)
}


pub fn naive_to_localoffset(naive_date_time: NaiveDateTime) -> DateTime<FixedOffset> {
    let local_date_time = Local.from_local_datetime(&naive_date_time).unwrap();
    let fixed_offset_date_time = local_date_time.with_timezone(Local::now().offset());
    fixed_offset_date_time
}

#[tauri::command]
pub fn reset_deadline(
    state: tauri::State<DatabaseState>,
    deadline_id: i32,
    study_intensity: i32,
    new_deadline_date: String
) {
    use crate::schema::{quotas, parents, deadlines, cards};

    let conn= &mut *state.conn.lock().unwrap();

    // update deadline date and num_reset
    let deadline = string_to_chrono(&new_deadline_date);

    update(deadlines::table)
        .filter(deadlines::id.eq(deadline_id))
        .set((deadlines::num_reset.eq(deadlines::num_reset + 1), deadlines::study_intensity.eq(study_intensity), deadlines::deadline_date.eq(deadline.naive_local())))
        .execute(conn)
        .expect("failed to update deadline num reset");
    

    let deck_ids = parents::table
        .filter(parents::parent_id.eq(deadline_id))
        .select(parents::child_id)
        .get_results::<i32>(conn)
        .expect("failed to retrieve deck ids");

    // update deck num_boxes
    for deck_id in deck_ids {
        delete(quotas::table)
            .filter(quotas::id.eq(deck_id))
            .execute(conn)
            .expect("failed to delete existing quotas");

        let num_cards = cards::table
            .filter(cards::deck_id.eq(deck_id))
            .select(cards::id)
            .get_results::<i32>(conn)
            .expect("failed to get number of items")
            .len() as i32;

        write_quotas(conn, deadline_id, deck_id, num_cards);

    }

}


#[tauri::command]
pub fn toggle_is_expanded(state: tauri::State<DatabaseState>, entry_id: i32, is_expanded: bool) {
    use crate::schema::entries;

    let conn= &mut *state.conn.lock().unwrap();

    update(entries::table)
        .filter(entries::id.eq(entry_id))
        .set(entries::is_expanded.eq(is_expanded))
        .execute(conn)
        .expect("failed to update expanded");
    
}

#[derive(Serialize, Deserialize)]
struct SubjectContents {
    name: String,
    decks: Vec<DeckContents>,

}

#[derive(Serialize, Deserialize)]
struct DeckContents {
    name: String,
    new_per_day: i32,
    cards: Vec<CardContents>
}

#[derive(Serialize, Deserialize, Queryable, )]
struct CardContents {
    front: String,
    back: String,
    explanation: Option<String>,
    queue_score: Option<i32>,
    repetitions: Option<i32>,
    easiness: Option<f32>,
    interval: Option<i32>,
    next_practice: Option<NaiveDate>,
}


#[tauri::command]
pub fn export_entry(state: tauri::State<DatabaseState>, entry_id: i32, include_review_state: bool) {
    use crate::schema::entries;

    let conn = &mut *state.conn.lock().unwrap();
    let my_object = entry_to_json(conn, entry_id, include_review_state);

    // serialize the object to JSON
    let json = serde_json::to_string_pretty(&my_object).expect("Failed to serialize");

    let subject_name = entries::table
        .filter(entries::id.eq(entry_id))
        .select(entries::name)
        .get_result::<String>(conn)
        .expect("failed to find subject id");
    let subject_name = format!(
        "adam_{}.json", 
        subject_name.to_lowercase().replace(" ", "_")
    );

    // Create a file dialog builder
    let file_dialog = FileDialogBuilder::new()
        .add_filter("JSON files", &["json"])
        // .set_directory(desktop_dir().unwrap())
        .set_file_name(&subject_name);

    // open a file dialog
    let json_clone = json.clone();  // clone the json value
    file_dialog.save_file(move |file_path| {
        if let Some(path) = file_path {
            // create a file at the chosen path
            let mut file = File::create(&path).expect("Failed to create file");

            // write the JSON data to the file
            file.write_all(json_clone.as_bytes()).expect("Failed to write data");  // use the cloned json value
        }
    });
}



fn entry_to_json(conn: &mut SqliteConnection, entry_id: i32, include_review_state: bool) -> SubjectContents {
    use crate::schema::{entries, parents};

    let subject_name = entries::table
        .filter(entries::id.eq(entry_id))
        .select(entries::name)
        .get_result::<String>(conn)
        .expect("failed to find subject id");

    let child_deck_ids = parents::table
        .filter(parents::parent_id.eq(entry_id))
        .select(parents::child_id)
        .get_results::<i32>(conn)
        .expect("failed to get child decks");

    let mut subject = SubjectContents {
        name: subject_name,
        decks: Vec::new()
    };

    for deck_id in child_deck_ids {
        subject.decks.push(
            load_deck_contents(conn, deck_id, include_review_state)
        );
    }

    subject
}


fn load_deck_contents(conn: &mut SqliteConnection, deck_id: i32, include_review_state: bool) -> DeckContents {
    use crate::schema::{cards, entries, decks};

    let deck_name = entries::table
        .filter(entries::id.eq(deck_id))
        .select(entries::name)
        .get_result::<String>(conn)
        .expect("failed to get deck name");

    let new_per_day = decks::table 
        .filter(decks::id.eq(deck_id))
        .select(decks::new_per_day)
        .get_result::<Option<i32>>(conn)
        .expect("failed to get new_per_day");


    let mut cards: Vec<CardContents> = cards::table
        .filter(cards::deck_id.eq(deck_id))
        .select((
            cards::front,
            cards::back,
            cards::explanation,
            cards::queue_score,
            cards::repetitions,
            cards::easiness,
            cards::interval,
            cards::next_practice,
        ))
        .get_results(conn)
        .expect("Error loading cards");

    if include_review_state {
        for card in &mut cards {
            card.queue_score = None;
            card.repetitions = Some(0);
            card.easiness = Some(2.5);
            card.interval = Some(0);
            card.next_practice = Some(chrono::Local::now().date_naive());
        }
    }
    

    let deck_contents = DeckContents {
        name: deck_name,
        new_per_day: new_per_day.unwrap(),
        cards
    };

    deck_contents
}


#[tauri::command]
pub async fn select_entry() -> Option<PathBuf> {
    let (tx, rx) = oneshot::channel::<Option<PathBuf>>();
    
    // Create a file dialog builder
    let file_dialog = FileDialogBuilder::new()
        .add_filter("JSON files", &["json"]);

    // open a file dialog
    file_dialog.pick_file(move |file_path| {
        if let Some(path) = file_path.clone() {
            // Try to open the file to check if it's readable
            if File::open(&path).is_ok() {
                // Send the path using the oneshot channel
                tx.send(Some(path)).expect("Failed to send path");
            } else {
                tx.send(None).expect("Failed to send path");
            }
        } else {
            tx.send(None).expect("Failed to send path");
        }
    });

    // Await the file path. This will not block the main thread.
    let file_path = rx.await.unwrap();
    
    // Return the selected file path
    file_path
}


#[tauri::command]
pub fn import_entry(state: tauri::State<DatabaseState>, entry_path: String, include_review_state: bool, folder_id: i32) {
    let conn = &mut *state.conn.lock().unwrap();

    let entry_path = PathBuf::from(entry_path);

    // Open the file in read-only mode with buffer.
    let file = match File::open(&entry_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            return;
        }
    };
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `SubjectContents`.
    let subject_contents: Result<SubjectContents, _> = serde_json::from_reader(reader);
    match subject_contents {
        Ok(data) => {
            // Handle your data here
            write_subject_contents(conn, data, folder_id, include_review_state)
        },
        Err(e) => {
            eprintln!("Failed to deserialize JSON: {}", e);
        }
    }
}


fn write_subject_contents(conn: &mut SqliteConnection, contents: SubjectContents, parent_id: i32, include_review_state: bool) {

    let entry_type = "ankibox";

    let subject_id = insert_entry(conn, Some(parent_id), &contents.name, entry_type);
    insert_deadline(conn, subject_id, None, None, true);

    for deck in contents.decks {
        let deck_id = insert_entry(conn, Some(subject_id), &deck.name, "deck");
        insert_deck(conn, deck_id, subject_id, Some(deck.new_per_day));

        insert_imported_cards(conn, deck, include_review_state, deck_id)
        
    }

}

fn insert_imported_cards(conn: &mut SqliteConnection, contents: DeckContents, include_review_state: bool, deck_id: i32) {
    use crate::schema::cards;

    if include_review_state {
        for card in contents.cards {
            insert_into(cards::table)
                .values((
                    cards::deck_id.eq(deck_id), 
                    cards::front.eq(card.front), 
                    cards::back.eq(card.back), 
                    cards::explanation.eq(card.explanation),
                    cards::next_practice.eq(card.next_practice),
                    cards::repetitions.eq(card.repetitions), 
                    cards::interval.eq(card.interval), 
                    cards::easiness.eq(card.easiness)
                ))
                .execute(conn)
                .expect("failed to insert new cards");
        }

    } else {
        let today = chrono::Local::now().date_naive();
        for card in contents.cards {
            insert_into(cards::table)
                .values((
                    cards::deck_id.eq(deck_id), 
                    cards::front.eq(card.front), 
                    cards::back.eq(card.back), 
                    cards::explanation.eq(card.explanation),
                    cards::next_practice.eq(today),
                    cards::repetitions.eq(0), 
                    cards::interval.eq(0), 
                    cards::easiness.eq(2.5)
                ))
                .execute(conn)
                .expect("failed to insert new cards");
            
        }
    }

}

#[tauri::command]
pub fn get_new_per_day(state: tauri::State<DatabaseState>, deck_id: i32) -> i32{
    use crate::schema::decks;
    let conn = &mut *state.conn.lock().unwrap();


    decks::table   
        .filter(decks::id.eq(deck_id))
        .select(decks::new_per_day)
        .get_result::<Option<i32>>(conn)
        .expect("failed to get deck")
        .expect("failed to unwrap new_per_day")

}

#[tauri::command]
pub fn write_new_per_day(state: tauri::State<DatabaseState>, deck_id: i32, new_per_day: i32) {
    use crate::schema::decks;
    let conn = &mut *state.conn.lock().unwrap();

    update(decks::table)
        .filter(decks::id.eq(deck_id))
        .set(decks::new_per_day.eq(Some(new_per_day)))
        .execute(conn)
        .expect("failed to get new_id");
}