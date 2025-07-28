//! # Altria Error Handling
//!
//! Altria provides an efficient, concise, and easy-to-use error handling system designed
//! specifically for web development scenarios, fully integrated with Rust's standard
//! library error chain mechanism.
//!
//! ## Features
//!
//! ### 1. Quick Error Classification
//! Supports fast identification of various common error types:
//! - [`ErrorKind::Database`] - Database-related errors
//! - [`ErrorKind::Io`] - Input/output errors
//! - [`ErrorKind::Network`] - Network/HTTP errors
//! - [`ErrorKind::Auth`] - Authentication/authorization errors
//! - [`ErrorKind::Validation`] - Data validation errors
//! - [`ErrorKind::Config`] - Configuration errors
//! - [`ErrorKind::Business`] - Custom business logic errors
//! - [`ErrorKind::External`] - External service errors
//! - [`ErrorKind::Internal`] - Internal system errors
//!
//! ### 2. Complete Error Information
//! Each error contains:
//! - Error message (required)
//! - Error code (optional, mainly for business errors)
//! - Error type (automatic classification)
//! - Error stack trace (automatically captured)
//! - Metadata (optional key-value pairs)
//! - Source error chain (using Rust's standard library `std::error::Error` trait)
//!
//! ### 3. Standard Library Compatible Error Chain
//! - **Fully compatible with `std::error::Error` trait**: Can be used with any error type that implements `std::error::Error`
//! - **Standardized error chain traversal**: Uses the `source()` method for error chain recursion
//! - **Third-party library compatibility**: Works directly with libraries like `anyhow`, `thiserror`, etc.
//!
//! ### 4. Structured Error Output
//! Error output is formatted in the following order:
//! 1. Error message
//! 2. Error code (if present)
//! 3. Error type
//! 4. Metadata (if present)
//! 5. Error stack trace
//! 6. Source error chain (if present)
//!
//! ## Basic Usage
//!
//! ### Creating Errors
//!
//! ```rust
//! use altria::error::{Error, ErrorKind, Result};
//!
//! // Basic error
//! let error = Error::new(ErrorKind::Database, "Connection failed");
//!
//! // Convenience methods
//! let db_error = Error::database("Connection timeout");
//! let io_error = Error::io("File not found");
//! let net_error = Error::network("HTTP 500");
//! let auth_error = Error::auth("Invalid token");
//! let validation_error = Error::validation("Invalid email format");
//!
//! // Business error (with error code)
//! let business_error = Error::business("Invalid operation", "USER_001");
//! ```
//!
//! ### Adding Metadata
//!
//! ```rust
//! # use altria::error::Error;
//! let error = Error::validation("Input validation failed")
//!     .with_metadata("field", "email")
//!     .with_metadata("value", "invalid-email")
//!     .with_metadata("rule", "email_format");
//! ```
//!
//! ### Error Chaining
//!
//! ```rust
//! # use altria::error::Error;
//! let source_error = Error::io("File not found");
//! let main_error = Error::config("Configuration loading failed")
//!     .with_source(source_error);
//! ```
//!
//! ### Error Checking
//!
//! ```rust
//! # use altria::error::Error;
//! let error = Error::database("Connection failed");
//!
//! // Check error type
//! if error.is_database() {
//!     println!("Database error detected");
//! }
//!
//! // Get error information
//! println!("Kind: {}", error.kind());
//! println!("Message: {}", error.message());
//! if let Some(code) = error.code() {
//!     println!("Code: {}", code);
//! }
//! ```
//!
//! ### Using Result Type
//!
//! ```rust
//! # use altria::error::{Error, Result};
//! # fn some_condition_fails() -> bool { true }
//! fn risky_operation() -> Result<String> {
//!     // Simulate potentially failing operation
//!     if some_condition_fails() {
//!         return Err(Error::validation("Invalid input")
//!             .with_code("VALIDATION_001")
//!             .with_metadata("field", "user_id"));
//!     }
//!
//!     Ok("Success".to_string())
//! }
//!
//! // Usage
//! match risky_operation() {
//!     Ok(value) => println!("Success: {}", value),
//!     Err(error) => {
//!         eprintln!("Error occurred:");
//!         eprintln!("{}", error);
//!     }
//! }
//! ```
//!
//! ### Error Conversion
//!
//! The system automatically supports conversion of common error types:
//!
//! ```rust
//! # use altria::error::Result;
//! use std::fs::File;
//!
//! fn read_file() -> Result<String> {
//!     let _file = File::open("config.txt")?; // Automatically converts std::io::Error
//!     // ... other processing
//!     Ok("file content".to_string())
//! }
//! ```
//!
//! ## Advanced Features
//!
//! ### Error Chain Traversal
//!
//! ```rust
//! # use altria::error::Error;
//! # let error = Error::database("test");
//! let error_chain = error.error_chain();
//! for (i, err) in error_chain.iter().enumerate() {
//!     println!("Level {}: {}", i, err);
//! }
//! ```
//!
//! ### Complex Error Construction
//!
//! ```rust
//! # use altria::error::Error;
//! use std::collections::HashMap;
//!
//! let mut metadata = HashMap::new();
//! metadata.insert("user_id".to_string(), "12345".to_string());
//! metadata.insert("action".to_string(), "delete".to_string());
//! metadata.insert("timestamp".to_string(), "2025-07-28T15:30:00Z".to_string());
//!
//! let source_error = Error::auth("Insufficient permissions");
//! let complex_error = Error::business("Operation failed", "BIZ_001")
//!     .with_metadata_map(metadata)
//!     .with_source(source_error);
//! ```
//!
//! ### Standard Library Error Chain Integration
//!
//! ```rust
//! # use altria::error::Error;
//! use std::io;
//!
//! // Interoperability with standard library errors
//! let io_error = io::Error::new(io::ErrorKind::NotFound, "Config file not found");
//! let app_error = Error::config("Failed to load application config")
//!     .with_source(io_error)
//!     .with_metadata("config_path", "/etc/app/config.toml");
//!
//! // Standard library error chain traversal
//! let mut current = &app_error as &(dyn std::error::Error);
//! while let Some(source) = current.source() {
//!     println!("Caused by: {}", source);
//!     current = source;
//! }
//!
//! // Our own error chain traversal
//! let chain = app_error.error_chain();
//! for (i, error) in chain.iter().enumerate() {
//!     println!("Level {}: {}", i, error);
//! }
//! ```
//!
//! ## Design Principles
//!
//! 1. **Efficient**: Zero-overhead abstractions, minimize runtime costs
//! 2. **Concise**: Provide convenient APIs, reduce boilerplate code
//! 3. **Easy to use**: Clear error messages and structured output
//! 4. **Complete**: Support error chains, metadata, stack traces, and other advanced features
//!
//! ## Example Output
//!
//! ```text
//! Error: Account deletion failed
//! Code: ACCOUNT_DEL_001
//! Kind: BUSINESS
//! Metadata:
//!   action: delete_account
//!   timestamp: 2025-07-28T15:30:00Z
//!   user_id: 12345
//! Backtrace:
//!    0: altria::error::Error::business
//!              at /path/to/src/error.rs:91:40
//!    1: main
//!              at /path/to/examples/error_usage.rs:43:25
//! Caused by:
//!   Error: Insufficient permissions
//!   Kind: AUTH
//!   Backtrace:
//!      0: altria::error::Error::auth
//!                at /path/to/src/error.rs:114:9
//!      1: main
//!                at /path/to/examples/error_usage.rs:42:28
//! ```

