use std::{ fs, fmt, path::{ Path, PathBuf } };
use chrono::{ DateTime, prelude::* };

use crate::mio0::utils::{ read_from_cfg, deadline_to_datetime, days_until };

#[derive(Clone, Debug)]
pub struct DeckEntry {
    pub deck_name: String,        // without extension (e.g., "empty")
    pub deck_path_buf: PathBuf,
    pub deadline: DateTime<FixedOffset>,
    pub days_to_go: i64,
}

pub fn print_decks() {
    let decks_path = "./decks";
    let dir = Path::new(decks_path);
    if !dir.is_dir() {
        panic!("cd to the main directory `mio0` to use program");
    }

    // print header of table
    println!("{0: <20} {1: <20} {2: <20}", "Deck Name", "Deadline Y-M-D", "Days To Go");

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let deck_entry = DeckEntry::new_from_path(path);
        println!("{deck_entry}");
    }
}

impl DeckEntry {
    // Try to create a deck entry from a deck name (without extension)
    // handles search for deck file from user-provided `deck_name`
    pub fn new_from_name(deck_name: &str) -> Option<Self> {
        // Read in possible deck directory
        let decks_path = "./decks";
        let dir = Path::new(decks_path);
        if !dir.is_dir() {
            panic!("you must be in the main `mio0` directory to use the program");
        }
        
        let deck_path = dir.join(deck_name.to_string() + ".csv");
        if !deck_path.is_file() {
            return None;
        }
        
        // [!] WORK ON THIS
        let timestamp_str = read_from_cfg(
            deck_name, "deadline").expect("deadline not in cfg");
        let deadline = deadline_to_datetime(timestamp_str);

        let days_to_go = days_until(deadline);

        return Some(DeckEntry {
            deck_name: deck_name.to_string(),
            deck_path_buf: deck_path,
            deadline,
            days_to_go,
        });
    }

    // create a deck entry from a path buffer (path buffer found internally)
    pub fn new_from_path(path: PathBuf) -> Self {
        let decks_path = "./decks";
        if !path.is_file() || !path.extension().unwrap().eq("csv") {
            panic!("invalid path to deck");
        }

        // convert path to file name without extension (e.g., `test`)
        let deck_name = path
            .strip_prefix(decks_path)
            .unwrap()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let timestamp_str = read_from_cfg(
            &deck_name, "deadline").expect("deadline not in cfg");
        let naivedatetime_utc = deadline_to_datetime(timestamp_str);
        let duration_remaining = days_until(naivedatetime_utc);

        DeckEntry {
            deck_name,
            deck_path_buf: path,
            deadline: naivedatetime_utc,
            days_to_go: duration_remaining,
        }
    }
    
    pub fn get_quotas_path(&self) -> PathBuf {
        self.deck_path_buf.parent().unwrap()
            .join("quotas")
            .join("".to_owned() + &self.deck_name + "-quotas.csv")
    }

    // pub fn get_configs_path(&self) -> PathBuf {
    //     self.deck_path_buf.parent().unwrap()
    //         .join("configs")
    //         .join("".to_owned() + &self.deck_name + "-configs.csv")
    // }
}

impl fmt::Display for DeckEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let deadline_string = self.deadline.format("%Y-%m-%d %H:%M");
        write!(f, "{0: <20} {1: <20} {2: <20}", self.deck_name, deadline_string, self.days_to_go)
    }
}
