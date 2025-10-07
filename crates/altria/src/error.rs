//! Error handling module for Altria
//!
//! Provides a flexible and efficient error type with the following features:
//! - Optional error code for API integration
//! - Required error message
//! - Optional source error for error chaining
//! - Optional backtrace for debugging
//! - Context key-value pairs for additional information
//! - Thread-safe and Send + Sync compatible

use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;

/// A flexible error type for the Altria library
///
/// This error type is designed to be:
/// - **Efficient**: Uses `Box` for optional fields to minimize size
/// - **Flexible**: Supports error codes, messages, source errors, and extra context
/// - **Debuggable**: Captures backtrace information when created
/// - **Thread-safe**: Implements `Send` and `Sync`
///
/// # Examples
///
/// ```
/// use altria::error::Error;
///
/// // Simple error with just a message
/// let err = Error::new("Something went wrong");
///
/// // Error with code and message
/// let err = Error::new("Not found").with_code(404);
///
/// // Error with context information
/// let err = Error::new("Database error")
///     .with_context_value("table", "users")
///     .with_context_value("operation", "insert");
/// ```
#[derive(Debug)]
pub struct Error {
    /// Optional error code (e.g., HTTP status code, custom error code)
    code: Option<i64>,
    /// Required error message
    message: String,
    /// Optional source error for error chain
    source: Option<Box<dyn StdError + Send + Sync>>,
    /// Optional backtrace for debugging
    backtrace: Option<Box<Backtrace>>,
    /// Context key-value pairs for additional information
    context: HashMap<String, String>,
}

impl Error {
    /// Create a new error with just a message
    ///
    /// By default, backtrace is not captured for performance.
    /// Use [`with_backtrace`](Self::with_backtrace) to enable it.
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::error::Error;
    ///
    /// let err = Error::new("Something went wrong");
    /// assert_eq!(err.message(), "Something went wrong");
    /// assert!(err.backtrace().is_none()); // No backtrace by default
    /// ```
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            code: None,
            message: message.into(),
            source: None,
            backtrace: None,
            context: HashMap::new(),
        }
    }

    /// Add an error code (builder pattern)
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::error::Error;
    ///
    /// let err = Error::new("Error").with_code(500);
    /// assert_eq!(err.code(), Some(500));
    /// ```
    #[must_use]
    pub const fn with_code(mut self, code: i64) -> Self {
        self.code = Some(code);
        self
    }

    /// Add a source error (builder pattern)
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::error::Error;
    /// use std::io;
    ///
    /// let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    /// let err = Error::new("Failed to read file").with_source(io_err);
    /// ```
    #[must_use]
    pub fn with_source(mut self, source: impl StdError + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Add a single context key-value pair (builder pattern)
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::error::Error;
    ///
    /// let err = Error::new("Database error")
    ///     .with_context_value("table", "users")
    ///     .with_context_value("operation", "insert");
    /// ```
    #[must_use]
    pub fn with_context_value(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Add multiple context key-value pairs at once from a `HashMap`
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::error::Error;
    /// use std::collections::HashMap;
    ///
    /// let mut ctx = HashMap::new();
    /// ctx.insert("table".to_string(), "users".to_string());
    /// ctx.insert("operation".to_string(), "insert".to_string());
    ///
    /// let err = Error::new("Database error").with_context_map(ctx);
    /// assert_eq!(err.get_context("table"), Some("users"));
    /// ```
    #[must_use]
    pub fn with_context_map(mut self, context: HashMap<String, String>) -> Self {
        self.context.extend(context);
        self
    }

    /// Enable backtrace capture for debugging
    ///
    /// Backtrace is disabled by default for performance. Use this method
    /// when you need detailed stack trace information for debugging.
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::error::Error;
    ///
    /// let err = Error::new("Error").with_backtrace();
    /// assert!(err.backtrace().is_some());
    /// ```
    #[must_use]
    pub fn with_backtrace(mut self) -> Self {
        self.backtrace = Some(Box::new(Backtrace::capture()));
        self
    }

    /// Get the error code
    #[must_use]
    pub const fn code(&self) -> Option<i64> {
        self.code
    }

    /// Get the error message
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the backtrace if available
    #[must_use]
    pub fn backtrace(&self) -> Option<&Backtrace> {
        self.backtrace.as_deref()
    }

    /// Get all context information as a `HashMap`
    #[must_use]
    pub const fn context(&self) -> &HashMap<String, String> {
        &self.context
    }

    /// Get a specific context value by key
    pub fn get_context(&self, key: &str) -> Option<&str> {
        self.context.get(key).map(String::as_str)
    }

    /// Returns an iterator over the entire error chain, starting from this error
    ///
    /// This iterator includes the current error as the first item,
    /// followed by all source errors in the chain.
    ///
    /// # Examples
    ///
    /// ```
    /// use altria::error::Error;
    /// use std::error::Error as StdError;
    /// use std::io;
    ///
    /// let io_err = io::Error::new(io::ErrorKind::NotFound, "file.txt not found");
    /// let err = Error::new("Failed to read config")
    ///     .with_source(io_err)
    ///     .with_code(500);
    ///
    /// let chain: Vec<String> = err.iter_error_chain()
    ///     .map(|e| e.to_string())
    ///     .collect();
    ///
    /// assert_eq!(chain.len(), 2); // Current error + 1 source
    /// assert!(chain[0].contains("Failed to read config"));
    /// ```
    #[must_use]
    pub fn iter_error_chain(&self) -> ErrorChainIter<'_> {
        ErrorChainIter {
            current: Some(self),
        }
    }
}

