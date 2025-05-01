/*
 * Logger implementation module for Rusty Logger v2
 * 
 * This file implements the core Logger functionality which includes:
 * - Creation and initialization of the global logger instance
 * - Configuration of the logger from TOML files or programmatically
 * - Asynchronous logging through Tokio with message passing
 * - Automatic fallback to synchronous logging when needed
 * - Thread-safe logging with proper synchronization
 * 
 * The Logger uses a singleton pattern with lazy initialization via OnceCell
 * to ensure there's only one logger instance throughout the application.
 */

use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex};
use std::path::Path;
use chrono::Utc;
use std::io::{self, Write};
use tokio::sync::mpsc::{self, Sender, Receiver};
use tokio::runtime::Runtime;

use crate::config::{LogConfig, LogLevel};
use crate::outputs::{LogOutput, create_log_output, create_async_log_output, AsyncLogOutputTrait};
use crate::outputs::AsyncLogOutput;

// Global logger instance
static LOGGER_INSTANCE: OnceCell<Arc<Mutex<LoggerInner>>> = OnceCell::new();
static RUNTIME: OnceCell<Runtime> = OnceCell::new();

// Message structure for async logging channel
struct LogMessage {
    timestamp: String,
    level: LogLevel,
    message: String,
    context: Option<String>,
    file: String,
    line: u32,
    module: String,
}

struct LoggerInner {
    initialized: bool,
    config: Option<LogConfig>,
    output: Option<Box<dyn LogOutput>>,
    // Channel sender for async logging
    async_sender: Option<Sender<LogMessage>>,
    /// Flag to indicate if asynchronous logging is enabled
    /// When false, all logging operations will be synchronous
    async_enabled: bool,
}

impl LoggerInner {
    /// Creates a new uninitialized logger inner structure
    fn new() -> Self {
        LoggerInner {
            initialized: false,
            config: None,
            output: None,
            async_sender: None,
            async_enabled: false,
        }
    }

    /// Initializes the logger with the provided configuration
    fn init_with_config(&mut self, config: LogConfig) -> Result<(), String> {
        println!("Setting up logger with log type: {:?}", config.log_type);
        
        // Create the appropriate log output based on configuration
        let output = create_log_output(&config.log_type)?;
        self.output = Some(output);
        
        // Set up async logging if enabled
        if config.async_logging {
            // Create Tokio runtime if not already initialized
            let runtime = RUNTIME.get_or_init(|| {
                Runtime::new().expect("Failed to create Tokio runtime")
            });
            
            // Create channel for async logging
            let (tx, rx) = mpsc::channel::<LogMessage>(100);
            self.async_sender = Some(tx);
            
            // Create the async output
            let async_output = create_async_log_output(&config.log_type)?;
            
            // Spawn a task to process log messages
            runtime.spawn(async move {
                process_log_messages(rx, async_output).await
                    .unwrap_or_else(|e| eprintln!("Async logging failed: {}", e));
            });
        }
        
        // Store the configuration
        self.config = Some(config.clone());
        self.async_enabled = config.async_logging;
        self.initialized = true;
        
        Ok(())
    }

    /// Log a message with the configured output
    fn log(&mut self, level: LogLevel, message: &str, context: Option<&str>, file: &str, line: u32, module: &str) {
        // Check if we're initialized with a configuration
        if let Some(ref config) = self.config {
            // Skip logging if level is below threshold
            if (level.clone() as usize) < (config.threshold.clone() as usize) {
                return;
            }
            
            // Format timestamp
            let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
            
            // Try async logging first if enabled
            if self.async_enabled {
                if let Some(ref sender) = self.async_sender {
                    // Create a log message for the async channel
                    let log_message = LogMessage {
                        timestamp: timestamp.clone(),
                        level: level.clone(),
                        message: message.to_string(),
                        context: context.map(|s| s.to_string()),
                        file: file.to_string(),
                        line,
                        module: module.to_string(),
                    };
                    
                    // Send to the async channel, fallback to sync if channel is full
                    if let Err(_) = sender.try_send(log_message) {
                        // Channel full or closed, fallback to sync logging
                        self.log_sync(&timestamp, &level, message, context, file, line, module);
                    }
                } else {
                    // Async sender not initialized, fallback to sync logging
                    self.log_sync(&timestamp, &level, message, context, file, line, module);
                }
            } else {
                // Async logging disabled, use sync logging
                self.log_sync(&timestamp, &level, message, context, file, line, module);
            }
        } else {
            // Fallback to stderr for uninitialized logger
            let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
            self.log_sync(&timestamp, &level, message, context, file, line, module);
        }
    }

    /// Synchronous logging fallback
    fn log_sync(&mut self, timestamp: &str, level: &LogLevel, message: &str, 
                context: Option<&str>, file: &str, line: u32, module: &str) {
        if let Some(ref mut output) = self.output {
            // Format the log message
            let formatted_message = format_log_message(timestamp, level, message, context, file, line, module);
            
            // Write the log
            if let Err(e) = output.write_log(&formatted_message) {
                eprintln!("Failed to write log: {}", e);
            }
        } else {
            // No output configured, write to stderr
            let level_str = level.as_str();
            let log_line = match context {
                Some(ctx) => format!("{} [{}] [{}:{}] [{}] {} | {}\n", 
                    timestamp, level_str, file, line, module, message, ctx),
                None => format!("{} [{}] [{}:{}] [{}] {}\n",
                    timestamp, level_str, file, line, module, message),
            };
            
            let _ = io::stderr().write_all(log_line.as_bytes());
        }
    }
}

