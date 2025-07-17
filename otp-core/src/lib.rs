// File:    lib.rs
// Author:  apezoo
// Date:    2025-07-17
//
// Description: The main library crate for otp-core, orchestrating encryption, decryption, and pad management.
//
// License:
// This project is licensed under the terms of the GNU AGPLv3 license.
// See the LICENSE.md file in the project root for full license information.

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