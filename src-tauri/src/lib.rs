// use log::info;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::Command;
use tauri_plugin_store::StoreExt;
use serde_json::Value;

fn parse_pactl_list() -> HashMap<String, HashMap<String, String>> {
    // Execute the `pactl list` command
    let output = Command::new("pactl")
        .arg("list")
        .output()
        .expect("Failed to execute command");

    // Convert the output to a string
    let stdout = String::from_utf8(output.stdout).expect("Failed to parse output as UTF-8");

    // Create a reader for the output
    let reader = BufReader::new(stdout.as_bytes());

    // Initialize the main HashMap
    let mut result = HashMap::new();
    let mut current_section = String::new();
    let mut current_map = HashMap::new();

    // Iterate over each line of the output
    for line in reader.lines() {
        let line = line.expect("Failed to read line");

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Check if the line is a section header (e.g., "Sink #0")
        if line.starts_with(char::is_alphabetic)
            && !line.starts_with('\t')
            && !line.starts_with(' ')
        {
            // Save the previous section if it exists
            if !current_section.is_empty() {
                result.insert(current_section.clone(), current_map);
                current_map = HashMap::new();
            }
            current_section = line.trim().to_string();
        } else {
            // Parse key-value pairs
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                current_map.insert(key, value);
            }
        }
    }

    // Insert the last section
    if !current_section.is_empty() {
        result.insert(current_section, current_map);
    }

    return result;
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn get_sound_server() -> String {
    let sound_server;
    if cfg!(target_os = "windows") {
        panic!("Not implemented for Windows!");
    } else {
        let output_pulseaudio = Command::new("pgrep")
            .arg("pulseaudio")
            .output()
            .expect("failed to execute process");
        let output_pipewire = Command::new("pgrep")
            .arg("pipewire")
            .output()
            .expect("failed to execute process");

        let pulseaudio_str = String::from_utf8_lossy(&output_pulseaudio.stdout);
        let pipewire_str = String::from_utf8_lossy(&output_pipewire.stdout);

        if pulseaudio_str != "" {
            sound_server = "pulseaudio".to_string()
        } else if pipewire_str != "" {
            sound_server = "pipewire".to_string()
        } else {
            panic!("Not implemented for other sound servers than pulseaudio or pipewire!");
        }
        // info!("sound_server: {}", sound_server);
    }
    return sound_server;
}

#[tauri::command]
fn get_audio_outputs(sound_server: String) -> (Vec<String>, Option<String>) {
    if sound_server == "pulseaudio" {
        return get_outputs_pulseaudio();
    } else if sound_server == "pipewire" {
        return get_outputs_pipewire();
    } else {
        panic!("Sound server not recognized!");
    }
    // return vec!["Hauts-parleurs".to_string(), "Razer".to_string(), "Vibe 100".to_string()];
}

fn get_outputs_pulseaudio() -> (Vec<String>, Option<String>) {
    return (Vec::<String>::new(), None);
}

fn get_outputs_pipewire() -> (Vec<String>, Option<String>) {
    let pactl_data = parse_pactl_list();
    // println!("{:#?}", pactl_data);

    let mut descriptions = Vec::new();
    let mut active_sink_desc = None;

    for (section, map) in pactl_data {
        if section.starts_with("Sink #") {
            if let Some(desc) = map.get("Description") {
                descriptions.push(desc.clone());
            }
            if let Some(state) = map.get("State") {
                if state == "RUNNING" {
                    if let Some(desc) = map.get("Description") {
                        active_sink_desc = Some(desc.clone());
                    }
                }
            }
        }
    }

    return (descriptions, active_sink_desc);
}

#[tauri::command]
async fn set_key_in_store(app: tauri::AppHandle, key: &str, value: Value) -> Result<(), String> {
    let store = app.store("store.json").map_err(|e| e.to_string())?;
    store.set(key, value);
    let _ = print_store_contents(&app);
    Ok(())
}

#[tauri::command]
async fn get_value_in_store(app: tauri::AppHandle, key: &str) -> Result<Value, String> {
    let store = app.store("store.json").map_err(|e| e.to_string())?;
    let value = store.get(key).ok_or_else(|| "Key not found in store".to_string())?;
    Ok(value)
}

fn print_store_contents(app: &tauri::AppHandle) -> Result<(), String> {
    let store = app.store("store.json").map_err(|e| e.to_string())?;

    // Get all keys in the store
    let keys = store.keys();

    // Iterate over keys and print their values
    for key in keys {
        let value = store.get(&key);
        println!("Key: {}, Value: {:?}", key, value);
    }

    Ok(())
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())        
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_sound_server,
            get_audio_outputs,
            set_key_in_store,
            get_value_in_store,
        ])
        .setup(|app| {
            let store = app.store("store.json")?;
            if let Some(value) = store.get("takenValues") {
                println!("{}", value);
            } else {
                println!("Key 'takenValues' not found in store.");
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
