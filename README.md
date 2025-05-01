# Rusty Logger v2: Production-Ready Logging for Rust

## 1. Problem Statement

Modern distributed systems require effective logging for observability and debugging. Organizations face specific challenges when logging is implemented manually or inconsistently across services.

Developers often omit critical logs during development, leading to blind spots during incident response. When logs are present, they frequently lack essential context such as precise location, timing, and error cause. The formats vary between services, complicating automated parsing and analysis.

Without consistent entry and exit tracking in functions, tracing request flows becomes difficult during high-severity incidents. Many logs also miss crucial correlation identifiers like user IDs, request IDs, and session IDs that connect related events across distributed components.

Traditional synchronous logging can also block application execution during I/O operations, leading to performance degradation under high load. Asynchronous logging solves this problem by moving I/O operations off the critical path, but implementing it correctly requires careful handling of futures, channels, and thread safety.

The Rusty Logger v2 Framework addresses these operational challenges by providing structured, consistent logging capabilities that capture required context automatically while offering efficient asynchronous operations through Tokio integration.

---

## 2. Architecture Overview

The Rusty Logger v2 Framework provides logging capabilities with multiple output options. The system supports three output targets: Console for standard output, File with rotation functionality, and HTTP endpoints for remote logging aggregation.

Configuration occurs through an `app_config.toml` file, where users specify output type, log level thresholds, and target-specific parameters. The framework implements a singleton pattern, requiring a single `Logger::init()` call during program initialization, after which any component can use static methods like `Logger::info()` to record events.

Each log entry follows a consistent format that includes timestamp, severity level, file name, line number, module path, function name, and additional context. The framework employs procedural macros to capture this metadata automatically without requiring developers to specify it manually.

The asynchronous logging implementation uses Tokio to handle I/O operations in the background through message passing channels and task scheduling. This allows your application to continue execution without waiting for logs to be written to their destination. The library handles all the complexities of proper future pinning, message buffering, and graceful shutdown coordination transparently to the developer. If the async channel becomes full, the system automatically falls back to synchronous logging as a reliability measure.

---

## 3. Quick Start Guide

### Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
liblogger = { path = "../path/to/liblogger" }
liblogger_macros = { path = "../path/to/liblogger_macros" }
```

### Basic Usage in 3 Steps

1. **Create a configuration file** named `app_config.toml` in your project root:

```toml
[logging]
type = "console"  # Options: console, file, http
threshold = "debug"  # Options: debug, info, warn, error
file_path = "application.log"  # Used when type = "file"
log_folder = "logs"  # Directory where logs are stored 
max_file_size_mb = 10  # Rotation size when using file logging
http_endpoint = "https://logs.example.com"  # Used when type = "http"
http_timeout_seconds = 5  # HTTP request timeout
```

2. **Initialize the logger** in your application's entry point:

```rust
use liblogger::{Logger, log_info, log_error};

fn main() {
    // Initialize from config file
    match Logger::init_with_config_file("app_config.toml") {
        Ok(_) => log_info!("Logger initialized successfully"),
        Err(e) => {
            println!("Failed to initialize logger: {}", e);
            // Fall back to console logging
            Logger::init();
        }
    }
    
    // Your application code here
}
```

3. **Start logging** throughout your codebase:

```rust
use liblogger::{log_info, log_debug, log_warn, log_error};

fn process_order(order_id: &str) {
    log_debug!("Starting order processing");
    log_info!(&format!("Processing order {}", order_id));
    
    if order_id.is_empty() {
        log_warn!("Received empty order ID");
        return;
    }
    
    // Processing logic...
    
    if let Err(e) = validate_order(order_id) {
        log_error!(&format!("Order validation failed: {}", e));
    }
}
```

---

## 4. Configuration Details

### Configuration Options

| Parameter | Description | Default |
|-----------|-------------|---------|
| `type` | Output destination (`console`, `file`, `http`) | `console` |
| `threshold` | Minimum log level to record (`debug`, `info`, `warn`, `error`) | `info` |
| `file_path` | Log file name | `app.log` |
| `log_folder` | Directory for log files | `logs` |
| `max_file_size_mb` | Maximum file size before rotation | `10` |
| `http_endpoint` | URL for HTTP logging | `http://localhost:8080/logs` |
| `http_timeout_seconds` | HTTP request timeout | `5` |

