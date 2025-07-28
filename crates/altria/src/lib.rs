//! # Altria Library
//!
//! A comprehensive Rust library for web development with robust error handling,
//! utilities, and common patterns.

pub mod error;

// Re-export commonly used types
pub use error::{Error, ErrorKind, Result};
