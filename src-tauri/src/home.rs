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

use std::{
    error::Error,
    fs::{ File, create_dir, read_dir, rename},
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

use chrono::{Local, Timelike};


#[derive(Serialize, Deserialize, Debug)]
pub struct FileSystemObject {
    pub entity_type: String,
    pub name: String,
    pub files: Option<Vec<FileSystemObject>>,
    pub expanded: Option<bool>,
    pub deadline_date: Option<String>,
    pub deadline_time: Option<String>
}



/**
 * Read and write folder structure
 */

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
    assert!(fs.len() == 1, "fount multiple root folders. current fs does not support");
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
        .join("deadlines.toml");
    let cfg_path = path2string(&cfg_path);
    let deadline_name = path + "~~" + &new_name;

    let deadline = time_input2deadline(deadline_date, deadline_time);


    append_val_cfg(&cfg_path, &deadline_name, deadline);

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

    // get path to deck directory and create dir
    let root = data_dir.path.as_ref().unwrap();
    let conf_path = root.join("folders").join("deadlines.toml");
    assert!(conf_path.is_file(), 
        "could not find deadlines.toml at {}", conf_path.to_str().unwrap());

    let dl_name = &path;
    
    // read deadline from `folders/deadlines.toml`
    let deadline = read_from_cfg(&conf_path, &dl_name)
        .expect("did not find deadline");
    
    // get metadata used to compute quotas
    let datetime = string_to_datetime(&deadline);
    // mark of new day is 2am, and mark of counting test day is 2pm
    let days_to_go = days_until_deadline(datetime, 2, 14);

    // compute numBoxes and other deck metadata
    let num_boxes = get_num_boxes(days_to_go);


    // create deck directory with subdir containing images
    let deck_path = root.join("decks").join(path + "~~" + &new_name);
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

}

/**
 * Handle renaming decks
 */

#[tauri::command] 
pub fn rename_entry(data_dir: State<AppDataDirState>, path: String, old_path: String) { 

    // get path to deck directory and create dir
    let root = data_dir.path.as_ref().unwrap();
    let conf_path = root.join("folders").join("deadlines.toml");
    assert!(conf_path.is_file(), 
        "could not find deadlines.toml at {}", conf_path.to_str().unwrap());

    // read deadline of old entry
    let deadline = read_from_cfg(&conf_path, &old_path);

    // if entry is a deadline, append new deadline in file
    if let Some(dl) = deadline {
        let dl_conf_path = &path2string(&conf_path);
        delete_field_cfg(dl_conf_path, &old_path);
        append_val_cfg(dl_conf_path, &path, dl);
    }
    
    // rename 
    let deck_dir = root.join("decks");
    let deck_dirs = read_dir(&deck_dir).expect("failed to read dirs");

    let old_path = path2string(&deck_dir.join(old_path));
    let path = path2string(&deck_dir.join(path));



    for dir in deck_dirs {
        let dir = path2string(&dir.unwrap().path());
        if dir.starts_with(&old_path) {
            // get new name of deck
            let new_dir = "".to_owned() + &path + 
                &dir.strip_prefix(&old_path).unwrap();

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
    let conf_path = root.join("folders").join("deadlines.toml");
    assert!(conf_path.is_file(), 
        "could not find deadlines.toml at {}", conf_path.to_str().unwrap());


    // if entry is a deadline, append new deadline in file
    let dl_conf_path = path2string(&conf_path);
    delete_field_cfg(dl_conf_path.as_str(), &path);

    // move decks descendant of the entry to "deleted" directory
    let deleted_dir = root.join("deleted");
    if !deleted_dir.is_dir() {
        create_dir(&deleted_dir).expect("failed to create dir of deleted decks");
    }
    
    // read all deck files
    let deck_dir = root.join("decks");
    let deck_dirs = read_dir(&deck_dir).expect("failed to read dirs");

    // absolute path to deleted entry, including root path
    let path = path2string(&deck_dir.join(path));
    let deleted_dir = path2string(&deleted_dir);

    //Initialize default values for CopyOptions
    let options = CopyOptions::new();

    for dir in deck_dirs {
        let dir = path2string(&dir.unwrap().path());
        if dir.starts_with(&path) {
            move_dir(dir, &deleted_dir, &options)
                .expect("failed to move deck into directory of deleted items");
        }
    }
}
