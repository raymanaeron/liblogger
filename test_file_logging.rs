use liblogger::{Logger, log_info, log_warn, log_error, log_debug};
use std::fs;

fn main() {
    println!("====== FILE LOGGING TEST ======");
    
    // Print current directory
    let current_dir = std::env::current_dir().unwrap();
    println!("Current directory: {:?}", current_dir);
    
    // Check if config file exists
    let config_path = "app_config.toml";
    if let Ok(contents) = fs::read_to_string(config_path) {
        println!("Found config file with content:\n{}", contents);
    } else {
        println!("Could not read config file: {}", config_path);
    }
    
    // Initialize logger with config file
    match Logger::init_with_config_file(config_path) {
        Ok(_) => println!("Logger initialized successfully"),
        Err(e) => println!("Failed to initialize logger: {}", e),
    }
    
    // Write some log messages
    log_debug!("This is a DEBUG message");
    log_info!("This is an INFO message");
    log_warn!("This is a WARN message");
    log_error!("This is an ERROR message");
    
    // Check if log directory and file were created
    let log_dir = "logs";
    let log_file = "logs/workflow.log";
    
    println!("\nChecking if logs were written:");
    if let Ok(metadata) = fs::metadata(log_dir) {
        if metadata.is_dir() {
            println!("✓ Log directory created: {}", log_dir);
        } else {
            println!("✗ Log directory path exists but is not a directory: {}", log_dir);
        }
    } else {
        println!("✗ Log directory was not created: {}", log_dir);
    }
    
    if let Ok(metadata) = fs::metadata(log_file) {
        if metadata.is_file() {
            println!("✓ Log file created: {}", log_file);
            if let Ok(content) = fs::read_to_string(log_file) {
                println!("Log file contents:\n{}", content);
            }
        } else {
            println!("✗ Log file path exists but is not a file: {}", log_file);
        }
    } else {
        println!("✗ Log file was not created: {}", log_file);
    }
    
    println!("====== TEST COMPLETE ======");
}