### Sample Configurations

#### Console Logging
```toml
[logging]
type = "console"
threshold = "debug"
```

#### File Logging with Rotation
```toml
[logging]
type = "file"
threshold = "info"
file_path = "application.log"
log_folder = "logs"
max_file_size_mb = 5
```

#### Remote HTTP Logging
```toml
[logging]
type = "http"
threshold = "warn"
http_endpoint = "https://logging-service.example.com/ingest"
http_timeout_seconds = 3
```

---

## 5. Writing Logs

### Basic Logging Macros

The library provides four logging macros corresponding to different severity levels:

```rust
// Debug information useful during development
log_debug!("Connection pool initialized with 10 connections");

// Regular operational information
log_info!("User profile updated successfully");

// Warning conditions that should be addressed
log_warn!("Database connection pool running low (10% remaining)");

// Error conditions requiring attention
log_error!("Failed to process payment: timeout");
```

### Adding Context to Logs

You can add contextual information by providing an optional second parameter:

```rust
// With context as String
log_info!("User login successful", Some(format!("user_id={}", user_id)));

// With context as Option<String>
let context = if is_premium { Some("account=premium".to_string()) } else { None };
log_info!("Feature accessed", context);
```

### Logging in Asynchronous Code

Rusty Logger v2 seamlessly supports asynchronous code:

```rust
async fn process_data(user_id: &str) -> Result<(), Error> {
    log_info!(&format!("Starting data processing for user {}", user_id));
    
    let result = fetch_user_data(user_id).await;
    
    match result {
        Ok(data) => {
            log_info!("Data processing complete");
            Ok(())
        },
        Err(e) => {
            log_error!(&format!("Data processing failed: {}", e));
            Err(e)
        }
    }
}
```

---

## 6. Using Procedural Macros

### Step 1: Import and Initialize Macro Support

At the top of your source file, add:

```rust
use liblogger_macros::*;

// This brings all procedural macros into scope (must be at module level)
initialize_logger_attributes!();
```

### Step 2: Apply Macros to Functions

Annotate functions with the desired logging behaviors:

```rust
// Log function entry and exit points
#[log_entry_exit]
fn process_payment(payment_id: &str) {
    // Function implementation
}

// Measure and log execution time
#[measure_time]
fn generate_report() -> Report {
    // Time-consuming operation
}

// Log errors returned by the function
#[log_errors]
fn validate_input(data: &str) -> Result<(), ValidationError> {
    // Implementation that might return errors
}
```

### Common Macro Examples

#### Logging Entry and Exit
```rust
#[log_entry_exit]
fn update_user_profile(user_id: &str, profile_data: &ProfileData) {
    // Function implementation
}
// Produces logs like:
// "ENTRY: update_user_profile"
// "EXIT: update_user_profile"
```

#### Measuring Execution Time
```rust
#[measure_time]
fn process_large_dataset(data: &[DataPoint]) -> Analysis {
    // Time-consuming data processing
}
// Produces logs like:
// "process_large_dataset completed in 1250 ms"
```

#### Logging Function Arguments
```rust
#[log_args(user_id, action)]
fn audit_user_action(user_id: &str, action: &str, details: &ActionDetails) {
    // Only user_id and action will be logged
}
// Produces logs like:
// "Entering audit_user_action with args: user_id = "12345", action = "delete_account""
```

#### Retry Logic with Logging
```rust
#[log_retries(max_attempts=3)]
fn connect_to_database() -> Result<Connection, DbError> {
    // The function will be retried up to 3 times if it fails
}
// Produces logs like:
// "Retry attempt 1 for connect_to_database failed: connection refused"
// "Retry attempt 2 for connect_to_database succeeded"
```

