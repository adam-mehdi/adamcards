#![allow(unused_imports)]
#![allow(dead_code)]

// boilerplate allowing tauri to work on windows
#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::{
  Manager,
  State
};
use std::{
  path::PathBuf,
  fs::create_dir,
  sync::{
    Mutex, 
    Arc
  }
};

mod update;
use update::{
  read_decks,
  write_decks,
  parse_textfield,
};

mod utils;
use utils::{
  AppDataDirState,
  calculate_hash
  // DeckEntry,
};

mod review;
use review::{
  ReviewSessionState,
  init_review_session,
  draw_cards,
  cleanup,
  save_card_buffer
};

mod home;
use home::{
  read_global_config,
  write_global_config,
  read_fs_json,
  write_fs_json,
  create_deadline,
  create_deck,
  rename_entry,
  delete_entry,
  get_deck_quotas,
  get_deadline_progress
};

/*
 * Run builder code
 */
fn main() {

  let review_session_state = ReviewSessionState {
    systems: Arc::new(Mutex::new(Vec::new())),
    quotas: Arc::new(Mutex::new(Vec::new())),
    deck_paths: Arc::new(Mutex::new(Vec::new())),
  };


  tauri::Builder::default()
    // define what variables will be in the state of the backend
    .setup(|app| {
      let handle = app.handle();
      let data_dir = handle.path_resolver().app_data_dir().unwrap();

      join_create_dir(&data_dir, "decks");
      join_create_dir(&data_dir, "deleted");
      join_create_dir(&data_dir, "folders");

      let data_dir_state = AppDataDirState {
        path: Some(data_dir),
      };


      app.manage(data_dir_state);
      app.manage(review_session_state);

      Ok(())
    })
    // define what backend functions are callable from the frontend
    .invoke_handler(tauri::generate_handler![
      read_global_config,
      write_global_config,
      get_deck_quotas,
      read_decks,
      get_deadline_progress,
      calculate_hash,
      write_decks,
      parse_textfield,
      init_review_session,
      draw_cards,
      save_card_buffer,
      cleanup,
      read_fs_json,
      write_fs_json,
      create_deadline,
      create_deck,
      rename_entry,
      delete_entry
      ])
    // run application (boilerplate)
    .run(tauri::generate_context!())
    .expect("error while running tauri application");



}

/**
 * Helper to setup base directories at root of app data
 */
fn join_create_dir(root: &PathBuf, dir: &str) {
  let dir = &root.join(dir);
  if !dir.is_dir() {
    create_dir(dir)
      .expect("failed to create data");
  }
}


