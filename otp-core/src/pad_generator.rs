// File:    pad_generator.rs
// Author:  apezoo
// Date:    2025-07-17
//
// Description: Provides functionality for generating high-quality one-time pads for cryptographic use.
//
// License:
// This project is licensed under the terms of the GNU AGPLv3 license.
// See the LICENSE.md file in the project root for full license information.

use rand::{rngs::OsRng, TryRngCore};
use std::fs::File;
use std::io::Write;

/// Generates a new one-time pad file with the specified size in bytes.
///
/// # Arguments
///
/// * `path` - The path where the pad file will be created.
/// * `size` - The size of the pad in bytes.
///
/// # Returns
///
/// A `std::io::Result<()>` which is `Ok(())` on success and `Err` on failure.
///
/// # Errors
///
/// This function will return an error if the pad file cannot be created or written to.
pub fn generate_pad(path: &str, size: usize) -> std::io::Result<()> {
    let mut rng = OsRng;
    let mut buffer = vec![0u8; size];
    // Use the failable `try_fill_bytes` and map the error to an `io::Error`.
    rng.try_fill_bytes(&mut buffer)
        .map_err(std::io::Error::other)?;

    let mut file = File::create(path)?;
    file.write_all(&buffer)?;

    Ok(())
}