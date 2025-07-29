//! # Altria Error Handling
//!
//! Altria provides an efficient, concise, and easy-to-use error handling system
//! specifically for web development scenarios, fully integrated with Rust's standard
//! library error chain mechanism.
//!
//! ## Features
//!
//! ### 1. Configurable Backtrace Capture
//! Backtrace capture can be controlled for optimal performance:
//! - **Default Behavior**: Backtraces are not captured by default for better performance
//! - **Explicit Capture**: Use `with_backtrace()` method to capture backtrace when needed
//! - **Environment Control**: Standard `RUST_BACKTRACE` environment variable controls backtrace detail level
//!
//! ### 2. Quick Error Classification
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
//! - [`ErrorKind::Unknown`] - Unknown or unclassified errors
//!
//! ### 3. Complete Error Information
//! Each error contains:
//! - Error message (required)
//! - Error code (optional, mainly for business errors)
//! - Error type (automatic classification)
//! - Error stack trace (optional, configurable for performance)
//! - Metadata (optional key-value pairs)
//! - Source error chain (using Rust's standard library `std::error::Error` trait)
//!
//! ### 4. Standard Library Compatible Error Chain
//! - **Fully compatible with `std::error::Error` trait**: Can be used with any error type that implements `std::error::Error`
//! - **Standardized error chain traversal**: Uses the `source()` method for error chain recursion
//! - **Third-party library compatibility**: Works directly with libraries like `anyhow`, `thiserror`, etc.
//!
//! ### 5. Structured Error Output
//! Error output is formatted in the following order:
//! 1. Error message
//! 2. Error code (if present)
//! 3. Error type
//! 4. Metadata (if present)
//! 5. Error stack trace (if captured)
//! 6. Source error chain (if present)
//!
//! ## Basic Usage
//!
//! ### Backtrace Configuration
//!
//! ```rust
//! use altria::error::Error;
//!
//! // By default, no backtrace is captured (for performance)
//! let error = Error::database("Connection failed");
//! assert!(error.backtrace().is_none());
//!
//! // Explicitly capture backtrace when needed
//! let error_with_trace = Error::database("Critical error")
//!     .with_backtrace();
//! assert!(error_with_trace.backtrace().is_some());
//!
//! // The standard RUST_BACKTRACE environment variable controls
//! // backtrace formatting when displaying errors
//! ```
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
//! let business_error = Error::business(1001, "Invalid operation");
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
//!             .with_code(5001)
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
//! let complex_error = Error::business(2001, "Operation failed")
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

use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;

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
    code: Option<i32>,
    /// Stack trace captured at error creation (optional for performance)
    backtrace: Option<Backtrace>,
    /// Additional metadata
    metadata: HashMap<String, String>,
    /// Source error chain (using std::error::Error trait)
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl Error {
    /// Create a new error with the specified kind and message
    ///
    /// This is the basic constructor for creating errors. For convenience,
    /// consider using the specialized constructors like [`Error::database`],
    /// [`Error::validation`], etc.
    ///
    /// By default, backtrace is not captured for performance reasons.
    /// Use [`Error::with_backtrace`] to capture backtrace when needed.
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
            backtrace: None,
            metadata: HashMap::new(),
            source: None,
        }
    }

    // Convenience constructors and kind checking methods generated by macro above

    /// Get the error kind
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Get the error message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the error code
    pub fn code(&self) -> Option<i32> {
        self.code
    }

    /// Get the backtrace (if captured)
    pub fn backtrace(&self) -> Option<&Backtrace> {
        self.backtrace.as_ref()
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
    pub fn with_code(mut self, code: i32) -> Self {
        self.code = Some(code);
        self
    }

    /// Force enable backtrace for this error (captures if not already present)
    ///
    /// This method allows you to enable backtrace for a specific error instance,
    /// even if global backtrace capture is disabled.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use altria::error::Error;
    /// let error = Error::database("Critical database error")
    ///     .with_backtrace(); // Force capture backtrace for this error
    /// ```
    pub fn with_backtrace(mut self) -> Self {
        if self.backtrace.is_none() {
            self.backtrace = Some(Backtrace::capture());
        }
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

    // Error kind checking methods generated by macro above

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

        // 5. Stack trace (if captured)
        if let Some(backtrace) = &self.backtrace {
            writeln!(f, "Backtrace:")?;
            writeln!(f, "{}", backtrace)?;
        }

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

/// Master macro to define all error kinds and generate associated code
///
/// This macro is the single source of truth for all error types in the system.
/// It automatically generates:
/// - ErrorKind enum variants
/// - Constructor methods with different signatures (message-only or code+message)
/// - Kind checking methods (is_database, is_io, is_network, etc.)
/// - Display implementation for ErrorKind
/// - Documentation for all generated items
///
/// Constructor Types:
/// - `simple`: fn method_name(message: impl Into<String>) -> Self
/// - `with_code`: fn method_name(code: i32, message: impl Into<String>) -> Self
///
/// To add a new error kind, simply add it to this macro definition.
macro_rules! define_error_kinds {
    ($(
        $(#[$variant_doc:meta])*
        $variant:ident => {
            method: $method_name:ident,
            check_method: $check_method:ident,
            constructor: $constructor_type:ident,
            display: $display_name:literal,
            doc: $method_doc:literal,
            check_doc: $check_doc:literal,
        }
    ),* $(,)?) => {
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
            $(
                $(#[$variant_doc])*
                $variant,
            )*
        }

        impl fmt::Display for ErrorKind {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        ErrorKind::$variant => write!(f, $display_name),
                    )*
                }
            }
        }

        impl Error {
            // Generate constructor methods with different signatures
            $(
                define_error_kinds!(@constructor $constructor_type, $method_name, $variant, $method_doc);
            )*

            // Generate kind checking methods with explicit names
            $(
                #[doc = $check_doc]
                pub fn $check_method(&self) -> bool {
                    if let ErrorKind::$variant = self.kind {
                        true
                    } else {
                        false
                    }
                }
            )*
        }
    };

    // Internal macro for simple constructors (message only)
    (@constructor simple, $method_name:ident, $variant:ident, $method_doc:literal) => {
        #[doc = $method_doc]
        pub fn $method_name(message: impl Into<String>) -> Self {
            Self::new(ErrorKind::$variant, message)
        }
    };

    // Internal macro for constructors with code
    (@constructor with_code, $method_name:ident, $variant:ident, $method_doc:literal) => {
        #[doc = $method_doc]
        pub fn $method_name(code: i32, message: impl Into<String>) -> Self {
            Self::new(ErrorKind::$variant, message).with_code(code)
        }
    };
}

