#[allow(dead_code)]
use tauri;
use std::{ 
    io::{BufReader, BufRead, BufWriter, Write},
    path::{ PathBuf, Path },
    collections::hash_map::DefaultHasher,
    hash::{ Hash, Hasher },
    fmt::Display,
    fs::{ File, OpenOptions, read_dir },
    str::FromStr
};
use serde::{
    Serialize, 
    Deserialize
};
use chrono::{ 
  DateTime,
  Duration,
  prelude::*,
};


  
  
/*
 * Structs
 */


// Path to folder with app data
pub struct AppDataDirState{
  pub path: Option<PathBuf>
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub struct Card {
  pub fcard: FrontendCard,
  pub md: MetaData
}

// card fields that are editable from frontend
#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub struct FrontendCard {
  pub id: usize,
  pub front: String,
  pub back: String,
  pub deck_name: String,
}

// read-only data from frontend
#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub struct MetaData {
  pub box_pos: usize,
  pub last_review: String,
}


/*
 * Date Helpers
 */

 pub fn get_days_to_go(deck_path: &PathBuf) -> i32 {
    let deadline = read_deadline(&deck_path)
      .expect("deadline not found in config");
    
    let datetime = string_to_datetime(&deadline);
    // mark of new day is 2am, and mark of counting test day is 2pm
    let days_to_go = days_until_deadline(datetime, 2, 14) as i32;
    days_to_go 
 }

/**
 * converts string in rfc3339 format to datetime
 */
pub fn string_to_datetime(string: &str) -> DateTime<FixedOffset> {

    if string.chars().count() == 25 {
        return DateTime::parse_from_rfc3339(string)
            .expect("failed to parse datetime in the rfc3339 format");
    } else {
        panic!(
            "string must have form or rfc3339 but got: {}", 
            string);
    }
}

fn get_next_datetime_at_time(dt: DateTime<FixedOffset>, time: i64) -> DateTime<FixedOffset> {
  // get time at 2am ahead of now
  let h = dt.hour() as i64;
  let m = dt.minute() as i64;
  let s = dt.second() as i64;
  let h_until_2am;
  if h < 2 {
      h_until_2am = time - h;
  } else {
      h_until_2am = 24 + time - h;
  }

  let thresh_ts = dt.checked_add_signed(
    Duration::seconds(h_until_2am as i64 * 60 * 60 - m * 60 - s)).unwrap();
  thresh_ts
}

/**
 * Returns number of days until the given datetime, counting `new_day_time` as 
 * the time marking transition between days. Returns -1 if now is past the given 
 * datetime.
 * 
 * @param datetime: date of the deadline in rfc3339 format
 * @param new_day_time: hour (0-24) at which one day switches to the next
 * @param test_day_time: hour `h` (0-24) such that if the test is after hour h 
 *                       then the test date is counted 
 */
pub fn days_until_deadline(
    datetime: DateTime<FixedOffset>,
    new_day_time: i64,
    test_day_time: i64
    ) -> i64 {
    // mark of new day: 2am
    // mark that deadline day is day 0: dl >= 2pm
    let mut day_bins: Vec<DateTime<FixedOffset>> = Vec::new();

    let mut inter = Local::now().with_timezone(&datetime.timezone());

    // get next time is at 2am
    inter = get_next_datetime_at_time(inter, new_day_time);
    
    // build up day_bins with datetimes at 2am on consecutive days
    while inter.timestamp() < datetime.timestamp() {
      day_bins.push(inter.clone());
      inter = get_next_datetime_at_time(inter, new_day_time);
    }

    if day_bins.len() == 0 {
      // on exam day
      if Local::now().timestamp() < datetime.timestamp() {
        return 0;
      }
      // past exam
      return -1;
    }

    // get deadline time in DateTime<Local>
    let dl_time = day_bins.last().unwrap().checked_add_signed(
      Duration::seconds((test_day_time - new_day_time) * 60 * 60)).unwrap();

    // time before test time does not count as a new day if it is before 2pm
    if datetime.hour() < test_day_time as u32 {
      day_bins.pop();
    }

    day_bins.push(dl_time);
    
    let now = Local::now().timestamp();

    for i in 0..day_bins.len() {
      if now < day_bins[i].timestamp() {
        let days = day_bins.len() - 1 - i;
        return days as i64;
      }
    }
    panic!("no day bins found")
}


