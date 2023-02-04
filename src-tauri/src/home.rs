#![allow(unused_imports)]
#![allow(dead_code)]
use tauri::{self, utils::config::parse::read_from};
use tauri::State;

use serde::{ Serialize, Deserialize };
use serde_json::{Result, Value, json};

use fs_extra::dir::{
    move_dir,
    CopyOptions
};

use std::fs::remove_file;
use std::{
    error::Error,
    fs::{ File, create_dir, read_dir, remove_dir_all, rename, create_dir_all},
    io::{BufReader, BufWriter, Write},
    path::{PathBuf, Path}, 
};

use crate::utils::{
    AppDataDirState,
    append_val_cfg,
    path2string, 
    read_from_cfg, 
    get_num_boxes, 
    get_days_to_go,
    string_to_datetime,
    days_until_deadline,
    delete_field_cfg
};

use crate::review::{
    get_todays_quotas,
    Quotas
};

use chrono::{Local, Timelike, Month};


#[derive(Serialize, Deserialize, Debug)]
pub struct FileSystemObject {
    pub entity_type: String,
    pub name: String,
    pub files: Option<Vec<FileSystemObject>>,
    pub expanded: Option<bool>,
    pub deadline_date: Option<String>,
    pub deadline_time: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    is_dark_mode: bool,
    is_textfield: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EntryQuota {
    new_left: i32,
    review_left: i32,
    num_progressed: i32,
    days_to_go: i32,
    tot_days: i32,
    deck_path: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PbarData {
    start_date: String,
    end_date: String,
    curr_timestamp: i64,
    end_timestamp: i64,
    days_to_go: i64,
}


/**
 * Read and write global config and folder structure
 */


#[tauri::command] 
pub fn read_global_config(data_dir: State<AppDataDirState>) -> AppConfig { 
    let path = data_dir.path.as_ref().unwrap().join("folders").join("global-config.toml");

    // init global config if hasn't been created yet
    if !path.is_file() {
        File::create(&path).expect("failed to open fs json");
        let path_str = path2string(&path);
        append_val_cfg(&path_str, "is_dark_mode", "false");
        append_val_cfg(&path_str, "is_textfield", "false");
    }

    let dark_mode = read_from_cfg(&path, "is_dark_mode")
        .expect("failed to read dark mode");
    let textfield = read_from_cfg(&path, "is_textfield")
        .expect("failed to read textfield");

    let config = AppConfig {
        is_dark_mode:  dark_mode == "true",
        is_textfield: textfield == "true"
    };
    
    config
}

#[tauri::command] 
pub fn write_global_config(data_dir: State<AppDataDirState>, config: AppConfig) { 
    let path = data_dir.path.as_ref().unwrap().join("folders").join("global-config.toml");
    File::create(&path)
        .expect("failed to truncate global config");

    let path_str = path2string(&path);
    append_val_cfg(&path_str, "is_dark_mode", config.is_dark_mode);
    append_val_cfg(&path_str, "is_textfield", config.is_textfield);
}


#[tauri::command] 
pub fn read_fs_json(data_dir: State<AppDataDirState>) -> FileSystemObject {
    let path = data_dir.path.as_ref().unwrap().join("folders").join("fs.json");
    assert!(path.is_file(), "could not find fs.json");

    // Open the file in read-only mode with buffer.
    let file = File::open(path).expect("failed to open fs json");
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let json: FileSystemObject = serde_json::from_reader(reader)
        .expect("failed to read fs.json because it's not correctly formatted");

    json
}

#[tauri::command] 
pub fn write_fs_json(data_dir: State<AppDataDirState>, fs: Vec<FileSystemObject>) {
    let path = data_dir.path.as_ref().unwrap().join("folders").join("fs.json");
    assert!(path.is_file(), "could not find fs.json");

    // Open the file in read-only mode with buffer.
    let file = File::create(path).expect("failed to open fs json");
    let mut writer = BufWriter::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    assert!(fs.len() == 1, "found multiple root folders. current fs does not support");
    let fs_string = serde_json::to_string_pretty(&fs[0])
        .expect("failed to read fs.json because it's not correctly formatted");

    writer.write_all(
        fs_string.as_bytes()
    ).expect("failed to write json");
}


/**
 * Create a deadline in `/folders/deadlines.toml`
 */

// invoke("create_deadline", { path, newName, deadlineDate, deadlineTime });
#[tauri::command] 
pub fn create_deadline(
    data_dir: State<AppDataDirState>, 
    path: String,
    new_name: String, 
    deadline_date: String, // YYYY-MM-DD format
    deadline_time: String // HH:MM format
) {
    
    let cfg_path = data_dir.path.as_ref().unwrap()
        .join("folders")
        .join("deadlines");

    if !cfg_path.is_dir() {
        create_dir_all(&cfg_path).expect("failed to create dirs");
    }

    let deadline_name = path + "~~" + &new_name;
    let dl_cfg = cfg_path.join(deadline_name + ".toml");
    let dl_cfg = path2string(&dl_cfg);

    let deadline = time_input2deadline(deadline_date, deadline_time);
    let date_created = Local::now()
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    append_val_cfg(&dl_cfg, "deadline", deadline);
    append_val_cfg(&dl_cfg, "date_created", date_created);
    append_val_cfg(&dl_cfg, "num_reset", 0);     // number of times deadline is reset
    append_val_cfg(&dl_cfg, "mastery_level", 0); // (num_boxes + m), so m = -1, 0, 1
}

 // converts `deadline_date` (YYYY-MM-DD format) and `deadline_time` (HH:MM format)
 // to rfc3339 format
fn time_input2deadline(deadline_date: String, deadline_time: String) -> String {
    let now = Local::now();

    // extract time zone
    let now_str = now.to_rfc3339();
    let mut tz = "".to_string();
    for i in (1..7).rev() {
        tz.push(now_str.chars().nth(now_str.len() - i).unwrap());
    }

    // extract date
    let mut deadline_date = deadline_date.split("-");
    let yyyy = deadline_date.next().expect("invalid deadline date");
    let mm = deadline_date.next().expect("invalid deadline date");
    assert!(mm.len() == 2);
    let dd = deadline_date.next().expect("invalid deadline date");
    assert!(dd.len() == 2);

    // extract time
    let mut deadline_time = deadline_time.split(":");
    let hh = deadline_time.next().expect("invalid deadline time");
    assert!(hh.len() == 2);
    let min = deadline_time.next().expect("invalid deadline time");
    assert!(min.len() == 2);

    // put all info into an rfc3339 string YYYY-MM-DDTHH:MM:SS-TZ:TZ
    let deadline = yyyy.to_string() + "-" + mm + "-" + dd + "T" + hh + ":" + min + ":00" + &tz;

    deadline
    
}



/**
 * Initializing deck diretory and writing deadline to deck config
 */
#[tauri::command] 
pub fn create_deck(data_dir: State<AppDataDirState>, path: String, new_name: String) {

    // get path to deck directory and create dir|preventDefault
    let root = data_dir.path.as_ref().unwrap();
    let conf_path = root
        .join("folders").join("deadlines").join(path.clone() + ".toml");
    assert!(conf_path.is_file(), 
        "could not find deadlines.toml at {}", conf_path.to_str().unwrap());

    
    // read deadline from `folders/deadlines.toml`
    let deadline = read_from_cfg(&conf_path, "deadline")
        .expect("did not find deadline");
    
    // get metadata used to compute quotas
    let datetime = string_to_datetime(&deadline);
    // mark of new day is 2am, and mark of counting test day is 2pm
    let days_to_go = days_until_deadline(datetime, 2, 14);

    // compute numBoxes and other deck metadata
    let num_boxes = get_num_boxes(days_to_go);
    let date_created = Local::now()
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);


    // create deck directory with subdir containing images
    let deck_path = root.join("decks").join(path + "~~" + &new_name);

    // remove contents of dir if exists (should not be needed but as a safeguard)
    if deck_path.is_dir() {
        remove_dir_all(&deck_path).expect("failed to remove deck dir");
    }

    create_dir(&deck_path).expect("failed to create dir to path");
    create_dir(&deck_path.join("images")).expect("failed to create dir");

    File::create(deck_path.join("cards.csv"))
        .expect("failed to create file");
    File::create(deck_path.join("quotas.csv"))
        .expect("failed to create file");

    let mut readme = File::create(deck_path.join("readme.md"))
        .expect("failed to create file");
    let opening_message = "Welcome, Cardwegian!";
    readme.write_all(opening_message.as_bytes())
        .expect("failed to write readme");
    
    // write deadline and metadata to `config.toml` specific to the deck
    let deck_conf_path = deck_path.join("config.toml");
    let deck_conf_path = path2string(&deck_conf_path);
    append_val_cfg(&deck_conf_path, "num_boxes", num_boxes);
    append_val_cfg(&deck_conf_path, "days_to_go", days_to_go);
    append_val_cfg(&deck_conf_path, "deadline", deadline);
    append_val_cfg(&deck_conf_path, "date_created", date_created);

}

/**
 * Handle renaming decks
 */

#[tauri::command] 
pub fn rename_entry(data_dir: State<AppDataDirState>, path: String, old_path: String) { 

    // get path to deck directory and create dir
    let root = data_dir.path.as_ref().unwrap();

    // read deadline of old entry
    let dl_root = root.join("folders").join("deadlines");
    let old_dl_path = dl_root.join(old_path.clone() + ".toml");
    if old_dl_path.is_file() {
        let new_dl_path = dl_root.join(old_path.clone() + ".toml");
        rename(old_dl_path, new_dl_path)
            .expect("failed to rename deadline file");
    }

    
    // rename 
    let deck_root = root.join("decks");
    let deck_dirs = read_dir(&deck_root).expect("failed to read dirs");
    let old_path = path2string(&deck_root.join(old_path));

    for dir in deck_dirs {
        let dir = path2string(&dir.unwrap().path());

        // modify paths of all decks to account for change of entry name
        if dir.starts_with(&old_path) {
            // get new name of deck; replacing prefix with that of renamed entry
            let new_dir = deck_root
                .join(&path).join(&dir.strip_prefix(&old_path).unwrap());

            // rename deck
            rename(dir, new_dir).expect("failed to rename {dir}");
        }
    }

}


/**
 * Delete entry and all entries decendent from it by moving decks into 
 * folder of deleted decks.
 */

#[tauri::command] 
pub fn delete_entry(data_dir: State<AppDataDirState>, path: String) { 

    // get path to deck directory and create dir
    let root = data_dir.path.as_ref().unwrap();
    let dl_root = root.join("folders").join("deadlines");
    let dls = read_dir(&dl_root).expect("failed to read deadline config root");

    // if entry is deadline, remove its file
    let dl_path = dl_root.join(path.clone() + ".toml");

    for dl in dls {
        let dl = dl.expect("failed to unwrap deadline");
        if &dl.path().as_os_str() == &dl_path.as_os_str() {
            remove_file(&dl_path).expect("failed to remove deadline file");
        }
    }

    // read all deck files
    let deck_root = root.join("decks");
    let deck_path_to_delete = deck_root.join(path);
    if deck_path_to_delete.is_file() {
        remove_file(&deck_path_to_delete).expect("failed to remove deadline file");
    }
}

#[tauri::command] 
pub fn get_deck_quotas(data_dir: State<AppDataDirState>, deck_paths: Vec<String>
    ) -> Vec<EntryQuota> {

    // get path to deck directory and create dir
    let root = data_dir.path.as_ref().unwrap();
    let deck_root = root.join("decks");
    assert!(deck_root.is_dir(), 
    "could not find root at {}", deck_root.to_str().unwrap());

    let mut paths = Vec::new();
    for path in &deck_paths {
        paths.push((&deck_root.join(path)).clone());
    }

    let quotas: Vec<Quotas> = get_todays_quotas(&paths);
    let mut equotas: Vec<EntryQuota> = Vec::new();
    
    for i in 0..deck_paths.len() {

        equotas.push(
            EntryQuota {
                new_left: quotas[i].new_left,
                review_left: quotas[i].review_left,
                num_progressed: quotas[i].num_progressed,
                days_to_go: 0,
                tot_days: 0,
                deck_path: deck_paths[i].clone()
            }
        );
    }

    equotas
}

#[tauri::command] 
pub fn get_deadline_progress(data_dir: State<AppDataDirState>, deadline_name: String) -> PbarData {
    let root = data_dir.path.as_ref().unwrap();
    let dl_root = root.join("folders").join("deadlines");
    let dl_path = dl_root.join(deadline_name.clone() + ".toml");

    assert!(dl_path.is_file(), "deadline not found {}", &dl_path.to_str().unwrap());

    let deadline = read_from_cfg(&dl_path, "deadline")
        .expect("failed to read");
    let date_created = read_from_cfg(&dl_path, "date_created")
        .expect("failed to read");

    let deadline_dt = string_to_datetime(&deadline);
    let date_created_dt = string_to_datetime(&date_created);
    
    let deadline_ts = deadline_dt.timestamp();
    let start_ts = date_created_dt.timestamp();


    // get now timestamp with timezone info
    let now = Local::now()
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true); // YYYY-MM-DD
    let now_dt = string_to_datetime(&now);
    let curr_ts = now_dt.timestamp();

    let start_date = rfc2mmdd(date_created);
    let end_date = rfc2mmdd(deadline);

    let days_to_go = days_until_deadline(deadline_dt, 2, 14);

    PbarData {
        start_date,
        end_date,
        curr_timestamp: curr_ts - start_ts,
        end_timestamp: deadline_ts - start_ts,
        days_to_go,
    }
}

fn rfc2mmdd(rfc_date: String) -> String {
    let mut now_it = rfc_date.split("-").skip(1);
    let now_mm = now_it.next().expect("failed to parse current month");
    // for later: convert to month name
    // let now_mm = Month::from_u32(now_mm.parse::<u32>().unwrap());
    let now_dd = now_it.next().expect("failed to parse current day");
    let now_dd = now_dd.split("T").next().expect("failed to parse day");

    let curr_date = "".to_string() + now_mm + "-" + now_dd;
    curr_date
}