use std::{
    collections::hash_map::DefaultHasher,
    hash::{ Hash, Hasher },
    iter::zip,
    fs::OpenOptions,
    io::{ BufWriter, BufReader, prelude::* },
    fmt::{ Display, Debug },
    str::FromStr,
    path::{ Path, PathBuf },
};

use chrono::{ 
    NaiveDate, 
    DateTime,
    prelude::*,
};


// appends `field_name = data` to file at `./decks/configs/<deck_name>-cfg.toml`,
// assuming that the field does not already exist
pub fn append_val_cfg<T>(deck_name: &str, field_name: &str, data: T) 
    where T: Display {

    
    let cfg_path = "./decks/configs/".to_string() + deck_name + "-cfg.toml";
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

// reads the value of `field_name` from `./decks/configs/<deck_name>-cfg.toml`
pub fn read_from_cfg<T: FromStr>(deck_name: &str, field_name: &str) -> Option<T>
    where <T as FromStr>::Err: Debug 
    {

    let cfg_path = "./decks/configs/".to_string() + deck_name + "-cfg.toml";
    let cfg_path = Path::new(&cfg_path);

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
            let data = T::from_str(data).expect("failed to extract value");
            return Some(data);
        }
    }
    None
}

// updates `field_name` from `./decks/configs/<deck_name>-cfg.toml` with `value`
// and appends it if not found
pub fn update_val_cfg<T: Display>(deck_name: String, field_name: String, value: T) {

    let cfg_path = Path::new(&cfg_path);
    let file = OpenOptions::new()
        .read(true)
        .open(&cfg_path)
        .expect("failed to open deck cfg");
    let file = BufReader::new(file);
    let mut cfg_contents = "".to_string();

    let mut found_val = false;
    for line in file.lines() {
        let line = line.expect("failed to read line from cfg");
        let mut it = line.split("=");
        let name = it.next().unwrap().trim();
        if !found_val && name == field_name {
            cfg_contents.push_str(&format!("{} = {}\n", name, value));
            found_val = true;
        } else {
            cfg_contents.push_str(&line);
        }
    }

    // write contents in file
    let mut writer = OpenOptions::new()
        .truncate(true)
        .append(true)
        .open(cfg_path)
        .expect("failed to open deck cfg");

    // write contents back into file
    writer.write_all(cfg_contents.as_bytes())
        .expect("failed to write contents");
    // append field if not found
    if !found_val {
        writer.write_fmt(format_args!("{} = {}\n", field_name, value))
            .expect("failed to append value");
    }
}
// 
// // increment `field_name` in `./decks/configs/<deck_name>-cfg.toml` by `value`
// // (differs from update_val_cfg because supports only i32 and adds instead of 
// // overwriting val)
// pub fn increment_val_cfg(deck_name: String, field_name: String, value: i32) {
// 
//     let cfg_path = "./decks/configs/".to_string() + &deck_name + "-cfg.toml";
//     let cfg_path = Path::new(&cfg_path);
//     let file = OpenOptions::new()
//         .read(true)
//         .open(&cfg_path)
//         .expect("failed to open deck cfg");
//     let file = BufReader::new(file);
//     let mut cfg_contents = "".to_string();
// 
//     let mut found_val = false;
//     for line in file.lines() {
//         let line = line.expect("failed to read line from cfg");
//         let mut it = line.split("=");
//         let name = it.next().unwrap().trim();
//         if !found_val && name == field_name {
//             let curr_value: i32 = it.next().unwrap().parse()
//                 .expect("failed to parse value");
// 
//             cfg_contents.push_str(
//                 &format!("{} = {}\n", name, curr_value + value)
//             );
//             found_val = true;
//         } else {
//             cfg_contents.push_str(&line);
//         }
//     }
// 
//     // write contents in file
//     let mut writer = OpenOptions::new()
//         .truncate(true)
//         .append(true)
//         .open(cfg_path)
//         .expect("failed to open deck cfg");
// 
//     // write contents back into file
//     writer.write_all(cfg_contents.as_bytes())
//         .expect("failed to write contents");
//     // append field if not found
//     if !found_val {
//         panic!("value not found")
//     }
// }

