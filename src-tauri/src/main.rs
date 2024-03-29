// boilerplate allowing tauri to work on windows
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
mod home_db;
use crate::home_db::{ 
  init_root_folder,
  init_getting_started,
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
  toggle_is_expanded
};

mod edit_db;
use edit_db::{
  read_deadline_contents,
  write_text_field,
  create_cards,
  update_card,
  delete_card,
};

mod utils_db;
use utils_db::get_is_anki_frontend;

mod review_db;
use review_db::{
  ReviewSessionState,
  init_review_session,
  get_next_card,
  record_response,
  get_last_card,
  print_cards
};


// use diesel::pg::SqliteConnection;
use diesel::sqlite::SqliteConnection;
use diesel::Connection;

// use dotenvy::dotenv;
use std::env;
use dotenvy::dotenv;

// ../migrations

// pub fn establish_connection() -> SqliteConnection {
//   let database_url = "./trunk.db";
 
//   SqliteConnection::establish(&database_url)
//     .expect(&format!("Error connecting to {}", database_url))
// }

// #[macro_use]
// extern crate diesel_migrations;

// use diesel_migrations::{embed_migrations};
// embed_migrations!("../migrations");


pub fn establish_connection() -> SqliteConnection {
  dotenv().ok();

  // let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  // let database_url = "./trunk.db";
  let database_url = "adam.db";
  SqliteConnection::establish(&database_url)
      .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}


// Postgres is not working for me. What do I need to do to use SqLite now?
/*
 * Run builder code
 */
// #[macro_use]
// extern crate diesel_migrations;


// embed_migrations!("migrations");

fn main() {
  let mut conn = establish_connection();
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


  tauri::Builder::default()
    // define what variables will be in the state of the backend
    .setup(|app| {
      // let handle = app.handle();
      // let data_dir = handle.path_resolver().app_data_dir().unwrap();

      app.manage(database_state);
      app.manage(review_session_state);


      Ok(())
    })
    // define what backend functions are callable from the frontend
    .invoke_handler(tauri::generate_handler![
      // home_db
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

      print_cards,

      // edit_db
      read_deadline_contents,
      write_text_field,
      create_cards,
      update_card,
      delete_card,

      // review_db
      init_review_session,
      get_next_card,
      record_response,
      get_last_card,

      // utils_db
      get_is_anki_frontend

      ])
    // run application (boilerplate)
    .run(tauri::generate_context!())
    .expect("error while running tauri application");



}


use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
use std::error::Error;


fn run_migrations(connection: &mut impl MigrationHarness<diesel::sqlite::Sqlite>) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {

    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.
    connection.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}