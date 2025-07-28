//! Example demonstrating the usage of the Error type

use std::collections::HashMap;

use altria::error::{Error, Result};

fn main() {
    println!("=== Altria Error Type Usage Examples ===\n");

    // Example 1: Simple error creation
    println!("1. Basic error creation:");
    let basic_error = Error::database("Connection to database failed");
    println!("{}\n", basic_error);

    // Example 2: Business error with custom code
    println!("2. Business error with custom code:");
    let business_error = Error::business("Invalid user operation", "USER_001");
    println!("{}\n", business_error);

    // Example 3: Error with metadata
    println!("3. Error with metadata:");
    let error_with_metadata = Error::validation("Input validation failed")
        .with_metadata("field", "email")
        .with_metadata("value", "invalid-email")
        .with_metadata("rule", "email_format");
    println!("{}\n", error_with_metadata);

    // Example 4: Error chain
    println!("4. Error chain:");
    let io_error = Error::io("Failed to read configuration file");
    let config_error = Error::config("Configuration loading failed").with_source(io_error);
    println!("{}\n", config_error);

    // Example 5: Complex error with all features
    println!("5. Complex error with all features:");
    let mut metadata = HashMap::new();
    metadata.insert("user_id".to_string(), "12345".to_string());
    metadata.insert("action".to_string(), "delete_account".to_string());
    metadata.insert("timestamp".to_string(), "2025-07-28T15:30:00Z".to_string());

    let permission_error = Error::auth("Insufficient permissions");
    let complex_error = Error::business("Account deletion failed", "ACCOUNT_DEL_001")
        .with_metadata_map(metadata)
        .with_source(permission_error);
    println!("{}\n", complex_error);

    // Example 6: Error type checking
    println!("6. Error type checking:");
    let errors = vec![
        Error::database("DB error"),
        Error::network("Network error"),
        Error::business("Business error", "BIZ_001"),
        Error::validation("Validation error"),
    ];

    for (i, error) in errors.iter().enumerate() {
        println!(
            "Error {}: Kind = {}, Is Business = {}, Is Database = {}",
            i + 1,
            error.kind(),
            error.is_business(),
            error.is_database()
        );
    }
    println!();

    // Example 7: Error chain traversal (using standard Error trait)
    println!("7. Error chain traversal (using std::error::Error):");
    let root_error = Error::io("File not accessible");
    let mid_error = Error::config("Config parse failed").with_source(root_error);
    let top_error = Error::internal("System initialization failed").with_source(mid_error);

    // Using standard library error chain
    let chain = top_error.error_chain();
    println!("Standard error chain has {} errors:", chain.len());
    for (i, error) in chain.iter().enumerate() {
        println!("  {}: {}", i + 1, error);
    }

    // Using our specific Altria error chain (for type-specific operations)
    let altria_chain = top_error.error_chain_as_altria_errors();
    println!("Altria error chain has {} errors:", altria_chain.len());
    for (i, error) in altria_chain.iter().enumerate() {
        println!("  {}: {} ({})", i + 1, error.message(), error.kind());
    }
    println!();

    // Example 8: Interoperability with standard library errors
    println!("8. Interoperability with standard library errors:");

    // Create a standard IO error
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");

    // Chain it with our Error
    let app_error = Error::config("Failed to load configuration")
        .with_source(io_error)
        .with_metadata("config_path", "/etc/myapp/config.toml");

    println!("App error with std::io::Error source:");
    println!("{}", app_error);

    // Demonstrate error chain traversal
    let mut current = &app_error as &(dyn std::error::Error);
    let mut level = 0;
    println!("\nError chain traversal:");
    loop {
        println!("  Level {}: {}", level, current);
        match current.source() {
            Some(source) => {
                current = source;
                level += 1;
            }
            None => break,
        }
    }
    println!();

    // Example 9: Using Result type
    println!("9. Using Result type:");
    match risky_operation() {
        Ok(value) => println!("Success: {}", value),
        Err(error) => {
            println!("Operation failed:");
            println!("{}", error);
        }
    }
}

/// Example function that returns a Result
fn risky_operation() -> Result<String> {
    // Simulate some operation that might fail
    let user_input = "invalid@email";

    if !user_input.contains('@') {
        return Err(Error::validation("Invalid email format")
            .with_code("EMAIL_001")
            .with_metadata("input", user_input)
            .with_metadata("expected_format", "user@domain.com"));
    }

    Ok("Operation completed successfully".to_string())
}
