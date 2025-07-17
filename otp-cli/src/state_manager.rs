use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the state of a single one-time pad file.
use std::fs;
use std::path::Path;

const STATE_FILE: &str = ".otp_state.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pad {
    pub id: String,
    pub path: String,
    pub size: usize,
    pub used_bytes: usize,
}

/// Represents the overall state of all managed pads.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AppState {
    pub pads: HashMap<String, Pad>,
}

impl AppState {
    /// Adds a new pad to the state.
    pub fn add_pad(&mut self, id: String, path: String, size: usize) {
        let pad = Pad {
            id: id.clone(),
            path,
            size,
            used_bytes: 0,
        };
        self.pads.insert(id, pad);
    }
}

pub fn load_state() -> AppState {
    if Path::new(STATE_FILE).exists() {
        let state_str = fs::read_to_string(STATE_FILE).expect("Failed to read state file");
        serde_json::from_str(&state_str).expect("Failed to parse state file")
    } else {
        AppState::default()
    }
}

pub fn save_state(state: &AppState) {
    let state_str = serde_json::to_string_pretty(state).expect("Failed to serialize state");
    fs::write(STATE_FILE, state_str).expect("Failed to write state file");
}