/// Iterator over the complete error chain
///
/// Created by the [`Error::iter_error_chain`] method. Iterates over the entire error chain,
/// starting from the current error and following the source chain.
#[derive(Debug, Clone)]
pub struct ErrorChainIter<'a> {
    current: Option<&'a (dyn StdError + 'static)>,
}

impl<'a> Iterator for ErrorChainIter<'a> {
    type Item = &'a (dyn StdError + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = current.source();
        Some(current)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(code) = self.code {
            write!(f, "[{}] {}", code, self.message)?;
        } else {
            write!(f, "{}", self.message)?;
        }

        if !self.context.is_empty() {
            write!(f, " (")?;
            let mut first = true;
            for (key, value) in &self.context {
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "{key}: {value}")?;
                first = false;
            }
            write!(f, ")")?;
        }

        Ok(())
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_deref()
            .map(|e| e as &(dyn StdError + 'static))
    }
}

// Ensure Error is Send + Sync for async safety
// The compiler will enforce this through the trait bounds on source
// and the use of standard library types for other fields
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    let _ = assert_send_sync::<Error>;
};

/// Convenience type alias for Result with our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Helper macro for creating errors quickly with builder pattern support
///
/// # Examples
///
/// ```
/// use altria::error;
///
/// // Simple error
/// let err = error!("Something went wrong");
///
/// // Error with code
/// let err = error!("Not found"; code: 404);
///
/// // Error with code and context
/// let err = error!("Database error"; code: 500, "table" => "users", "op" => "insert");
///
/// // Error with format string
/// let value = "request";
/// let err = error!("Failed to process {}", value);
///
/// // Error with format string and code (use semicolon before code)
/// let err = error!("User {} not found", "alice"; code: 404);
/// ```
#[macro_export]
macro_rules! error {
    // Format string with arguments, code, and context fields
    // error!("msg {}", arg; code: 404, "key" => "value", "key2" => "value2")
    ($fmt:literal, $($arg:expr),+ ; code: $code:expr, $($key:expr => $value:expr),+ $(,)?) => {{
        let mut err = $crate::error::Error::new(format!($fmt, $($arg),+))
            .with_code($code);
        $(
            err = err.with_context_value($key, $value);
        )+
        err
    }};
    // Format string with arguments and code
    // error!("msg {}", arg; code: 404)
    ($fmt:literal, $($arg:expr),+ ; code: $code:expr $(,)?) => {
        $crate::error::Error::new(format!($fmt, $($arg),+))
            .with_code($code)
    };
    // Format string with arguments and context fields
    // error!("msg {}", arg; "key" => "value")
    ($fmt:literal, $($arg:expr),+ ; $($key:expr => $value:expr),+ $(,)?) => {{
        let mut err = $crate::error::Error::new(format!($fmt, $($arg),+));
        $(
            err = err.with_context_value($key, $value);
        )+
        err
    }};
    // Simple message with code and context fields
    // error!("msg"; code: 404, "key" => "value")
    ($msg:expr ; code: $code:expr, $($key:expr => $value:expr),+ $(,)?) => {{
        let mut err = $crate::error::Error::new($msg).with_code($code);
        $(
            err = err.with_context_value($key, $value);
        )+
        err
    }};
    // Simple message with code
    // error!("msg"; code: 404)
    ($msg:expr ; code: $code:expr $(,)?) => {
        $crate::error::Error::new($msg).with_code($code)
    };
    // Simple message with context fields
    // error!("msg"; "key" => "value", "key2" => "value2")
    ($msg:expr ; $($key:expr => $value:expr),+ $(,)?) => {{
        let mut err = $crate::error::Error::new($msg);
        $(
            err = err.with_context_value($key, $value);
        )+
        err
    }};
    // Format string with arguments
    ($fmt:literal, $($arg:expr),+ $(,)?) => {
        $crate::error::Error::new(format!($fmt, $($arg),+))
    };
    // Simple string message
    ($msg:expr $(,)?) => {
        $crate::error::Error::new($msg)
    };
}

