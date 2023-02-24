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
  fs::{ create_dir, File },
  sync::{
    Mutex, 
    Arc
  },
  io::{
    BufWriter,
    Write
  }
};

mod edit;
use edit::{
  read_decks,
  write_decks,
  parse_textfield,
};

mod utils;
use utils::{
  AppDataDirState,
  calculate_hash,
  append_val_cfg,
  path2string
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
  get_deadline_progress,
  reset_deadline, FileSystemObject
};

/*
 * Run builder code
 */
fn main() {

  let review_session_state = ReviewSessionState {
    systems: Arc::new(Mutex::new(Vec::new())),
    quotas: Arc::new(Mutex::new(Vec::new())),
    deck_paths: Arc::new(Mutex::new(Vec::new())),
    dtg: Arc::new(Mutex::new(Vec::new()))
  };


  tauri::Builder::default()
    // define what variables will be in the state of the backend
    .setup(|app| {
      let handle = app.handle();
      let data_dir = handle.path_resolver().app_data_dir().unwrap();

      join_create_dir(&data_dir, "decks");
      join_create_dir(&data_dir, "folders");
      let folder_dir = &data_dir.join("folders");
      init_fs_json(&folder_dir);
      init_global_config(&folder_dir);
      join_create_dir(&folder_dir, "deadlines");

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
      reset_deadline,
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

/** 
 * Helper to initialize file system file at `folders/fs.json`
 */
fn init_fs_json(folder_dir: &PathBuf) {
  let fs_path = folder_dir.join("fs.json");
  if fs_path.is_file() {
    return;
  }

  let root_folder = FileSystemObject {
    entity_type: "folder".to_string(),
    name: "My Cardway".to_string(),
    files: Some(Vec::new()),
    expanded: Some(true),
    deadline_date: None,
    deadline_time: None
  };

  let mut fs = Vec::new();
  fs.push(root_folder);

  // initialize file
  let file = File::create(fs_path).expect("failed to open fs json");
  let mut writer = BufWriter::new(file);

  // Read the JSON contents of the file as an instance of `User`.
  let fs_string = serde_json::to_string_pretty(&fs[0])
      .expect("failed to read fs.json because it's not correctly formatted");

  writer.write_all(fs_string.as_bytes()).expect("failed to write json");
}


/** 
 * Helper to initialize global config file at `folders/global-config.json`
 */
fn init_global_config(folder_dir: &PathBuf) {
    let path = folder_dir.join("global-config.toml");
    if path.is_file() {
      return;
    }

    // init global config if hasn't been created yet
    File::create(&path).expect("failed to open fs json");
    let path_str = path2string(&path);
    append_val_cfg(&path_str, "is_dark_mode", "false");
    append_val_cfg(&path_str, "is_textfield", "false");

}