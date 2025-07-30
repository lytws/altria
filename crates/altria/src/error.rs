//! Error handling for the Altria library.
//!
//! This module provides a comprehensive error handling system built around the [`Error`] struct,
//! which implements Rust's standard error handling patterns while providing additional features
//! like metadata, error codes, backtraces, and error chaining.
//!
//! # Features
//!
//! - **Structured Error Information**: Store error messages, codes, and custom metadata
//! - **Error Chaining**: Chain errors together using Rust's standard `std::error::Error` trait
//! - **Backtraces**: Optional capture of call stack information for debugging
//! - **Builder Pattern**: Fluent API for constructing complex errors
//! - **Iterator Support**: Traverse error chains using standard Rust iterators
//!
//! # Examples
//!
//! ## Basic Error Creation
//!
//! ```rust
//! use altria::error::Error;
//!
//! let error = Error::new("Something went wrong");
//! println!("{}", error);
//! ```
//!
//! ## Error with Metadata and Code
//!
//! ```rust
//! use altria::error::Error;
//!
//! let error = Error::new("Validation failed")
//!     .with_code(400)
//!     .with_metadata("field", "email")
//!     .with_metadata("value", "invalid@email");
//!
//! println!("Error code: {:?}", error.code());
//! println!("Field: {:?}", error.metadata().get("field"));
//! ```
//!
//! ## Error Chaining
//!
//! ```rust
//! use altria::error::Error;
//! use std::io;
//!
//! let io_error = io::Error::new(io::ErrorKind::NotFound, "config.toml not found");
//! let app_error = Error::new("Failed to load configuration")
//!     .with_source(io_error);
//!
//! // Traverse the error chain
//! for error in app_error.iter_error_chain() {
//!     println!("Error: {}", error);
//! }
//! ```
//!
//! ## With Backtrace (for debugging)
//!
//! ```rust
//! use altria::error::Error;
//!
//! let error = Error::new("Critical system failure")
//!     .with_backtrace();
//!
//! println!("{}", error); // Will include backtrace in output
//! ```
//!
//! # Error Chain Iteration
//!
//! The [`Error::iter_error_chain`] method returns an iterator that traverses the entire
//! error chain, starting from the current error and following the source chain:
//!
//! ```rust
//! use altria::error::Error;
//!
//! let root_cause = Error::new("Network timeout");
//! let intermediate = Error::new("Service unavailable").with_source(root_cause);
//! let top_level = Error::new("Request failed").with_source(intermediate);
//!
//! let errors: Vec<_> = top_level.iter_error_chain().collect();
//! assert_eq!(errors.len(), 3);
//! ```
//!
//! # Integration with Standard Library
//!
//! The [`Error`] type implements [`std::error::Error`], making it compatible with the
//! standard Rust error handling ecosystem, including libraries like `anyhow` and `thiserror`.

use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

