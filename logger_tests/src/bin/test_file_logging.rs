use liblogger::{Logger, log_info, log_warn, log_error, log_debug};
use std::fs;
use std::path::Path;

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
    if Path::new(log_dir).exists() {
        println!("✓ Log directory exists: {}", log_dir);
        
        if Path::new(log_file).exists() {
            println!("✓ Log file exists: {}", log_file);
            match fs::read_to_string(log_file) {
                Ok(content) => println!("Log file contents:\n{}", content),
                Err(e) => println!("Could not read log file: {}", e),
            }
        } else {
            println!("✗ Log file was not created: {}", log_file);
        }
    } else {
        println!("✗ Log directory was not created: {}", log_dir);
    }
    
    println!("====== TEST COMPLETE ======");
}
