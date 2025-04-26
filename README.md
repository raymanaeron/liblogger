# Rusty Logger v2

A flexible, attribute-based logging system for Rust applications.

## Features

- Multi-level logging (debug, info, warn, error)
- Procedural macros for aspect-oriented logging
- Automatic function entry/exit logging
- Performance measurement
- Error tracking and retry logic
- Circuit breaker pattern implementation
- And more...

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
liblogger = { path = "path/to/liblogger" }
liblogger_macros = { path = "path/to/liblogger_macros" }
```

## Usage

### Important Initialization Steps

When using Rusty Logger v2, you must follow these two important initialization steps:

1. **Call the `initialize_logger_attributes!()` macro** at the beginning of your module. This brings all the procedural macros into scope.
2. **Create a custom initializer function** to configure the logger according to your needs.

Here's an example of how to properly set up the logger:

```rust
use liblogger::{Logger, log_info, log_warn, log_error, log_debug};
use liblogger_macros::*;

// IMPORTANT: This macro must be called at the module level 
// to make all attribute macros available
initialize_logger_attributes!();

fn main() {
    // Initialize the logger with a custom initializer
    initialize_custom_logger();
    
    log_info!("Application started");
    
    // Your application code...
}

// Custom logger initialization function
fn initialize_custom_logger() {
    // Initialize logger with a specific config file
    match Logger::init_with_config_file("app_config.toml") {
        Ok(_) => log_info!("Logger successfully initialized from config file"),
        Err(e) => {
            // Something went wrong with the config file
            println!("Error initializing logger from config: {}", e);
            // Fall back to console logging
            Logger::init();
            log_error!("Failed to initialize file logger, falling back to console");
        }
    }
    
    // You can add additional initialization logic here
    log_info!("======== LOGGER INITIALIZED ========");
    log_debug!("Debug logging is enabled");
    log_info!("Info logging is enabled");
    log_warn!("Warning logging is enabled");
    log_error!("Error logging is enabled");
}
```

### Alternative: Using the Built-in Initializer

For simpler use cases, you can use the built-in initializer:

```rust
use liblogger::{log_info};
use liblogger_macros::*;

initialize_logger_attributes!();

fn main() {
    // Initialize with a specific config file
    liblogger::init("app_config.toml");
    
    log_info!("Application started");
    
    // Your application code...
}
```

## Logging Attributes

The library provides the following attributes for aspect-oriented logging:
