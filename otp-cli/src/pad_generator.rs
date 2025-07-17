use rand::{rngs::OsRng, TryRngCore};
use std::fs::File;
use std::io::{Write, Error, ErrorKind};

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
pub fn generate_pad(path: &str, size: usize) -> std::io::Result<()> {
    let mut rng = OsRng;
    let mut buffer = vec![0u8; size];
    // Use the failable `try_fill_bytes` and map the error to an `io::Error`.
    rng.try_fill_bytes(&mut buffer)
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    let mut file = File::create(path)?;
    file.write_all(&buffer)?;

    Ok(())
}