pub fn deadline_to_datetime(deadline_string: String) -> DateTime<FixedOffset> {
    if deadline_string.chars().count() == 25 {
        return DateTime::parse_from_rfc3339(&deadline_string)
            .expect("failed to parse datetime in the rfc3339 format");
    } else {
        panic!(
            "deadline string must have form or rfc3339 but got: {}", 
            deadline_string);
    }
}

fn local_to_fixed(local_date_time: DateTime<Local>) -> DateTime<FixedOffset> {
    local_date_time.with_timezone(local_date_time.offset())
}

fn naive_deadline_to_datetime(deadline_string: String) -> DateTime<Local> {
    if deadline_string.chars().count() != 16 {
        panic!("deadline string must be in form YYYY-MM-DD-HH:MM but got: {}",
            deadline_string);
    }
    let len = deadline_string.chars().count();
    let minute: u32 = deadline_string[len - 2..len].parse().ok().expect("Invalid Deadline Format");
    let hour: u32 = deadline_string[len - 5..len - 3]
        .parse()
        .ok()
        .expect("Invalid Deadline Format");
    let day: u32 = deadline_string[len - 8..len - 6].parse().ok().expect("Invalid Deadline Format");
    let month: u32 = deadline_string[len - 11..len - 9]
        .parse()
        .ok()
        .expect("Invalid Deadline Format");
    let year: i32 = deadline_string[len - 16..len - 12]
        .parse()
        .ok()
        .expect("Invalid Deadline Format");

    // convert to local time zone
    let local_dt = Local.from_local_datetime(
        &NaiveDate::from_ymd_opt(year, month, day).unwrap()
        .and_hms_opt(hour, minute, 0).unwrap()
    ).unwrap();
    local_dt
}

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

// returns vector containing `days_to_go + 1` `(nq, rq)` pairs
// note that day_to_go i corresponds with index (len - 1) - i for the returned vector
pub fn compute_quotas(num_cards: i32, days_to_go: i64, num_boxes: i32) -> Vec<(i32, i32)> {

    let n = num_cards;          // number of cards                                                           
    let t = days_to_go as i32;  // days until deadline                                          
    let b = num_boxes;          // number of boxes
                                                                                   
    let sum: i32 = (0..t).sum();
    
    // compute new card quota vector
    let mut nq: Vec<i32> = (0..t).rev().map(|x| x * n / sum).collect();
    let nq_sum = n;

    // enforce sum of NQ equals number of cards by adding remainder
    if let Some(first) = nq.get_mut(0) {
        *first += n % nq_sum;
    }
                                                                                   
    
    // compute review card quota vector
    let mut rq: Vec<i32> = (0..t).map(|x| x * n * (b - 2) / sum).collect();                             
    let rq_sum = n * (b - 2);

    // enforce sum of RQ equals number of cards times number of bins minus 2
    if let Some(last) = rq.last_mut() {
        *last += (n * (b - 2)) % rq_sum;
    }

    // zip two vectors into vector of doubles [(nq_i, rq_i)]_{i=0}^{T-1}
    let mut zipped: Vec<(i32, i32)> = zip(
        nq.into_iter(),
        rq.into_iter(),
    ).collect();

    // user reviews all cards the day of exam
    zipped.push((0, n));
    zipped
}

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

fn reverse_order<T: Copy>(coll: &mut Vec<T>) {
    let mut j = coll.len() - 1;
    let mut i = 0;
    while i <= j {
        let temp = coll[i];
        coll[i] = coll[j];
        coll[j] = temp;

        j -= 1;
        i += 1;
    }
}

