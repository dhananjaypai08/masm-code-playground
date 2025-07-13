pub mod client;

#[cfg(feature = "web_server")]
pub mod web_server;

use client::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            instantiate, 
            exec_program, 
            exec_program_with_inputs, 
            generate_proof_with_inputs, 
            get_example_programs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}