/// A comprehensive error type with support for error chaining, metadata, backtraces,
/// and structured error information.
///
/// This type is designed to be the primary error type for applications, providing
/// rich error context while maintaining compatibility with Rust's standard error
/// handling patterns.
#[derive(Debug)]
pub struct Error {
    /// Primary error message
    message: String,
    /// Error code (optional)
    code: Option<i32>,
    /// Stack trace captured at error creation (optional for performance)
    backtrace: Option<Backtrace>,
    /// Additional metadata
    metadata: HashMap<String, String>,
    /// Source error chain (using `std::error::Error` trait)
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl Error {
    /// Create a new error with a message
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
            backtrace: None,
            metadata: HashMap::new(),
            source: None,
        }
    }

    /// Get the error message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the error code
    pub const fn code(&self) -> Option<i32> {
        self.code
    }

    /// Get the metadata
    pub const fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Set custom error code
    #[must_use]
    pub const fn with_code(mut self, code: i32) -> Self {
        self.code = Some(code);
        self
    }

    /// Capture and attach a backtrace to the error
    ///
    /// This method captures the current call stack and attaches it to the error.
    /// This can be useful for debugging but may have performance implications.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use altria::error::Error;
    /// let error = Error::new("Something went wrong")
    ///     .with_backtrace();
    /// ```
    #[must_use]
    pub fn with_backtrace(mut self) -> Self {
        self.backtrace = Some(Backtrace::capture());
        self
    }

    /// Add metadata to the error
    ///
    /// Metadata provides additional context about the error that can be useful
    /// for debugging, logging, or error handling. This method can be chained
    /// to add multiple metadata entries.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use altria::error::Error;
    /// let error = Error::new("Invalid input")
    ///     .with_metadata("field", "email")
    ///     .with_metadata("value", "invalid@")
    ///     .with_metadata("rule", "email_format");
    ///
    /// assert_eq!(error.metadata().get("field"), Some(&"email".to_string()));
    /// ```
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add multiple metadata entries
    #[must_use]
    pub fn with_metadata_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    /// Chain with source error
    ///
    /// This method allows you to create an error chain by setting a source error.
    /// The source error must implement `std::error::Error + Send + Sync + 'static`.
    /// This integrates with Rust's standard error handling mechanisms.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use altria::error::Error;
    /// use std::io;
    /// use std::error::Error as StdError; // Import trait for source() method
    ///
    /// let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    /// let app_error = Error::new("Failed to load config")
    ///     .with_source(io_error);
    ///
    /// // You can traverse the error chain using std::error::Error::source()
    /// assert!(app_error.source().is_some());
    /// ```
    #[must_use]
    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    // Error kind checking methods generated by macro above

    /// Get error chain as iterator using `std::error::Error`'s source mechanism
    ///
    /// This method returns an iterator over all errors in the chain, starting with
    /// the current error and following the source chain. This is the standard
    /// way to traverse error chains in Rust, providing lazy evaluation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use altria::error::Error;
    /// let source = Error::new("File not found");
    /// let main_error = Error::new("Config load failed")
    ///     .with_source(source);
    ///
    /// let chain: Vec<_> = main_error.iter_error_chain().collect();
    /// assert_eq!(chain.len(), 2);
    /// // chain[0] is the main_error
    /// // chain[1] is the source error
    /// ```
    pub fn iter_error_chain(&self) -> ErrorChainIter<'_> {
        ErrorChainIter {
            current: Some(self),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 1. Error code (if present)
        if let Some(code) = self.code {
            writeln!(f, "Code: {code}")?;
        }

        // 2. Error message
        writeln!(f, "Error: {}", self.message)?;

        // 3. Metadata (if present)
        if !self.metadata.is_empty() {
            writeln!(f, "Metadata:")?;
            for (key, value) in &self.metadata {
                writeln!(f, "  {key}: {value}")?;
            }
        }

        // 4. Stack trace (if captured)
        if let Some(backtrace) = &self.backtrace {
            writeln!(f, "Backtrace:")?;
            writeln!(f, "{backtrace}")?;
        }

        // 5. Source error chain
        if let Some(source) = &self.source {
            writeln!(f, "Caused by:")?;
            let source_str = format!("{source}");
            for line in source_str.lines() {
                writeln!(f, "  {line}")?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|e| e as &(dyn std::error::Error + 'static))
    }
}

/// An iterator over an error and its sources.
///
/// This iterator is created by the [`Error::iter_error_chain`] method and provides
/// lazy traversal over the entire error chain, starting from the current error
/// and following the source chain through the [`std::error::Error::source`] method.
///
/// # Examples
///
/// ```rust
/// # use altria::error::Error;
/// let root = Error::new("Root cause");
/// let top = Error::new("Top level").with_source(root);
///
/// for (i, error) in top.iter_error_chain().enumerate() {
///     println!("Level {}: {}", i, error);
/// }
/// ```
pub struct ErrorChainIter<'a> {
    current: Option<&'a (dyn std::error::Error + 'static)>,
}

impl<'a> Iterator for ErrorChainIter<'a> {
    type Item = &'a (dyn std::error::Error + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        let current_error = self.current;
        if let Some(error) = current_error {
            self.current = error.source();
        }
        current_error
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;
    use std::io; // Import the trait

    #[test]
    fn test_error_creation() {
        let error = Error::new("Test error");
        assert_eq!(error.message(), "Test error");
        assert_eq!(error.code(), None);
        assert!(error.metadata().is_empty());
    }

    #[test]
    fn test_error_with_code() {
        let error = Error::new("Test error").with_code(404);
        assert_eq!(error.message(), "Test error");
        assert_eq!(error.code(), Some(404));
    }