// Implement From for common error types for easy conversion
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::new("I/O error").with_source(err)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(err: std::fmt::Error) -> Self {
        Self::new("Formatting error").with_source(err)
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_error() {
        let err = Error::new("test error");
        assert_eq!(err.message(), "test error");
        assert_eq!(err.code(), None);
        assert!(err.backtrace().is_none()); // No backtrace by default
    }

    #[test]
    fn test_error_with_backtrace() {
        let err = Error::new("test error").with_backtrace();
        assert!(err.backtrace().is_some());
    }

    #[test]
    fn test_error_with_code() {
        let err = Error::new("not found").with_code(404);
        assert_eq!(err.message(), "not found");
        assert_eq!(err.code(), Some(404));
    }

    #[test]
    fn test_error_with_context() {
        let err = Error::new("database error")
            .with_context_value("table", "users")
            .with_context_value("operation", "insert");

        assert_eq!(err.get_context("table"), Some("users"));
        assert_eq!(err.get_context("operation"), Some("insert"));
        assert_eq!(err.get_context("nonexistent"), None);
    }

    #[test]
    fn test_error_display() {
        let err = Error::new("internal error").with_code(500);
        assert_eq!(err.to_string(), "[500] internal error");

        let err = Error::new("simple error");
        assert_eq!(err.to_string(), "simple error");

        let err = Error::new("error").with_context_value("key", "value");
        assert_eq!(err.to_string(), "error (key: value)");
    }

    #[test]
    fn test_error_source() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = Error::new("failed to read").with_source(io_err);

        assert!(err.source().is_some());
        assert_eq!(err.message(), "failed to read");
    }

    #[test]
    fn test_error_chain_iterator() {
        use std::io;

        // Chain with source
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file.txt not found");
        let err = Error::new("Failed to read config").with_source(io_err);

        let chain: Vec<_> = err.iter_error_chain().collect();
        assert_eq!(chain.len(), 2); // Current error + 1 source

        // No source - only current error
        let err = Error::new("simple error");
        let chain: Vec<_> = err.iter_error_chain().collect();
        assert_eq!(chain.len(), 1); // Only current error
    }

    #[test]
    fn test_error_macro() {
        let err = error!("simple");
        assert_eq!(err.message(), "simple");

        let err = error!("not found"; code: 404);
        assert_eq!(err.code(), Some(404));
        assert_eq!(err.message(), "not found");

        let value = "test";
        let err = error!("format {}", value);
        assert_eq!(err.message(), "format test");

        let err = error!("error"; "key" => "value", "key2" => "value2");
        assert_eq!(err.get_context("key"), Some("value"));
        assert_eq!(err.get_context("key2"), Some("value2"));

        let err = error!("error"; code: 500, "table" => "users");
        assert_eq!(err.code(), Some(500));
        assert_eq!(err.get_context("table"), Some("users"));

        let err = error!("user {}", "alice"; code: 404);
        assert_eq!(err.message(), "user alice");
        assert_eq!(err.code(), Some(404));
    }

    #[test]
    fn test_with_context_map() {
        use std::collections::HashMap;

        let mut ctx = HashMap::new();
        ctx.insert("table".to_string(), "users".to_string());
        ctx.insert("operation".to_string(), "insert".to_string());

        let err = Error::new("Database error").with_context_map(ctx);
        assert_eq!(err.get_context("table"), Some("users"));
        assert_eq!(err.get_context("operation"), Some("insert"));
    }

    #[test]
    fn test_object_safety() {
        // Test that Error can be used as a trait object (dynamic dispatch)
        let err = Error::new("test error").with_code(500);
        let trait_obj: &dyn StdError = &err;

        assert!(trait_obj.to_string().contains("test error"));

        // Test that we can store Error in a Box<dyn Error>
        let boxed: Box<dyn StdError + Send + Sync> = Box::new(err);
        assert!(boxed.to_string().contains("test error"));
    }

    #[test]
    fn test_builder_chaining() {
        // Test complete builder pattern chain
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file.txt");
        let err = Error::new("Complex error")
            .with_code(500)
            .with_source(io_err)
            .with_context_value("table", "users")
            .with_context_value("operation", "read")
            .with_backtrace();

        assert_eq!(err.code(), Some(500));
        assert_eq!(err.message(), "Complex error");
        assert!(err.source().is_some());
        assert_eq!(err.get_context("table"), Some("users"));
        assert_eq!(err.get_context("operation"), Some("read"));
        assert!(err.backtrace().is_some()); // Enabled via with_backtrace()
    }

    #[test]
    fn test_from_implementations() {
        let err: Error = "string error".into();
        assert_eq!(err.message(), "string error");

        let err: Error = String::from("owned string").into();
        assert_eq!(err.message(), "owned string");
    }

    #[test]
    fn test_result_type() {
        fn returns_result() -> Result<i32> {
            Err(Error::new("failed"))
        }

        assert!(returns_result().is_err());
    }
}