#### Creating Audit Logs
```rust
#[audit_log]
fn change_permissions(user_id: &str, new_role: Role) {
    // Security-sensitive operation
}
// Produces logs like:
// "AUDIT: [general] Operation change_permissions started"
// "AUDIT: [general] Operation change_permissions completed | Context: result_type=success"
```

#### Error Handling and Logging
```rust
#[log_errors]
fn validate_transaction(transaction: &Transaction) -> Result<(), TransactionError> {
    // Function that might fail
}
// Produces logs when errors occur:
// "validate_transaction returned error: "insufficient funds""
```

---

## 7. Advanced Usage

### Combining Multiple Macros

You can stack macros to combine their functionality:

```rust
#[log_entry_exit]
#[measure_time]
#[log_errors]
fn critical_operation() -> Result<OperationResult, OperationError> {
    // Implementation
}
```

### Using Request Context

Track request flow across multiple functions:

```rust
#[trace_span]
fn handle_api_request(request: &Request) -> Response {
    // Will generate a trace ID
    process_request_data(request);
}

#[trace_span]
fn process_request_data(request: &Request) {
    // Will use the same trace ID as the parent function
}

// Produces logs like:
// "[TraceID: 748405dd-ce44-48bd-9f1a-86fdb5eae237] handle_api_request started"
// "[TraceID: 748405dd-ce44-48bd-9f1a-86fdb5eae237] process_request_data started"
```

### Graceful Shutdown

To ensure all pending logs are processed before your application exits:

```rust
fn main() {
    // Initialize logger
    Logger::init_with_config_file("app_config.toml").unwrap();
    
    // Application code...
    
    // Ensure all logs are flushed before exit
    liblogger::shutdown_logger().unwrap();
}
```

---

## 8. Performance Considerations

- **Asynchronous Operation**: By default, logs are processed asynchronously to avoid blocking your application.
- **Log Level Filtering**: Log messages below the configured threshold are filtered early to minimize overhead.
- **Channel Buffering**: The async logger uses a buffered channel (1024 messages) to handle bursts of log activity.
- **Fallback Mechanism**: If the async channel is full, the logger falls back to synchronous logging.

---

## 9. Usage Examples

### Basic Initialization Pattern

```rust
match Logger::init_with_config_file("app_config.toml") {
    Ok(_) => log_info!("Logger successfully initialized from config file"),
    Err(e) => {
        println!("Error initializing logger from config: {}", e);
        Logger::init(); // Fall back to default console logging
        log_error!("Failed to initialize file logger, falling back to console");
    }
}
```

### Asynchronous Logging Example

```rust
pub fn test_async_logger() {
    // Initialize from config file
    match Logger::init_with_config_file("app_config.toml") {
        Ok(_) => println!("Async logger initialized successfully"),
        Err(e) => {
            eprintln!("Failed to initialize async logger: {}", e);
            return;
        }
    }
    
    // Generate log messages rapidly
    for i in 0..1000 {
        log_info!(&format!("Async test message {}", i));
        
        if i % 100 == 0 {
            log_warn!(&format!("Warning message at {}", i));
        }
    }
    
    // Ensure logs are processed before shutdown
    shutdown_logger().unwrap();
}
```

---

## 10. Troubleshooting

### Common Issues

1. **Missing Logs**: Check if the log level is below your threshold in the config.
2. **File Permissions**: For file output, ensure your application has write permissions.
3. **Log Directory**: The library will try to create the log directory, but check permissions if this fails.
4. **Macro Errors**: Make sure you've called `initialize_logger_attributes!()` at the module level.
5. **Shutdown Issues**: If logs are missing at program exit, ensure you call `shutdown_logger()`.

### Getting Help

If you encounter issues not covered here, please open an issue on the GitHub repository with:
- Your configuration settings
- Code samples demonstrating the issue
- Expected vs. actual behavior