// Define all error kinds in one place - this is the single source of truth!
define_error_kinds! {
    /// Custom business logic errors
    Business => {
        method: business,
        check_method: is_business,
        constructor: with_code,
        display: "BUSINESS",
        doc: "Create a new business error with error code and message",
        check_doc: "Check if error is a business error",
    },
    /// Database-related errors
    Database => {
        method: database,
        check_method: is_database,
        constructor: simple,
        display: "DATABASE",
        doc: "Create a new database error",
        check_doc: "Check if error is a database error",
    },
    /// Input/Output errors
    Io => {
        method: io,
        check_method: is_io,
        constructor: simple,
        display: "IO",
        doc: "Create a new IO error",
        check_doc: "Check if error is an IO error",
    },
    /// Network/HTTP errors
    Network => {
        method: network,
        check_method: is_network,
        constructor: simple,
        display: "NETWORK",
        doc: "Create a new network error",
        check_doc: "Check if error is a network error",
    },
    /// Authentication/Authorization errors
    Auth => {
        method: auth,
        check_method: is_auth,
        constructor: simple,
        display: "AUTH",
        doc: "Create a new auth error",
        check_doc: "Check if error is an auth error",
    },
    /// Validation errors
    Validation => {
        method: validation,
        check_method: is_validation,
        constructor: simple,
        display: "VALIDATION",
        doc: "Create a new validation error",
        check_doc: "Check if error is a validation error",
    },
    /// Configuration errors
    Config => {
        method: config,
        check_method: is_config,
        constructor: simple,
        display: "CONFIG",
        doc: "Create a new config error",
        check_doc: "Check if error is a config error",
    },
    /// Cache-related errors
    Cache => {
        method: cache,
        check_method: is_cache,
        constructor: simple,
        display: "CACHE",
        doc: "Create a new cache error",
        check_doc: "Check if error is a cache error",
    },
    /// External service errors
    External => {
        method: external,
        check_method: is_external,
        constructor: simple,
        display: "EXTERNAL",
        doc: "Create a new external service error",
        check_doc: "Check if error is an external error",
    },
    /// Internal system errors
    Internal => {
        method: internal,
        check_method: is_internal,
        constructor: simple,
        display: "INTERNAL",
        doc: "Create a new internal error",
        check_doc: "Check if error is an internal error",
    },
    /// Unknown or unspecified errors
    Unknown => {
        method: unknown,
        check_method: is_unknown,
        constructor: simple,
        display: "UNKNOWN",
        doc: "Create a new unknown error",
        check_doc: "Check if error is an unknown error",
    },
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
        let err = Error::business(1001, "Invalid user input");
        assert!(err.is_business());
        assert_eq!(err.code(), Some(1001));
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
        // Test error with backtrace enabled
        let err = Error::business(1001, "Invalid operation")
            .with_metadata("user_id", "123")
            .with_metadata("action", "delete")
            .with_backtrace(); // Force backtrace capture

        let display_str = format!("{}", err);
        assert!(display_str.contains("Error: Invalid operation"));
        assert!(display_str.contains("Code: 1001"));
        assert!(display_str.contains("Kind: BUSINESS"));
        assert!(display_str.contains("user_id: 123"));
        assert!(display_str.contains("Backtrace:"));
    }

    #[test]
    fn test_error_kind_display() {
        assert_eq!(format!("{}", ErrorKind::Database), "DATABASE");
        assert_eq!(format!("{}", ErrorKind::Business), "BUSINESS");
        assert_eq!(format!("{}", ErrorKind::Validation), "VALIDATION");
        assert_eq!(format!("{}", ErrorKind::Cache), "CACHE");
    }

    #[test]
    fn test_new_cache_error_type() {
        let cache_err = Error::cache("Redis connection failed");
        assert!(cache_err.is_cache());
        assert_eq!(cache_err.kind(), &ErrorKind::Cache);
        assert_eq!(cache_err.message(), "Redis connection failed");
        assert!(!cache_err.is_database());
        assert!(!cache_err.is_business());
    }

    #[test]
    fn test_backtrace_configuration() {
        // Test that errors don't have backtrace by default
        let error_default = Error::database("Test error");
        assert!(error_default.backtrace().is_none());

        // Test explicit backtrace capture
        let error_with_backtrace = Error::database("Test error").with_backtrace();
        assert!(error_with_backtrace.backtrace().is_some());
    }
}
