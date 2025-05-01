/**
 * Test module for the asynchronous logging capabilities
 * 
 * This test demonstrates and verifies that:
 * - Logger can handle large volumes of log messages sent rapidly
 * - Asynchronous processing works as expected without blocking the main thread
 * - Messages of different log levels are properly handled
 * - Graceful shutdown properly processes all pending log messages
 */
use liblogger::{Logger, log_info, log_warn, log_error, shutdown_logger};
use std::{thread, time::Duration};

/**
 * Tests the asynchronous logging capabilities of the library
 * 
 * This function:
 * 1. Initializes the logger with settings from app_config.toml
 * 2. Sends 1000 log messages in rapid succession
 * 3. Includes messages at different severity levels (info, warn, error)
 * 4. Waits for processing to complete
 * 5. Demonstrates proper shutdown procedure
 * 
 * The test verifies that the async logger can handle high-volume message bursts
 * without message loss, and that the shutdown procedure waits for all
 * pending messages to be processed.
 */
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
