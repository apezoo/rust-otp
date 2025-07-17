use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Represents a segment of a pad that has been used.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UsedSegment {
    pub start: usize,
    pub end: usize,
}

/// Represents the state of a single one-time pad file.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pad {
    pub id: String,
    pub file_name: String,
    pub size: usize,
    pub used_segments: Vec<UsedSegment>,
}

impl Pad {
    /// Finds the first available contiguous segment of a given length.
    pub fn find_available_segment(&self, length: usize) -> Option<usize> {
        // Sort segments by start byte to iterate through them in order.
        let mut sorted_segments = self.used_segments.clone();
        sorted_segments.sort_by_key(|s| s.start);

        // If there are no used segments, the whole pad is available.
        if sorted_segments.is_empty() {
            return if self.size >= length { Some(0) } else { None };
        }

        // Check for space before the first segment
        if sorted_segments[0].start >= length {
            return Some(0);
        }

        // Now, iterate through the gaps between segments.
        let mut last_end = sorted_segments[0].end;

        // Check for space between segments
        for segment in sorted_segments.iter().skip(1) {
            let gap = segment.start - last_end;
            if gap >= length {
                return Some(last_end);
            }
            last_end = segment.end;
        }

        // Check for space after the last segment
        if self.size - last_end >= length {
            return Some(last_end);
        }

        None
    }
}

/// Represents the state of an OTP Vault.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct VaultState {
    pub pads: HashMap<String, Pad>,
}

impl VaultState {
    /// Adds a new pad to the state.
    pub fn add_pad(&mut self, id: String, file_name: String, size: usize) {
        let pad = Pad {
            id: id.clone(),
            file_name,
            size,
            used_segments: vec![],
        };
        self.pads.insert(id, pad);
    }
}

/// Loads the state from a specific vault path.
pub fn load_state(vault_path: &Path) -> VaultState {
    let state_file_path = vault_path.join("vault_state.json");
    if state_file_path.exists() {
        let state_str = fs::read_to_string(state_file_path).expect("Failed to read state file");
        serde_json::from_str(&state_str).expect("Failed to parse state file")
    } else {
        VaultState::default()
    }
}

/// Saves the state to a specific vault path.
pub fn save_state(vault_path: &Path, state: &VaultState) {
    let state_file_path = vault_path.join("vault_state.json");
    let state_str = serde_json::to_string_pretty(state).expect("Failed to serialize state");
    fs::write(state_file_path, state_str).expect("Failed to write state file");
}