// pub fn days_until_datetime_naive(datetime: DateTime<FixedOffset>) -> i64 {
//     datetime.signed_duration_since(Local::now()).num_days()
// }



/**
 * id of a card is hash of its deck name, front, and back fields concatenated,
 * plus the epoch time stamp of its creation in milliseconds.
 * 
 * The only purpose of this scheme is to derive a unique card id for each card
 * such that no ids collide
 */
#[tauri::command] 
pub fn calculate_hash(deck_name: String, front: String, back: String) -> u64 {
    let t = deck_name + &front + &back;
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish() + Local::now().timestamp_millis() as u64
}





/*
 * fs helpers 
 */


 // finds the index of `deck_name` in `deck_state`, None if not found
 pub fn get_deck_idx(deck_name: &String, deck_state: &Vec<PathBuf>) -> Option<usize> {
    for i in 0..deck_state.len() {
        if deck_state[i].ends_with(deck_name) {
            return Some(i);
        }
    }
    None
}

pub fn path2string(path: &PathBuf) -> String {
  path.clone().into_os_string().into_string().unwrap()
}

pub fn path2fname(path: &PathBuf) -> String {
    path.file_name().unwrap().to_owned().into_string().unwrap().to_string()
}

// get immutable path to app data
pub fn get_root_path(data_dir: tauri::State<AppDataDirState>) -> PathBuf {
  // entry refers to the name of either a deck or a directory
  data_dir.path.as_ref().unwrap().join("decks")
}

// returns vector of deck directory paths if they are children of `entry`
pub fn get_child_decks(root: &PathBuf, entry: &str) -> Vec<PathBuf> {
  read_dir(root).expect("wrong root to appdata")
    .filter_map(Result::ok)
    .filter(|f| f.path().is_dir() && f.file_name().into_string().unwrap().starts_with(entry))
    .map(|x| x.path())
    .collect::<Vec<PathBuf>>()
}

// given path to deck dir, returns value of field_name if found (otherwise None)
pub fn read_num_boxes(deck_path: &PathBuf) -> Option<i32> {
    
    let config_path = deck_path.join("config.toml");
    if !config_path.is_file() {
      return None;
    }

    let file = OpenOptions::new()
        .read(true)
        .open(config_path)
        .expect("failed to open deck cfg");
    let file = BufReader::new(file);

    for line in file.lines() {
        let line = line.expect("failed to read line from cfg");
        let mut it = line.split("=");
        let name = it.next().unwrap().trim();
        if name == "num_boxes" {
            let data = it.next()
                .expect("trying to retrieve empty field").trim();
            let data = data.parse::<i32>().expect("failed to extract value");
            return Some(data);
        }
    }
    None
}

pub fn read_deadline(deck_path: &PathBuf) -> Option<String> {

    let config_path = deck_path.join("config.toml");
    if !config_path.is_file() {
      return None;
    }

    let file = OpenOptions::new()
        .read(true)
        .open(config_path)
        .expect("failed to open deck cfg");
    let file = BufReader::new(file);

    for line in file.lines() {
        let line = line.expect("failed to read line from cfg");
        let mut it = line.split("=");
        let name = it.next().unwrap().trim();
        if name == "deadline" {
            let data = it.next()
                .expect("trying to retrieve empty field").trim();
            return Some(data.to_string());
        }
    }
    None
}

// appends `field_name = data` to file at `./decks/configs/<deck_name>-cfg.toml`,
// assuming that the field does not already exist
pub fn append_val_cfg<T>(cfg_path: &str, field_name: &str, data: T) 
  where T: Display {
  let file = OpenOptions::new()
      .append(true)
      .create(true)
      .open(cfg_path)
      .expect("failed to open deck cfg");
  let mut file = BufWriter::new(file);

  file.write_fmt(
      format_args!("{} = {}\n", field_name, data)
  ).expect("failed to append to  cfg");

}

