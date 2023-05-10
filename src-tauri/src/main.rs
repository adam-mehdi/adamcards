#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::Manager;
use std::sync::{
    Mutex, 
    Arc
};

mod models;
mod anki;
mod schema;
mod home;
use crate::home::{ 
  delete_entry,
  create_entry, 
  rename_entry,
  move_entry,
  write_dark_mode,
  read_folder_system, 
  DatabaseState, 
  is_duplicate_name,
  read_user_config,
  get_deadline_date,
  entered_past_deadline,
  reset_deadline,
  toggle_is_expanded,
  write_api_key,
  get_api_key
};

mod startup;
use crate::startup::{
  init_root_folder,
  init_getting_started
};

mod edit;
use edit::{
  read_deadline_contents,
  write_text_field,
  create_cards,
  update_card,
  delete_card,
};

// mod chat;
// use chat::get_explanation;

mod utils;
use utils::get_is_anki_frontend;

mod review;
use review::{
  ReviewSessionState,
  init_review_session,
  get_next_card,
  record_response,
  get_last_card,
  print_cards,
  get_next_intervals
};


mod db;
use db::{establish_connection, run_migrations};

mod chat;
use chat::send_gpt_request;

fn main() {


  tauri::Builder::default()
    // define what variables will be in the state of the backend
    .setup(|app| {
      
      let mut conn = establish_connection(app);
      run_migrations(&mut conn).expect("Error embedding migrations");
    
      
      if init_root_folder(&mut conn) {
        init_getting_started(&mut conn);
      }
    
      let database_state = DatabaseState {
        conn: Arc::new(Mutex::new(conn))
      };
    
      let review_session_state = ReviewSessionState {
        response_stack: Arc::new(Mutex::new(Vec::new())),
        undo_response_stack: Arc::new(Mutex::new(Vec::new())),
        curr_card: Arc::new(Mutex::new(None)),
        new_ids: Arc::new(Mutex::new(Vec::new())),
        days_to_go: Arc::new(Mutex::new(None)),
        deadline_id: Arc::new(Mutex::new(None))
      };
    

      app.manage(database_state);
      app.manage(review_session_state);


      Ok(())
    })
    // define what backend functions are callable from the frontend
    .invoke_handler(tauri::generate_handler![
      // home
      read_folder_system,
      create_entry,
      delete_entry,
      rename_entry,
      move_entry,
      is_duplicate_name,
      write_dark_mode,
      read_user_config,
      get_deadline_date,
      entered_past_deadline,
      reset_deadline,
      toggle_is_expanded,
      write_api_key,
      get_api_key,

      print_cards,

      // edit
      read_deadline_contents,
      write_text_field,
      create_cards,
      update_card,
      delete_card,

      // review
      init_review_session,
      get_next_card,
      record_response,
      get_last_card,

      // utils
      get_is_anki_frontend,

      //anki
      get_next_intervals,

      // chat
      send_gpt_request,

      ])
    // run application (boilerplate)
    .run(tauri::generate_context!())
    .expect("error while running tauri application");



}