use std::collections::HashMap;
use std::fmt;

use backtrace::Backtrace;
use serde::{Deserialize, Serialize};

/// Comprehensive error type for web development
///
/// The `Error` struct provides a rich error handling system that captures:
/// - Error classification through [`ErrorKind`]
/// - Human-readable error messages
/// - Optional error codes (particularly useful for business logic errors)
/// - Automatic stack trace capture
/// - Key-value metadata for additional context
/// - Error chain support using Rust's standard `std::error::Error` trait
///
/// # Examples
///
/// Basic error creation:
/// ```rust
/// # use altria::error::{Error, ErrorKind};
/// let error = Error::new(ErrorKind::Database, "Connection failed");
/// assert!(error.is_database());
/// assert_eq!(error.message(), "Connection failed");
/// ```
///
/// Error with metadata and chaining:
/// ```rust
/// # use altria::error::Error;
/// let source = Error::io("File not found");
/// let error = Error::config("Failed to load config")
///     .with_source(source)
///     .with_metadata("file_path", "/etc/config.toml")
///     .with_metadata("retry_count", "3");
/// ```
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
///
/// This enum provides semantic categorization of errors commonly encountered
/// in web development. Each variant represents a different category of error
/// that might require different handling strategies.
///
/// # Examples
///
/// ```rust
/// # use altria::error::{Error, ErrorKind};
/// let db_error = Error::new(ErrorKind::Database, "Connection timeout");
/// let auth_error = Error::new(ErrorKind::Auth, "Invalid credentials");
///
/// match db_error.kind() {
///     ErrorKind::Database => println!("Handle database error"),
///     ErrorKind::Auth => println!("Handle auth error"),
///     _ => println!("Handle other errors"),
/// }
/// ```
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
    ///
    /// This is the basic constructor for creating errors. For convenience,
    /// consider using the specialized constructors like [`Error::database`],
    /// [`Error::validation`], etc.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use altria::error::{Error, ErrorKind};
    /// let error = Error::new(ErrorKind::Database, "Connection failed");
    /// assert_eq!(error.message(), "Connection failed");
    /// assert!(error.is_database());
    /// ```
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
    ///
    /// Business errors are typically used for application-specific logic errors
    /// that need to be communicated to end users or other services. The error code
    /// can be used for programmatic error handling.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use altria::error::Error;
    /// let error = Error::business("Invalid operation", "USER_001");
    /// assert!(error.is_business());
    /// assert_eq!(error.code(), Some("USER_001"));
    /// ```
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
    /// let error = Error::validation("Invalid input")
    ///     .with_metadata("field", "email")
    ///     .with_metadata("value", "invalid@")
    ///     .with_metadata("rule", "email_format");
    ///
    /// assert_eq!(error.metadata().get("field"), Some(&"email".to_string()));
    /// ```
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
    /// let app_error = Error::config("Failed to load config")
    ///     .with_source(io_error);
    ///
    /// // You can traverse the error chain using std::error::Error::source()
    /// assert!(app_error.source().is_some());
    /// ```
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
    ///
    /// This method returns a vector of all errors in the chain, starting with
    /// the current error and following the source chain. This is the standard
    /// way to traverse error chains in Rust.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use altria::error::Error;
    /// let source = Error::io("File not found");
    /// let main_error = Error::config("Config load failed")
    ///     .with_source(source);
    ///
    /// let chain = main_error.error_chain();
    /// assert_eq!(chain.len(), 2);
    /// // chain[0] is the main_error
    /// // chain[1] is the source error
    /// ```
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

/// Convenient type alias for `std::result::Result<T, Error>`
///
/// This type alias provides a shorter way to work with results that use our [`Error`] type.
///
/// # Examples
///
/// ```rust
/// # use altria::error::{Error, Result};
/// fn might_fail() -> Result<String> {
///     if some_condition() {
///         Ok("success".to_string())
///     } else {
///         Err(Error::validation("Something went wrong"))
///     }
/// }
/// # fn some_condition() -> bool { true }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

/// Error category enum for quick error type identification
///
/// This enum provides semantic categorization of errors commonly encountered
/// in web development. Each variant represents a different category of error
/// that might require different handling strategies.
///
/// # Examples
///
/// ```rust
/// # use altria::error::{Error, ErrorKind};
/// let db_error = Error::new(ErrorKind::Database, "Connection timeout");
/// let auth_error = Error::new(ErrorKind::Auth, "Invalid credentials");
///
/// match db_error.kind() {
///     ErrorKind::Database => println!("Handle database error"),
///     ErrorKind::Auth => println!("Handle auth error"),
///     _ => println!("Handle other errors"),
/// }
/// ```
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