// Format a log message for output
fn format_log_message(timestamp: &str, level: &LogLevel, message: &str, 
                    context: Option<&str>, file: &str, line: u32, module: &str) -> String {
    let level_str = level.as_str();
    match context {
        Some(ctx) => format!("{} [{}] [{}:{}] [{}] {} | {}", 
            timestamp, level_str, file, line, module, message, ctx),
        None => format!("{} [{}] [{}:{}] [{}] {}",
            timestamp, level_str, file, line, module, message),
    }
}

// Async function to process log messages from the channel
async fn process_log_messages(mut receiver: Receiver<LogMessage>, mut output: AsyncLogOutput) -> Result<(), String> {
    while let Some(msg) = receiver.recv().await {
        // Format the log message
        let formatted_message = format_log_message(
            &msg.timestamp, &msg.level, &msg.message, 
            msg.context.as_deref(), &msg.file, msg.line, &msg.module);
        
        // Write using the async output
        if let Err(e) = output.write_log_async(&formatted_message).await {
            eprintln!("Async logging error: {}", e);
        }
    }
    
    Ok(())
}

pub struct Logger;

impl Logger {
    /// Initialize the logger with default configuration file "app_config.toml"
    pub fn init() {
        let _ = Self::init_with_config_file("app_config.toml");
    }

    /// Initialize the logger with a specific configuration file
    pub fn init_with_config_file(config_path: &str) -> Result<(), String> {
        let config = LogConfig::from_file(config_path)?;
        Self::init_with_config(config)
    }

    /// Initialize the logger with a LogConfig struct
    pub fn init_with_config(config: LogConfig) -> Result<(), String> {
        println!("Setting up logger with log type: {:?}", config.log_type);
        
        let logger = LOGGER_INSTANCE.get_or_init(|| Arc::new(Mutex::new(LoggerInner::new())));
        let mut logger_guard = match logger.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                println!("Logger mutex was poisoned, recovering...");
                poisoned.into_inner()
            }
        };
        
        match logger_guard.init_with_config(config) {
            Ok(_) => {
                println!("Logger initialized successfully");
                Ok(())
            },
            Err(e) => {
                println!("Failed to initialize logger: {}", e);
                Err(e)
            }
        }
    }

    /// Log a debug message
    pub fn debug(message: &str, context: Option<String>, file: &'static str, line: u32, module: &'static str) {
        Self::log_with_metadata(LogLevel::Debug, message, context, file, line, module)
    }

    /// Log an info message
    pub fn info(message: &str, context: Option<String>, file: &'static str, line: u32, module: &'static str) {
        Self::log_with_metadata(LogLevel::Info, message, context, file, line, module)
    }

    /// Log a warning message
    pub fn warn(message: &str, context: Option<String>, file: &'static str, line: u32, module: &'static str) {
        Self::log_with_metadata(LogLevel::Warn, message, context, file, line, module)
    }

    /// Log an error message
    pub fn error(message: &str, context: Option<String>, file: &'static str, line: u32, module: &'static str) {
        Self::log_with_metadata(LogLevel::Error, message, context, file, line, module)
    }

    fn log_with_metadata(level: LogLevel, message: &str, context: Option<String>, file: &str, line: u32, module: &str) {
        // Extract just the filename from the path
        let file_name = Path::new(file)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file);

        let logger = LOGGER_INSTANCE.get_or_init(|| Arc::new(Mutex::new(LoggerInner::new())));
        
        // Use a block to limit the scope of the mutex lock
        {
            if let Ok(mut logger) = logger.lock() {
                logger.log(level, message, context.as_deref(), file_name, line, module);
            } else {
                // If the mutex is poisoned, log to stderr
                let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
                let level_str = level.as_str();
                let log_line = format!("{} [{}] [{}:{}] [{}] {} | MUTEX POISONED\n",
                    timestamp, level_str, file_name, line, module, message);
                let _ = io::stderr().write_all(log_line.as_bytes());
            }
        }
    }

    /// Shutdown the logger gracefully, ensuring all pending logs are written
    pub fn shutdown() -> Result<(), String> {
        // Try to get the runtime
        if let Some(rt) = RUNTIME.get() {
            // Get a handle to the runtime for shutdown
            let handle = rt.handle().clone();
            handle.spawn(async {
                // Give some time for pending logs to be processed
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            });
            
            // Give it a moment to process remaining logs
            std::thread::sleep(std::time::Duration::from_secs(2));
            
            println!("Logger shutdown completed");
        }
        Ok(())
    }
}

// Ensure the logger is properly shutdown when the program exits
impl Drop for Logger {
    fn drop(&mut self) {
        let _ = Self::shutdown();
    }
}
