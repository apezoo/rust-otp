//! This module contains the core cryptographic operations.

/// Performs a simple XOR operation between two byte slices.
///
/// Panics if the slices are not of equal length.
pub fn xor(a: &[u8], b: &[u8]) -> Vec<u8> {
    if a.len() != b.len() {
        panic!("Input slices must have the same length for XOR operation.");
    }
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}