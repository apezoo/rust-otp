//! # OTP Core Library
//!
//! This library provides the core functionality for one-time pad (OTP) encryption,
//! including pad generation, state management, and the cryptographic operations.

/// Cryptographic operations for encryption and decryption.
pub mod crypto;
/// Utilities for generating new one-time pads.
pub mod pad_generator;
/// Manages the state of the OTP vault, including pad usage.
pub mod state_manager;