// redistributes quotas, based on assuming new reviews are twice as expensive
// quotas has form (nq, rq); i days to go can correspond either to index
// quotas.len() - 1 - i or index i
pub fn redistribute_quotas(quotas: &mut Vec<(i32, i32)>){
    let reversed = quotas[0].0 != 0;
    if reversed {
        reverse_order(quotas);
    }

    //  a quantification of effort, score S := 2NQ + RQ
    let score_tot: i32 = compute_study_cost(&quotas).iter().sum();
    let score_avg = score_tot / quotas.len() as i32;

    // don't want to modify index 0, so temporarily give it avg score
    assert!(quotas[0].0 == 0, "day 0 is allotted a new card");
    let n_cards = quotas[0].1;
    quotas[0].1 = score_avg;

    loop {
        let scores: Vec<i32> = compute_study_cost(&quotas);
        if !scores.iter().any(|&x| x - score_avg >= 4) {
            break;
        }

        // if any days have a score greater than 3
        for i in (0..quotas.len()).rev() {
            let scores: Vec<i32> = compute_study_cost(&quotas);
            if scores[i] - score_avg >= 4 {
                // give one NQ away to the min
                let min_idx = argmin(&scores);
                if quotas[i].0 > 0 {
                    quotas[i].0 -= 1;
                    quotas[min_idx].0 += 1;
                }

                // give one RQ away to the min twice
                let scores = compute_study_cost(&quotas);
                let min_idx = argmin(&scores);
                if quotas[i].1 > 0 {
                    quotas[i].1 -= 1;
                    quotas[min_idx].1 += 1;
                }

                let scores = compute_study_cost(&quotas);
                let min_idx = argmin(&scores);
                if quotas[i].1 > 0 {
                    quotas[i].1 -= 1;
                    quotas[min_idx].1 += 1;
                }
            }
        }
    }

    // reset index 0
    quotas[0].1 = n_cards;

    if reversed {
        reverse_order(quotas);
    }
}

fn compute_study_cost(quotas: &Vec<(i32, i32)>) -> Vec<i32> {
    let scores: Vec<i32> = quotas
        .iter()
        .map(|x| 2*x.0 + x.1) //"new cards are twice as hard as review cards"
        .collect();
    scores
}

fn argmin(collection: &Vec<i32>) -> usize {
    let min = collection.iter().min().unwrap();
    let argmin = collection.iter().position(|x| x == min)
        .expect("failed to find argmin");
    argmin
} 

// returns lines of quotas in vector form [[dtg, nq, rq, nqp, rqp], ...]
pub fn read_quotas_file(quotas_path: &PathBuf) -> Vec<Vec<i32>> {

    let reader = OpenOptions::new().read(true).open(quotas_path).unwrap();
    let reader = BufReader::new(reader);

    let mut quotas: Vec<Vec<i32>> = Vec::new();
    for line in reader.lines().skip(1) {
        let line = line.expect("failed to read line");
        let msg = "quotas file is improperly formatted. must be csv.";
        quotas.push(
            line.split(',').map(|x| x.parse::<i32>().expect(msg)).collect()
        );
    }
    quotas
}

// write [[dtg, nq, rq, nqp, rqp], ...] to ./decks/quotas/deckname.csv
pub fn write_quotas_file(quotas: &Vec<Vec<i32>>, quotas_path: &PathBuf) {
    // compute (nq, rq) doubles for each day
    let buf = OpenOptions::new().write(true).open(quotas_path)
        .expect("failed to create quota file");
    let mut buf = BufWriter::new(buf);

    let header = "DaysToGo,NewQuota,ReviewQuota,NewPracticed,ReviewPracticed\n";
    buf.write_all(header.as_bytes()).expect("Unable to write data");

    for d in 0..quotas.len() {
        buf.write_fmt(format_args!(
            "{},{},{},{},{}\n",
             quotas[d][0], 
             quotas[d][1], 
             quotas[d][2], 
             quotas[d][3], 
             quotas[d][4]
            )
        ).expect("failed to write");
    }
}


// get days until specified date
pub fn days_until(datetime: DateTime<FixedOffset>) -> i64 {
    datetime.signed_duration_since(Local::now()).num_days()
}

pub fn path2string(path: &PathBuf) -> String {
    path.clone().into_os_string().into_string().unwrap()
}
