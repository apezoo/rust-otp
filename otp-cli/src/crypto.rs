use std::io::{self, Read, Write};

/// Encrypts or decrypts data from a reader using a pad and writes to a writer.
///
/// The OTP operation is symmetric; applying it once encrypts, and applying it
/// again to the ciphertext with the same pad decrypts.
///
/// # Arguments
///
/// * `reader` - The source of the plaintext or ciphertext.
/// * `writer` - The destination for the resulting ciphertext or plaintext.
/// * `pad_segment` - The segment of the one-time pad to use for the operation.
///
/// # Returns
///
/// An `io::Result<()>` indicating success or failure.
pub fn process_stream<R: Read, W: Write>(
    mut reader: R,
    mut writer: W,
    pad_segment: &[u8],
) -> io::Result<()> {
    let mut buffer = [0; 4096]; // Process in 4KB chunks
    let mut total_bytes_processed = 0;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let input_chunk = &buffer[..bytes_read];

        let pad_start = total_bytes_processed;
        let pad_end = total_bytes_processed + bytes_read;

        if pad_end > pad_segment.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Input is larger than the provided pad segment.",
            ));
        }

        let pad_chunk = &pad_segment[pad_start..pad_end];

        let mut processed_chunk = Vec::with_capacity(bytes_read);
        for (i, &byte) in input_chunk.iter().enumerate() {
            processed_chunk.push(byte ^ pad_chunk[i]);
        }

        writer.write_all(&processed_chunk)?;
        total_bytes_processed += bytes_read;
    }

    Ok(())
}