//! Error handling utilities for web development
//!
//! This module provides a comprehensive error type that captures error information,
//! error codes, stack traces, metadata, and source errors for efficient debugging
//! and error handling in web applications.

use std::collections::HashMap;
use std::fmt;

use backtrace::Backtrace;
use serde::{Deserialize, Serialize};

/// Comprehensive error type for web development
#[derive(Debug)]
pub struct Error {
    /// Error category for quick identification
    kind: ErrorKind,
    /// Primary error message
    message: String,
    /// Custom error code (mainly for business errors)
    code: Option<String>,
    /// Stack trace captured at error creation
    backtrace: String,
    /// Additional metadata
    metadata: HashMap<String, String>,
    /// Source error chain (using std::error::Error trait)
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

/// Error category enum for quick error type identification
impl Clone for Error {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind.clone(),
            message: self.message.clone(),
            code: self.code.clone(),
            backtrace: self.backtrace.clone(),
            metadata: self.metadata.clone(),
            // Note: We lose the source chain when cloning because trait objects can't be cloned
            // This is a limitation when using std::error::Error trait objects
            source: None,
        }
    }
}

impl Error {
    /// Create a new error with the specified kind and message
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            backtrace: format!("{:?}", Backtrace::new()),
            metadata: HashMap::new(),
            source: None,
        }
    }

    /// Create a new business error with custom error code
    pub fn business(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Business,
            message: message.into(),
            code: Some(code.into()),
            backtrace: format!("{:?}", Backtrace::new()),
            metadata: HashMap::new(),
            source: None,
        }
    }

    /// Create a new database error
    pub fn database(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Database, message)
    }

    /// Create a new IO error
    pub fn io(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Io, message)
    }

    /// Create a new network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Network, message)
    }

    /// Create a new auth error
    pub fn auth(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Auth, message)
    }

    /// Create a new validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Validation, message)
    }

    /// Create a new config error
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Config, message)
    }

    /// Create a new external service error
    pub fn external(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::External, message)
    }

    /// Create a new internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, message)
    }

    /// Get the error kind
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Get the error message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the error code
    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    /// Get the backtrace
    pub fn backtrace(&self) -> &str {
        &self.backtrace
    }

    /// Get the metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Get the source error
    pub fn source_error(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)> {
        self.source.as_deref()
    }

    /// Set custom error code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add multiple metadata entries
    pub fn with_metadata_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    /// Chain with source error
    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    /// Check if error is of specific kind
    pub fn is_kind(&self, kind: &ErrorKind) -> bool {
        &self.kind == kind
    }

    /// Check if error is a database error
    pub fn is_database(&self) -> bool {
        self.is_kind(&ErrorKind::Database)
    }

    /// Check if error is an IO error
    pub fn is_io(&self) -> bool {
        self.is_kind(&ErrorKind::Io)
    }

    /// Check if error is a network error
    pub fn is_network(&self) -> bool {
        self.is_kind(&ErrorKind::Network)
    }

    /// Check if error is an auth error
    pub fn is_auth(&self) -> bool {
        self.is_kind(&ErrorKind::Auth)
    }

    /// Check if error is a validation error
    pub fn is_validation(&self) -> bool {
        self.is_kind(&ErrorKind::Validation)
    }

    /// Check if error is a business error
    pub fn is_business(&self) -> bool {
        self.is_kind(&ErrorKind::Business)
    }

    /// Check if error is a config error
    pub fn is_config(&self) -> bool {
        self.is_kind(&ErrorKind::Config)
    }

    /// Get error chain using std::error::Error's source mechanism
    pub fn error_chain(&self) -> Vec<&(dyn std::error::Error + 'static)> {
        let mut chain = vec![self as &(dyn std::error::Error + 'static)];
        let mut current = self as &(dyn std::error::Error + 'static);

        while let Some(source) = current.source() {
            chain.push(source);
            current = source;
        }

        chain
    }

    /// Get error chain as our Error types (for testing and specific use cases)
    pub fn error_chain_as_altria_errors(&self) -> Vec<&Error> {
        let mut chain = vec![self];
        let mut current_source = self.source.as_deref();

        while let Some(source) = current_source {
            if let Some(altria_error) = source.downcast_ref::<Error>() {
                chain.push(altria_error);
                current_source = altria_error.source.as_deref();
            } else {
                break;
            }
        }

        chain
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 1. Error message
        writeln!(f, "Error: {}", self.message)?;

        // 2. Error code (if present)
        if let Some(code) = &self.code {
            writeln!(f, "Code: {code}")?;
        }

        // 3. Error kind
        writeln!(f, "Kind: {}", self.kind)?;

        // 4. Metadata (if present)
        if !self.metadata.is_empty() {
            writeln!(f, "Metadata:")?;
            for (key, value) in &self.metadata {
                writeln!(f, "  {key}: {value}")?;
            }
        }

        // 5. Stack trace
        writeln!(f, "Backtrace:")?;
        writeln!(f, "{}", self.backtrace)?;

        // 6. Source error chain
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

// Conversion implementations for common error types
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::io(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::validation(format!("JSON serialization error: {err}"))
    }
}

// Result type alias for convenience
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorKind {
    /// Database-related errors
    Database,
    /// Input/Output errors
    Io,
    /// Network/HTTP errors
    Network,
    /// Authentication/Authorization errors
    Auth,
    /// Validation errors
    Validation,
    /// Configuration errors
    Config,
    /// Custom business logic errors
    Business,
    /// External service errors
    External,
    /// Internal system errors
    Internal,
    /// Unknown/Other errors
    Unknown,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Database => write!(f, "DATABASE"),
            ErrorKind::Io => write!(f, "IO"),
            ErrorKind::Network => write!(f, "NETWORK"),
            ErrorKind::Auth => write!(f, "AUTH"),
            ErrorKind::Validation => write!(f, "VALIDATION"),
            ErrorKind::Config => write!(f, "CONFIG"),
            ErrorKind::Business => write!(f, "BUSINESS"),
            ErrorKind::External => write!(f, "EXTERNAL"),
            ErrorKind::Internal => write!(f, "INTERNAL"),
            ErrorKind::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::new(ErrorKind::Database, "Connection failed");
        assert_eq!(err.kind(), &ErrorKind::Database);
        assert_eq!(err.message(), "Connection failed");
        assert!(err.code().is_none());
    }

    #[test]
    fn test_business_error_with_code() {
        let err = Error::business("Invalid user input", "USER_001");
        assert!(err.is_business());
        assert_eq!(err.code(), Some("USER_001"));
    }

    #[test]
    fn test_error_with_metadata() {
        let err = Error::database("Query failed")
            .with_metadata("table", "users")
            .with_metadata("query_id", "12345");

        assert_eq!(err.metadata().get("table"), Some(&"users".to_string()));
        assert_eq!(err.metadata().get("query_id"), Some(&"12345".to_string()));
    }

    #[test]
    fn test_error_chain() {
        let source_err = Error::io("File not found");
        let main_err = Error::config("Configuration loading failed").with_source(source_err);

        let chain = main_err.error_chain_as_altria_errors();
        assert_eq!(chain.len(), 2);
        assert!(chain[0].is_config());
        assert!(chain[1].is_io());

        // Test standard library error chain
        let std_chain = main_err.error_chain();
        assert_eq!(std_chain.len(), 2);
    }

    #[test]
    fn test_error_display() {
        let err = Error::business("Invalid operation", "BIZ_001")
            .with_metadata("user_id", "123")
            .with_metadata("action", "delete");

        let display_str = format!("{}", err);
        assert!(display_str.contains("Error: Invalid operation"));
        assert!(display_str.contains("Code: BIZ_001"));
        assert!(display_str.contains("Kind: BUSINESS"));
        assert!(display_str.contains("user_id: 123"));
        assert!(display_str.contains("Backtrace:"));
    }

    #[test]
    fn test_error_kind_display() {
        assert_eq!(format!("{}", ErrorKind::Database), "DATABASE");
        assert_eq!(format!("{}", ErrorKind::Business), "BUSINESS");
        assert_eq!(format!("{}", ErrorKind::Validation), "VALIDATION");
    }
}