    #[test]
    fn test_error_with_metadata() {
        let error = Error::new("Test error")
            .with_metadata("key1", "value1")
            .with_metadata("key2", "value2");

        assert_eq!(error.metadata().get("key1"), Some(&"value1".to_string()));
        assert_eq!(error.metadata().get("key2"), Some(&"value2".to_string()));
        assert_eq!(error.metadata().len(), 2);
    }

    #[test]
    fn test_error_with_metadata_map() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());
        metadata.insert("key2".to_string(), "value2".to_string());

        let error = Error::new("Test error").with_metadata_map(metadata);

        assert_eq!(error.metadata().get("key1"), Some(&"value1".to_string()));
        assert_eq!(error.metadata().get("key2"), Some(&"value2".to_string()));
        assert_eq!(error.metadata().len(), 2);
    }

    #[test]
    fn test_error_with_source() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = Error::new("Failed to read file").with_source(io_error);

        assert!(error.source().is_some());
        assert_eq!(error.source().unwrap().to_string(), "File not found");
    }

    #[test]
    fn test_error_chaining() {
        let root_error = Error::new("Root cause");
        let middle_error = Error::new("Middle error").with_source(root_error);
        let top_error = Error::new("Top level error").with_source(middle_error);

        let chain: Vec<_> = top_error.iter_error_chain().collect();
        assert_eq!(chain.len(), 3);

        // Test the chain contents by checking error messages
        assert!(chain[0].to_string().contains("Top level error"));
        assert!(chain[1].to_string().contains("Middle error"));
        assert!(chain[2].to_string().contains("Root cause"));
    }

    #[test]
    fn test_error_chain_with_std_error() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let app_error = Error::new("Cannot read config file").with_source(io_error);

        let chain: Vec<_> = app_error.iter_error_chain().collect();
        assert_eq!(chain.len(), 2);

        assert!(chain[0].to_string().contains("Cannot read config file"));
        assert_eq!(chain[1].to_string(), "Access denied");
    }

    #[test]
    fn test_error_chain_iterator_empty() {
        let error = Error::new("Single error");
        let chain: Vec<_> = error.iter_error_chain().collect();
        assert_eq!(chain.len(), 1);
        assert!(chain[0].to_string().contains("Single error"));
    }

    #[test]
    fn test_error_display_basic() {
        let error = Error::new("Test error");
        let display = format!("{}", error);
        assert!(display.contains("Error: Test error"));
    }

    #[test]
    fn test_error_display_with_code() {
        let error = Error::new("Test error").with_code(500);
        let display = format!("{}", error);
        assert!(display.contains("Code: 500"));
        assert!(display.contains("Error: Test error"));
    }

    #[test]
    fn test_error_display_with_metadata() {
        let error = Error::new("Test error")
            .with_metadata("field", "email")
            .with_metadata("value", "invalid@");

        let display = format!("{}", error);
        assert!(display.contains("Metadata:"));
        assert!(display.contains("field: email"));
        assert!(display.contains("value: invalid@"));
    }

    #[test]
    fn test_error_display_with_source() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = Error::new("Configuration error").with_source(io_error);

        let display = format!("{}", error);
        assert!(display.contains("Error: Configuration error"));
        assert!(display.contains("Caused by:"));
        assert!(display.contains("File not found"));
    }

    #[test]
    fn test_error_display_comprehensive() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "config.toml not found");
        let error = Error::new("Failed to load configuration")
            .with_code(404)
            .with_metadata("file", "config.toml")
            .with_metadata("operation", "read")
            .with_source(io_error);

        let display = format!("{}", error);
        assert!(display.contains("Code: 404"));
        assert!(display.contains("Error: Failed to load configuration"));
        assert!(display.contains("Metadata:"));
        assert!(display.contains("file: config.toml"));
        assert!(display.contains("operation: read"));
        assert!(display.contains("Caused by:"));
        assert!(display.contains("config.toml not found"));
    }

    #[test]
    fn test_error_debug() {
        let error = Error::new("Test error").with_code(123);
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Error"));
        assert!(debug_str.contains("Test error"));
        assert!(debug_str.contains("123"));
    }

    #[test]
    fn test_std_error_trait() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = Error::new("App error").with_source(io_error);

        // Test that our Error implements std::error::Error
        let std_error: &dyn std::error::Error = &error;
        assert!(std_error.source().is_some());
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_error() -> Result<String> {
            Err(Error::new("Something went wrong"))
        }

        match returns_error() {
            Ok(_) => panic!("Expected error"),
            Err(e) => assert_eq!(e.message(), "Something went wrong"),
        }
    }

    #[test]
    fn test_builder_pattern_chaining() {
        let error = Error::new("Base error")
            .with_code(400)
            .with_metadata("type", "validation")
            .with_metadata("field", "email");

        assert_eq!(error.message(), "Base error");
        assert_eq!(error.code(), Some(400));
        assert_eq!(
            error.metadata().get("type"),
            Some(&"validation".to_string())
        );
        assert_eq!(error.metadata().get("field"), Some(&"email".to_string()));
    }

    #[test]
    fn test_error_with_backtrace() {
        let error = Error::new("Test error").with_backtrace();

        // We can't easily test the content of backtrace, but we can test that
        // the display format includes "Backtrace:" when backtrace is present
        let display = format!("{}", error);
        assert!(display.contains("Error: Test error"));
        // Note: Backtrace content will vary and may be empty in some environments
    }

    #[test]
    fn test_error_chain_iter_trait() {
        let root = Error::new("Root");
        let middle = Error::new("Middle").with_source(root);
        let top = Error::new("Top").with_source(middle);

        let mut iter = top.iter_error_chain();

        // Test Iterator trait methods
        assert!(iter.next().is_some()); // Top error
        assert!(iter.next().is_some()); // Middle error  
        assert!(iter.next().is_some()); // Root error
        assert!(iter.next().is_none()); // End of chain
    }

    #[test]
    fn test_error_chain_iter_collect() {
        let root = Error::new("Root");
        let top = Error::new("Top").with_source(root);

        let errors: Vec<_> = top.iter_error_chain().collect();
        assert_eq!(errors.len(), 2);

        // Can use other iterator methods
        let count = top.iter_error_chain().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_metadata_with_different_types() {
        let error = Error::new("Test")
            .with_metadata("number", 42.to_string())
            .with_metadata("boolean", true.to_string())
            .with_metadata("string", "value");

        assert_eq!(error.metadata().get("number"), Some(&"42".to_string()));
        assert_eq!(error.metadata().get("boolean"), Some(&"true".to_string()));
        assert_eq!(error.metadata().get("string"), Some(&"value".to_string()));
    }

    #[test]
    fn test_error_source_chain_depth() {
        // Create a deep error chain
        let mut current_error = Error::new("Level 0");

        for i in 1..=5 {
            let new_error = Error::new(format!("Level {}", i)).with_source(current_error);
            current_error = new_error;
        }

        let chain: Vec<_> = current_error.iter_error_chain().collect();
        assert_eq!(chain.len(), 6); // 0 through 5
    }

    #[test]
    fn test_error_with_code_from_environment() {
        use std::env;

        // Test with valid environment variable
        unsafe {
            env::set_var("ALTRIA_ERR_CODE", "500");
        }

        let error_code = env::var("ALTRIA_ERR_CODE")
            .ok()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);

        let error = Error::new("Environment-based error")
            .with_code(error_code)
            .with_metadata("source", "environment_variable");

        assert_eq!(error.code(), Some(500));
        assert_eq!(error.message(), "Environment-based error");
        assert_eq!(
            error.metadata().get("source"),
            Some(&"environment_variable".to_string())
        );

        // Test with invalid environment variable (non-numeric)
        unsafe {
            env::set_var("ALTRIA_ERR_CODE", "invalid");
        }

        let fallback_code = env::var("ALTRIA_ERR_CODE")
            .ok()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(999); // Fallback value

        let error_with_fallback = Error::new("Error with fallback code").with_code(fallback_code);

        assert_eq!(error_with_fallback.code(), Some(999));

        // Test without environment variable
        unsafe {
            env::remove_var("ALTRIA_ERR_CODE");
        }

        let default_code = env::var("ALTRIA_ERR_CODE")
            .ok()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1); // Default value

        let error_with_default = Error::new("Error with default code").with_code(default_code);

        assert_eq!(error_with_default.code(), Some(1));

        // Clean up
        unsafe {
            env::remove_var("ALTRIA_ERR_CODE");
        }
    }
}
