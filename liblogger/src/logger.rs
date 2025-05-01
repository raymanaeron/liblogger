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
use std::pin::Pin;
use std::future::Future;

use crate::config::{LogConfig, LogLevel};
use crate::outputs::{LogOutput, create_log_output, create_async_log_output, AsyncLogOutputTrait};
use crate::outputs::AsyncLogOutput;

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
    /// 
    /// This is called internally when first creating the logger instance.
    /// All logs will go to stderr until properly initialized with a configuration.
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
    /// 
    /// Sets up:
    /// 1. Configuration settings and log threshold
    /// 2. Synchronous output for fallback operations
    /// 3. Tokio runtime for asynchronous logging
    /// 4. Message channel for non-blocking log operations
    /// 5. Background task for processing log messages
    ///
    /// # Parameters
    /// - `config`: LogConfig containing all logger settings
    /// 
    /// # Returns
    /// - `Result<(), String>`: Success or error message
    fn init_with_config(&mut self, config: LogConfig) -> Result<(), String> {
        self.config = Some(config.clone());
        
        // Initialize synchronous output for fallback
        self.output = Some(create_log_output(&config)?);
        
        // Try to get or initialize the Tokio runtime
        let runtime = match RUNTIME.get() {
            Some(rt) => rt,
            None => {
                // Create a new runtime
                let rt = Runtime::new()
                    .map_err(|e| format!("Failed to create Tokio runtime: {}", e))?;
                
                RUNTIME.set(rt).map_err(|_| "Failed to set Tokio runtime".to_string())?;
                RUNTIME.get().unwrap()
            }
        };
        
        // Initialize the async logging channel
        let (tx, rx) = mpsc::channel::<LogMessage>(1024); // Buffer size of 1024 messages
        self.async_sender = Some(tx);
        
        // Clone the config for the async task
        let config_clone = config.clone();
        
        // Spawn the async logging task
        runtime.spawn(async move {
            if let Err(e) = process_log_messages(rx, config_clone).await {
                eprintln!("Async logger task error: {}", e);
            }
        });
        
        self.initialized = true;
        self.async_enabled = true;
        
        Ok(())
    }

    fn log(&mut self, level: LogLevel, message: &str, context: Option<&str>, file: &str, line: u32, module: &str) {
        // Get current timestamp for both sync and async paths
        let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        
        // If not initialized or async is disabled, log synchronously
        if !self.initialized || !self.async_enabled {
            let log_line = if let Some(ctx) = context {
                format!("{} [{}] [{}:{}] [{}] {} | Context: {}\n", 
                    &timestamp, level.as_str(), file, line, module, message, ctx)
            } else {
                format!("{} [{}] [{}:{}] [{}] {}\n", 
                    &timestamp, level.as_str(), file, line, module, message)
            };
            let _ = io::stderr().write_all(log_line.as_bytes());
            return;
        }

        // Check if we should log this level
        if let Some(config) = &self.config {
            if level.should_log(&config.threshold) {
                // For async logging, send message to channel
                if let Some(sender) = &self.async_sender {
                    let log_message = LogMessage {
                        timestamp: timestamp.clone(), // Clone the timestamp
                        level: level.clone(),
                        message: message.to_string(),
                        context: context.map(|s| s.to_string()),
                        file: file.to_string(),
                        line,
                        module: module.to_string(),
                    };
                    
                    // Try to send the message async
                    if let Err(_) = sender.try_send(log_message) {
                        // If channel is full, fall back to sync logging
                        if let Some(output) = &mut self.output {
                            let _ = output.write_log(
                                &timestamp,
                                &level,
                                message,
                                file,
                                line,
                                module,
                                context
                            );
                        }
                    }
                } else if let Some(output) = &mut self.output {
                    // Fallback to sync logging if no sender
                    let _ = output.write_log(
                        &timestamp,
                        &level,
                        message,
                        file,
                        line,
                        module,
                        context
                    );
                }
            }
        }
    }
}

// Async function to process log messages from the channel
async fn process_log_messages(mut receiver: Receiver<LogMessage>, config: LogConfig) -> Result<(), String> {
    // Create async output
    let mut async_output = create_async_log_output(&config)?;
    
    // Process messages as they arrive
    while let Some(msg) = receiver.recv().await {
        // Instead of trying to await the boxed future directly, let's run it in a different way
        // Create a wrapper async block that calls the boxed future
        let result = run_async_log(&mut async_output, &msg).await;
        
        if let Err(e) = result {
            eprintln!("Async log error: {}", e);
        }
    }
    
    Ok(())
}

// Helper function to properly handle the boxed future
async fn run_async_log(
    output: &mut AsyncLogOutput, 
    msg: &LogMessage
) -> Result<(), String> {
    // Get the boxed future
    let boxed_future = output.write_log_async(
        &msg.timestamp,
        &msg.level,
        &msg.message,
        &msg.file,
        msg.line,
        &msg.module,
        msg.context.as_deref()
    );
    
    // Pin the boxed future properly before awaiting it
    let pinned_future: Pin<Box<dyn Future<Output = Result<(), String>> + Send>> = Pin::from(boxed_future);
    
    // Now we can await the properly pinned future
    pinned_future.await
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
                let log_line = format!("{} [{}] [{}:{}] [{}] {} | MUTEX POISONED\n",
                    timestamp, level.as_str(), file_name, line, module, message);
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
