use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the state of a single one-time pad file.
#[derive(Serialize, Deserialize, Debug)]
pub struct PadState {
    pub path: String,
    pub size: usize,
    pub used_bytes: usize,
}

/// Represents the overall state of all managed pads.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AppState {
    pub pads: HashMap<String, PadState>,
}

impl AppState {
    /// Adds a new pad to the state.
    pub fn add_pad(&mut self, path: String, size: usize) {
        let pad_state = PadState {
            path: path.clone(),
            size,
            used_bytes: 0,
        };
        self.pads.insert(path, pad_state);
    }
}