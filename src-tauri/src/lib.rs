
use std::io;
mod review_session;
mod create_deck;
mod update_deck;
mod mio_deck;
mod utils;

use create_deck::create_deck;
use update_deck::update_deck;
use review_session::run_session;

// nets called by Main
pub fn run(config: Config) -> Result<(), io::Error> {
    let mode = config.mode;
    match mode {
        Modes::Help => print_help_dialogue(),
        Modes::CreateDeck(dl, path) => create_deck(dl, path),
        Modes::UpdateDeck(deck_name, path) => update_deck(deck_name, path),
        Modes::ListDecks => mio_deck::print_decks(),
        Modes::ReviewDeck(deck_name) => run_session(deck_name),
    }
    Ok(())
}

// The Possible Modes For the Program
pub enum Modes {
    Help,
    CreateDeck(String, String),
    UpdateDeck(String, String),
    ListDecks,
    ReviewDeck(String),   
}

pub struct Config {
    pub mode: Modes,
}

impl Config {
    // This handles configuring
    pub fn new(args: &[String]) -> Result<Config, &str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        }

        let option_string = args[1].clone();

        let chosen_mode = match option_string.as_str() {
            "help" => { Modes::Help }
            "ls" => { Modes::ListDecks }
            "create" => {
                if args.len() != 4 {
                    return Err("wrong number of arguments");
                }

                Modes::CreateDeck(args[2].clone(), args[3].clone())
            }
            "update" => {
                // ./mio0 update <deck_name> <new_path>
                if args.len() != 4 {
                    return Err("wrong number of arguments");
                }
                Modes::UpdateDeck(args[2].clone(), args[3].clone())
            }
            "review" => {
                if args.len() != 3 {
                    return Err("wrong number of arguments");
                }
                Modes::ReviewDeck(args[2].clone())
            }
            _ => {
                return Err("invalid first argument, run help to see options");
            }
        };
        Ok(Config { mode: chosen_mode })
    }
}


fn print_help_dialogue() {
    let dialog =
        "\
USAGE
    Generally:
		./mio0 <mode> [ARGS] 

	By mode: 
		./mio0 help
		./mio0 ls
		./mio0 create <deadline> <path_to_csv>
            <deadline> is a date and time YYYY-MM-DD-HH:MM format
            <path_to_csv> specifies the path to the file with your notes
        ./mio0 update <deck_name> <new_path>
            <deck_name> name of deck to review (name of file used to create it)
            <new_path> path to file with cards to update deck with
		./mio0 review <deck_name>
            <deck_name> name of deck to review (name of file used to create it)

MODES
	help      show this help menu
	ls        list decks and their deadlines
	create    create a deck from file
    update    append cards in specifed file to a deck
	review	  begins a review session

    ";
    println!("{dialog}");
}
