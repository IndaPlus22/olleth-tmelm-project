// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Write;

mod backend {
    pub mod encoder;
    pub mod file;
    pub mod youtubeapi;
    pub mod csvreader;
}


// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn create_user(path: &str) -> backend::youtubeapi::User {
    let clients = backend::csvreader::create_client_secret_csv(path).unwrap();
    let user = tauri::async_runtime::block_on(backend::youtubeapi::User::new(clients.to_str().unwrap())).expect("error");
    user
}

#[tauri::command]
fn display_directory(user: backend::youtubeapi::User) -> Vec<String> {
    let keys: Vec<String> = user.directory.keys().cloned().collect();
    keys
}

#[tauri::command]
fn upload_video(mut user: backend::youtubeapi::User, filename: String, bytes: String) -> bool {
    let bytes_vec: Vec<u8> = bytes.split(',')
    .map(|s| s.parse().unwrap())
    .collect();

    let path = format!("../input/{}", filename);

    // create a new file and a buffered writer
    let file = std::fs::File::create(&path).unwrap();
    let mut writer = std::io::BufWriter::new(file);

    // write the data in chunks
    for chunk in bytes_vec.chunks(1024 * 1024) {
        writer.write_all(chunk).unwrap();
    }

    let result = tauri::async_runtime::block_on(user.upload(&path));
    match result {
        Ok(_res) => { 
            match std::fs::remove_file(format!("../videos/{}.mp4", filename)) {
                Ok(()) => println!("File removed successfully!"),
                Err(e) => println!("Error removing file: {}", e),
            }
            return true 
        },
        Err(_e) => return false,
    };

}

#[tauri::command]
fn username(user: backend::youtubeapi::User) -> String {
    println!("username: {}\nid: {}", user.name, user.id);
    //format!("username: {}\nid: {}", user.name, user.id)
    format!("?")
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, create_user, display_directory, username, upload_video])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
