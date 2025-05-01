use liblogger::{Logger, log_info, log_warn, log_error, shutdown_logger};
use std::{thread, time::Duration};

pub fn test_async_logger() {
    println!("Starting async logger test");
    
    // Initialize the logger from the same config file as main.rs uses
    match Logger::init_with_config_file("app_config.toml") {
        Ok(_) => println!("Async logger initialized successfully"),
        Err(e) => {
            eprintln!("Failed to initialize async logger: {}", e);
            return;
        }
    }
    
    // Generate a large number of log messages rapidly
    println!("Generating 1000 log messages...");
    for i in 0..1000 {
        log_info!(&format!("Async test message {}", i));
        
        if i % 100 == 0 {
            log_warn!(&format!("Warning message at {}", i));
        }
        
        if i % 250 == 0 {
            log_error!(&format!("Error message at {}", i));
        }
    }
    
    println!("Finished sending messages, waiting for processing...");
    // Give the async logger time to process the messages
    thread::sleep(Duration::from_secs(2));
    
    println!("Shutting down logger...");
    match shutdown_logger() {
        Ok(_) => println!("Logger shutdown successfully"),
        Err(e) => eprintln!("Error shutting down logger: {}", e),
    }
    
    println!("Async logger test completed");
}