// updates `field_name` from `./decks/configs/<deck_name>-cfg.toml` with `value`
// and appends it if not found
pub fn delete_field_cfg(cfg_path: &str, field_name: &str) {

    assert!(Path::new(cfg_path).is_file());
    let file = File::open(cfg_path).expect("failed to open cfg");
    let file = BufReader::new(file);
    let mut cfg_contents = "".to_string();

    for line in file.lines() {
        let line = line.expect("failed to read line from cfg");
        let mut it = line.split("=");
        let name = it.next().unwrap().trim();
        if name != field_name {
          cfg_contents.push_str(&(line + "\n"));
        }
    }

    // write contents in file
    let mut writer = OpenOptions::new()
        .append(true)
        .open(cfg_path)
        .expect("failed to open deck cfg");

    // write contents back into file
    writer.write_all(cfg_contents.as_bytes())
        .expect("failed to write contents");
}


// // updates `field_name` from `./decks/configs/<deck_name>-cfg.toml` with `value`
// // and appends it if not found
// pub fn update_val_cfg<T: Display>(cfg_path: &PathBuf, field_name: &str, value: T) {

//     let file = OpenOptions::new()
//         .read(true)
//         .open(&cfg_path)
//         .expect("failed to open deck cfg");
//     let file = BufReader::new(file);

//     // write contents in file
//     let writer = File::create(cfg_path)
//       .expect("failed to open deck cfg");
//     let mut writer = BufWriter::new(writer);

//     let mut found_val = false;
//     for line in file.lines() {
//         let line = line.expect("failed to read line from cfg");
//         let mut it = line.split("=");
//         let name = it.next().unwrap().trim();
//         let value = it.next().unwrap().trim();
//         writer.write_fmt(format_args!("{} = {}\n", name, value))
//           .expect("failed to write line");

//         if !found_val && name == field_name {
//             found_val = true;
//         }
//     }

//     // write contents back into file
//     if !found_val {
//         writer.write_fmt(format_args!("{} = {}\n", field_name, value))
//             .expect("failed to append value");
//     }
// }

// reads the value of `field_name` from `./decks/configs/<deck_name>-cfg.toml`
pub fn read_from_cfg(cfg_path: &PathBuf, field_name: &str) -> Option<String> {

    let file = OpenOptions::new()
        .read(true)
        .open(cfg_path)
        .expect("failed to open deck cfg");
    let file = BufReader::new(file);

    for line in file.lines() {
        let line = line.expect("failed to read line from cfg");
        let mut it = line.split("=");
        let name = it.next().unwrap().trim();
        if name == field_name {
            let data = it.next()
                .expect("trying to retrieve empty field").trim();
            return Some(data.to_string());
        }
    }
    None
}


/*
 * Algo helpers
 */

pub fn get_num_boxes(days_to_go: i64) -> i32 {
  let t = days_to_go as i32;
  // bins generated by recursive equation x_n = x_{n-1} + 2^n + 1 
  // applied 5 times to (2, 6)
  let bins = vec![
        (0, 1), (2, 6), (7, 15), (16, 32), (33, 65), (66, 130), (131, 259)];
  let mut i = 0;
    let mut found_bin = false;
  for (a, b) in bins {
    if a <= t && t <= b {
            found_bin = true;
      break;
    }
    i += 1;
  }
  assert!(found_bin, "deadline must be between 0 and 259 days in the future");
  let num_boxes = 2 + i;
  num_boxes
}


#[derive(Debug)]
pub struct QuotasRecord {
  pub dtg: i32, // days_to_go
  pub nq: i32,  // new_quota
  pub rq: i32,  // review_quota
  pub nqp: i32, // new_quota_practiced
  pub rqp: i32  // review_quota_practiced
}

// returns records of quotas, sorted with ascending dtg 
pub fn read_quotas_file(quotas_path: &PathBuf) -> Vec<QuotasRecord> {

    let reader = OpenOptions::new().read(true).open(quotas_path)
      .expect("failed to read quotas path");
    let reader = BufReader::new(reader);

    let mut quotas: Vec<QuotasRecord> = Vec::new();
    for line in reader.lines().skip(1) {
        let line = line.expect("failed to read line");

        let mut line_it = line.split(',')
          .map(|x| x.parse::<i32>()
          .expect("quotas file is improperly formatted. must be csv."));
     
        quotas.push(
          QuotasRecord {
            dtg: line_it.next().expect("failed to read quotas csv"),
            nq: line_it.next().expect("failed to read quotas csv"),
            rq: line_it.next().expect("failed to read quotas csv"),
            nqp: line_it.next().expect("failed to read quotas csv"),
            rqp: line_it.next().expect("failed to read quotas csv"),
        });
     
    }
    quotas
}

// write [[dtg, nq, rq, nqp, rqp], ...] to ./decks/quotas/deckname.csv
pub fn write_quotas_file(quotas: &Vec<QuotasRecord>, quotas_path: &PathBuf) {
  // compute (nq, rq) doubles for each day
  let buf = OpenOptions::new()
      .write(true)
      .truncate(true)
      .open(quotas_path)
      .expect("failed to create quota file");
  let mut buf = BufWriter::new(buf);

  let header = "DaysToGo,NewQuota,ReviewQuota,NewPracticed,ReviewPracticed\n";
  buf.write_all(header.as_bytes()).expect("Unable to write data");

  for d in 0..quotas.len() {
      buf.write_fmt(format_args!(
          "{},{},{},{},{}\n",
           quotas[d].dtg, 
           quotas[d].nq, 
           quotas[d].rq, 
           quotas[d].nqp, 
           quotas[d].rqp
          )
      ).expect("failed to write");
  }
}

// redistributes quotas, based on assuming new reviews are twice as expensive
// quotas has form (nq, rq); i days to go can correspond either to index
// quotas.len() - 1 - i or index i
pub fn redistribute_quotas(quotas: &mut [QuotasRecord]){

  assert!(quotas[0].dtg == 0);

  //  a quantification of effort, score S := 2NQ + RQ
  let score_tot: i32 = compute_study_cost(quotas).iter().sum();
  let score_avg = score_tot / quotas.len() as i32;

  // don't want to modify index 0, so temporarily give it avg score
  let n_cards = quotas[0].rq;
  quotas[0].rq = score_avg;

  loop {
      let scores: Vec<i32> = compute_study_cost(quotas);
      if !scores.iter().any(|&x| x - score_avg >= 4) {
          break;
      }

      // if any days have a score greater than 3
      for i in (0..quotas.len()).rev() {
          let scores: Vec<i32> = compute_study_cost(quotas);
          if scores[i] - score_avg >= 4 {
              // give one NQ away to the min
              let min_idx = argmin(&scores);
              if quotas[i].nq > 0 {
                  quotas[i].nq -= 1;
                  quotas[min_idx].nq += 1;
              }

              // give one RQ away to the min twice
              let scores = compute_study_cost(quotas);
              let min_idx = argmin(&scores);
              if quotas[i].rq > 0 {
                  quotas[i].rq -= 1;
                  quotas[min_idx].rq += 1;
              }

              let scores = compute_study_cost(quotas);
              let min_idx = argmin(&scores);
              if quotas[i].rq > 0 {
                  quotas[i].rq -= 1;
                  quotas[min_idx].rq += 1;
              }
          }
      }
  }

  // reset index 0
  quotas[0].rq = n_cards;
}

fn compute_study_cost(quotas: &[QuotasRecord]) -> Vec<i32> {
    let scores: Vec<i32> = quotas
        .iter()
        .map(|x| 2*x.nq + x.rq) //"new cards are twice as hard as review cards"
        .collect();
    scores
}

fn argmin(collection: &Vec<i32>) -> usize {
    let min = collection.iter().min().unwrap();
    let argmin = collection.iter().position(|x| x == min)
        .expect("failed to find argmin");
    argmin
} 