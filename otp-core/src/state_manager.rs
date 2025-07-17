use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Represents a segment of a pad that has been used.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UsedSegment {
    /// The starting byte (inclusive) of the used segment.
    pub start: usize,
    /// The ending byte (exclusive) of the used segment.
    pub end: usize,
}

/// Represents the state of a single one-time pad file.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pad {
    /// A unique identifier for the pad.
    pub id: String,
    /// The name of the file containing the pad data.
    pub file_name: String,
    /// The total size of the pad in bytes.
    pub size: usize,
    /// A list of segments that have been used.
    pub used_segments: Vec<UsedSegment>,
    /// Whether the pad has been fully consumed.
    pub is_fully_used: bool,
}

impl Pad {
    /// Calculates the total number of bytes used in the pad.
    pub fn total_used_bytes(&self) -> usize {
        self.used_segments.iter().map(|s| s.end - s.start).sum()
    }

    /// Checks if the pad is fully consumed.
    pub fn is_fully_used(&self) -> bool {
        self.total_used_bytes() >= self.size
    }

    /// Checks if the pad was fully used *before* a new segment of a given length was notionally added.
    /// This is important for finding the correct pad file directory during decryption.
    pub fn is_fully_used_before(&self, new_segment_length: usize) -> bool {
        let current_usage = self.total_used_bytes();
        // If the current usage is already conclusive, no need to subtract.
        if current_usage >= self.size {
            return true;
        }
        // Check if the state *before* this decryption would have been 'not full'.
        if current_usage < new_segment_length {
            // This case shouldn't logically happen if state is consistent, but as a safeguard:
            return false;
        }
        // This is the core logic: determine if the pad *was* available before this segment was used.
        (current_usage - new_segment_length) < self.size
    }


    /// Finds the first available contiguous segment of a given length.
    pub fn find_available_segment(&self, length: usize) -> Option<usize> {
        if self.is_fully_used() {
            return None;
        }

        // Sort segments by start byte to iterate through them in order.
        let mut sorted_segments = self.used_segments.clone();
        sorted_segments.sort_by_key(|s| s.start);

        // Handle case for an empty or completely available pad
        if sorted_segments.is_empty() {
            return if self.size >= length { Some(0) } else { None };
        }

        // Check for space before the first used segment
        if sorted_segments[0].start >= length {
            return Some(0);
        }
        
        // Now, iterate through the gaps between used segments.
        let mut last_end = sorted_segments[0].end;
        for segment in sorted_segments.iter().skip(1) {
            let gap = segment.start.saturating_sub(last_end);
            if gap >= length {
                return Some(last_end); // Found a suitable gap
            }
            last_end = segment.end;
        }

        // Finally, check for space after the very last segment
        if self.size.saturating_sub(last_end) >= length {
            return Some(last_end);
        }

        None
    }
}

/// Represents the state of an OTP Vault.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct VaultState {
    /// A map of pad IDs to their corresponding `Pad` state.
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
            is_fully_used: false,
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