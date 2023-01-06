use std::{
    fs,
    fs::File,
    path::Path,
    io:: { BufReader, BufWriter },
    io::prelude::*
};


use crate::utils::{ 
    calculate_hash, 
    deadline_to_datetime, 
    compute_quotas, 
    append_val_cfg,
    // read_from_cfg,
    get_num_boxes,
    days_until,
    redistribute_quotas,
};

pub fn create_deck_from_string(name: String, deadline: String, text: String) -> Result<String, String>{
    // Check if deck already exists with this name, error if so
    // Construct path to write to
    // - get data dir
    // - append decks
    Ok("Deck Created".into())
    
}

pub fn create_deck(deadline: String, path: String) {


    // extract filename from file path
    let fname = Path::new(&path).file_stem()
        .expect("failed to unwrap filename").to_str().unwrap();

    // delete duplicate decks
    let num_deleted = delete_duplicate_decks(&fname);
    if num_deleted > 0 {
        eprintln!("Deleted {} old decks with duplicate name '{}'\n", 
            num_deleted, fname);
    }

    // create deck path, first parsing the deadline date
    let datetime = deadline_to_datetime(deadline);
    let timestamp_str = datetime.to_rfc3339();
    let deck_path = "./decks/".to_owned() + "" + fname + ".csv";

    // open file reader on file path
    let file = File::open(&path).expect("file not found {path}");
    let reader = BufReader::new(file);

    // initialize writer; writes to ./decks/{deckname}.txt
    let deck_writer = File::create(deck_path).unwrap();
    let mut deck_writer = BufWriter::new(deck_writer);

    let mut num_cards = 0;
    let start_lbox = 0;
    // read in each line into read_cards
    deck_writer.write_all(
        "CardId >> BoxPosition >> LastReviewTime >> Front >> Back\n".as_bytes()
    ).expect("failed to write header");

    for line in reader.lines() {
        let line = line.expect("unable to read line");
        // ensure this line contains a valid card
        if line.matches(">>").count() != 1 {
            continue;
        }
        let mut field_it = line.split(">>");
        let front = field_it.next().unwrap().trim();
        let back = field_it.next().unwrap().trim();
        let last_review = 0;

        // card id is the hash of the concatenated front and back of the card
        let cid = calculate_hash(&(front.to_string() + back));

        // write format into file
        deck_writer
            .write_fmt(
                format_args!(
                    "{} >> {} >> {} >> {} >> {}\n",
                    cid,
                    start_lbox,
                    last_review,
                    front,
                    back
                )
            ).expect("failed to write");

        num_cards += 1;
    }

    // write quota and config data associated with this deck 
    let days_to_go = days_until(datetime);
    let num_boxes = get_num_boxes(days_to_go);
    // write num_boxes data to `./decks/configs/deck_name-cfg.toml`
    append_val_cfg(fname, "num_boxes", num_boxes);
    append_val_cfg(fname, "deadline", timestamp_str);
    create_quotas_file(fname.to_string(), num_cards, days_to_go, num_boxes);
}

// compute quotas for user and write them into ./decks/quotas/deckname.csv
fn create_quotas_file(deck_name: String, num_cards: i32, days_to_go: i64,
    num_boxes: i32) {
    // compute (nq, rq) doubles for each day
    let mut quotas: Vec<(i32, i32)> = compute_quotas(num_cards, days_to_go, num_boxes);
    redistribute_quotas(&mut quotas);

    // hard coding quotas dir for now
    let quota_dir = Path::new("./decks/quotas");

    // create quota dir ./decks/quotas if does not exist
    fs::create_dir_all(&quota_dir).expect("failed to create/check quota dir");

    let quota_path = quota_dir.join(deck_name + "-quotas.csv");
    let buf = File::create(quota_path)
        .expect("failed to create quota file");
    let mut buf = BufWriter::new(buf);

    let header = "DaysToGo,NewQuota,ReviewQuota,NewPracticed,ReviewPracticed\n";
    buf.write_all(header.as_bytes()).expect("Unable to write data");

    for d in 0..days_to_go + 1 {
        let (nq, rq) = quotas.pop().expect("failed to retrieve quotas");
        buf.write_fmt(format_args!("{},{},{},0,0\n", d, nq, rq))
            .expect("failed to write");
    }
}



// delete files in `./decks` whose names are prefixed with `deck_name`,
// also deletes quota and config file associated with deck
fn delete_duplicate_decks(deck_name: &str) -> i32 {
    let mut num_deleted = 0;

    // Read in possible deck directory
    let decks_dir = "./decks";
    let dir = Path::new(decks_dir);
    if !dir.is_dir() {
        panic!("you must be in the main `mio0` directory to use the program");
    }

    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_file() {
            continue;
        }
        
        let fname = path.strip_prefix(decks_dir).unwrap()
            .to_str().unwrap();
        let read_deck_name = fname.split("--").next()
            .expect("invalid deckname");

        // if deck has a duplicate name
        if read_deck_name == deck_name {
            fs::remove_file(path).expect("failed to remove duplicate deck");
            num_deleted += 1;
        }
    }

    // delete quota file
    let quota_path =  dir.join("quotas")
        .join(deck_name.to_string() + "-quotas.csv");
    if quota_path.is_file() {
        fs::remove_file(quota_path).expect("failed to remove duplicate deck");
    }

    // delete config file


    let cfg_path = dir.join("configs")
        .join(deck_name.to_string() + "-cfg.toml");
    if cfg_path.is_file() {
        fs::remove_file(cfg_path).expect("failed to remove duplicate deck");
    }

    num_deleted
}
