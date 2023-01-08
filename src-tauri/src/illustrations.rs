

// #[tauri::command]
// fn get_decks() -> Vec<DeckEntry> {

//   let deadline = NaiveDate::from_ymd_opt(2014, 11, 28).unwrap().and_hms_opt(12, 0, 0).unwrap();

//   let mut entries: Vec<DeckEntry> = Vec::new();

//   entries.push(DeckEntry {
//     id: 1,
//     name: "German".to_string(),
//     deadline_string: deadline.to_string(),
//   });

//   entries
// }

// #[derive(Serialize, Deserialize)]
// struct ExperimentalDeckEntry {
//   name: String,
//   ndt: NaiveDateTime,
// }

// #[tauri::command]
// async fn create_card_from_csv(card_path: &str) -> Result<String, String> {
//   println!("create_card_from_csv received: {}", card_path);

//   // Try to create the card from the CSV.
//   // Return any meaningful errors to the frontend if it fails 
//   // Otherwise let the user know it was a success

//   // Ok("Card Created".to_string());
//   Err("Not actually an error, create_card_from_csv not implemented".to_string())
// }


// #[tauri::command]
// async fn pde(deck_entry: ExperimentalDeckEntry) -> String {
//   println!("Hello! From Rust {}, {}", deck_entry.name, deck_entry.ndt);
//   "TEST".to_string()
// }

// #[tauri::command]
// fn create_deck(data_dir: tauri::State<AppDataDirState>, deck_info: NewDeckInfo) {
//   let disp = data_dir.path.as_ref().unwrap();
//   println!("DD: {}", disp.display());
//   println!("{}", deck_info.name); 
//   println!("{}", deck_info.deadline_string); // RFC3339
//   println!("{}", deck_info.text);
 
// }