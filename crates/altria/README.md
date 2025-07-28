# Altria

A comprehensive Rust library for web server development.

## Error Handling System

Altria provides an efficient, concise, and easy-to-use error handling system designed specifically for web development scenarios, fully integrated with Rust's standard library error chain mechanism.

### ✅ Key Features

1. **Error Type Naming**: Uses `Error` as the primary error type
2. **Fast Error Classification**: Supports `Database`, `Io`, `Network`, `Auth`, `Validation`, `Config`, `Business`, `External`, `Internal`, `Unknown` error categories
3. **Complete Error Information**:
   - Error message (required)
   - Custom error code (optional, mainly for business errors)
   - Error stack trace (automatically captured)
   - Error metadata (key-value pairs)
4. **Source Error Recording**: Complete error chain support using Rust's `std::error::Error` trait for full error propagation tracking
5. **Structured Output**: Prints in order: error message → error code → error type → metadata → stack trace → source error chain

### 🚀 Quick Start

```rust
use altria::{Error, Result};

// Basic error
let error = Error::database("Connection failed");

// Business error (with error code)
let business_error = Error::business("Invalid operation", "USER_001");

// Error with metadata
let detailed_error = Error::validation("Input validation failed")
    .with_metadata("field", "email")
    .with_metadata("value", "invalid@email.com");

// Error chain with standard library interoperability
use std::io;
let io_error = io::Error::new(io::ErrorKind::NotFound, "Config file not found");
let main_error = Error::config("Config loading failed")
    .with_source(io_error);

// Using in functions
fn risky_function() -> Result<String> {
    // Return error
    Err(Error::auth("Unauthorized")
        .with_code("AUTH_001")
        .with_metadata("user_id", "12345"))
}
```

### 📖 Documentation

For more usage examples and detailed information, please refer to:

- [API Documentation](https://docs.rs/altria) - Complete documentation with examples
- Run examples: `cargo run --example error_usage`

### 🧪 Testing

```bash
cargo test
```

### 🌟 Features

- **Zero-cost Abstractions**: Efficient error handling without performance overhead
- **Type Safety**: Compile-time error checking to prevent runtime errors
- **Complete Tracing**: Automatic stack capture with error chain support
- **Easy Debugging**: Structured error output for easy problem identification
- **Flexible Extension**: Support for custom metadata and error codes
- **Standard Library Integration**: Full compatibility with `std::error::Error` trait
- **Interoperability**: Works seamlessly with third-party error handling libraries (`anyhow`, `thiserror`, etc.)

### 🔗 Standard Library Compatibility

Altria's Error type implements `std::error::Error` trait, providing:

- **Native error chaining**: Use `source()` method for standard error traversal
- **Cross-library compatibility**: Chain with any `std::error::Error` implementation
- **Tool ecosystem support**: Works with error handling tools and debugging utilities

### 📋 Error Categories

- **Database**: Database connection and query errors
- **Io**: File system and I/O operation errors
- **Network**: HTTP requests and network communication errors
- **Auth**: Authentication and authorization errors
- **Validation**: Input validation and data format errors
- **Config**: Configuration loading and parsing errors
- **Business**: Custom business logic errors with error codes
- **External**: Third-party service integration errors
- **Internal**: Internal system and application errors
- **Unknown**: Fallback for unclassified errors
