use std::{
    fs::OpenOptions,
    io::{ BufReader, BufWriter },
    io::prelude::*,
};

use crate::mio0::{mio_deck::DeckEntry, utils::get_num_boxes};
use crate::mio0::utils::{ 
    calculate_hash, 
    compute_quotas, 
    read_quotas_file,
    write_quotas_file,
    path2string,
    redistribute_quotas
};


// appends cards in `new_path` to deck to name `deck_name`
pub fn update_deck(deck_name: String, new_path: String) {
    let deck = DeckEntry::new_from_name(&deck_name)
        .expect("failed to find deck");
    

    // open writer to deck file
    let deck_writer = OpenOptions::new()
        .append(true)
        .open(&path2string(&deck.deck_path_buf))
        .expect("failed to open deck file");
    let mut deck_writer = BufWriter::new(deck_writer);

    // open reader of file with cards
    let new_reader = OpenOptions::new()
        .read(true)
        .open(&new_path)
        .expect("failed to open deck file");
    let new_reader = BufReader::new(new_reader);
    let init_box_pos = 0;
    let mut num_cards = 0;

    // write header
    deck_writer.write_all(
        "CardId >> BoxPosition >> LastReviewTime >> Front >> Back\n".as_bytes()
    ).expect("failed to write header");

    for line in new_reader.lines() {
        let line = line.expect("failed to read line");
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
                    init_box_pos,
                    last_review,
                    front,
                    back
                )
            ).expect("failed to write");
        num_cards += 1;
    }


    let num_boxes = get_num_boxes(deck.days_to_go);
    update_quotas(deck, num_cards, num_boxes);
    
}

// computes quotas given `num_cards`, `num_boxes`, ans `deck.days_to_go`, and 
// adds them to the nq and rq files on the quotas file
fn update_quotas(deck: DeckEntry, num_cards: i32, num_boxes: i32) {
    // get quotas for the new cards being put into the deck (nq, rq)
    let mut new_quotas = compute_quotas(num_cards, deck.days_to_go, num_boxes);    
    // make index i correspond to days_to_go = i
    new_quotas.reverse();

    let quotas_path = deck.deck_path_buf.parent().unwrap()
        .join("quotas").join(deck.deck_name + "-quotas.csv");
    assert!(quotas_path.is_file(), 
        "quotas file does not exist. try `create` mode");
    // form: [[dtg, nq, rq, nqp, rqp], ...]
    let mut quotas = read_quotas_file(&quotas_path);

    assert!(quotas.len() >= new_quotas.len(), "days to go must decrease");

    for i in 0..new_quotas.len() {
        // update new quotas then review quotas
        new_quotas[i].0 += quotas[i][1];
        new_quotas[i].1 += quotas[i][2];
    }

    redistribute_quotas(&mut new_quotas);

    for i in 0..quotas.len() {
        // update new quotas then review quotas
        quotas[i][1] = new_quotas[i].0;
        quotas[i][2] = new_quotas[i].1;
    }
    
    write_quotas_file(&quotas, &quotas_path)


}