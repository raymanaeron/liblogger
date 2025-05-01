/*
 * Configuration management for the Rusty Logger v2
 * 
 * This module handles:
 * - Parsing configuration from TOML files (app_config.toml)
 * - Defining the LogType enum for output destinations (Console, File, Http)
 * - Defining the LogLevel enum for severity levels (Debug, Info, Warn, Error)
 * - Implementing methods for level comparison and string conversion
 * - Providing default configuration values for all settings
 * 
 * The configuration determines:
 * - Where logs are written (console, file with rotation, or HTTP endpoint)
 * - Which severity levels are included in the output based on threshold
 * - File paths, rotation sizes, and HTTP timeouts
 * - Behavior of both synchronous and asynchronous logging operations
 */

use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Defines the available output destinations for logs
/// 
/// - `Console`: Logs to standard output
/// - `File`: Logs to a file with rotation functionality
/// - `Http`: Sends logs to a remote HTTP endpoint
#[derive(Debug, Clone, Deserialize)]
pub enum LogType {
    #[serde(rename = "console")]
    Console,
    #[serde(rename = "file")]
    File,
    #[serde(rename = "http")]
    Http,
}

/// Defines the severity levels for log messages
/// 
/// - `Debug`: Detailed information for debugging purposes
/// - `Info`: General information about application operation
/// - `Warn`: Warning conditions that deserve attention
/// - `Error`: Error conditions that require intervention
#[derive(Debug, Clone, Deserialize)]
pub enum LogLevel {
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    Error,
}

impl LogLevel {
    /// Converts the log level to a string representation
    /// 
    /// Returns capitalized string representation (e.g., "DEBUG", "INFO")
    /// suitable for inclusion in log messages
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    /// Creates a LogLevel from a string representation
    /// 
    /// Case-insensitive matching of log level names
    /// Defaults to Info level if the string doesn't match any known level
    pub fn from_str(s: &str) -> LogLevel {
        match s.to_lowercase().as_str() {
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info, // Default to info for unknown levels
        }
    }

    /// Determines if a log message with this level should be recorded
    /// based on the configured threshold
    /// 
    /// - If threshold is Debug, all messages are logged
    /// - If threshold is Info, all except Debug are logged
    /// - If threshold is Warn, only Warn and Error are logged
    /// - If threshold is Error, only Error messages are logged
    pub fn should_log(&self, threshold: &LogLevel) -> bool {
        match threshold {
            // If threshold is Debug, log everything
            LogLevel::Debug => true,
            
            // If threshold is Info, log Info, Warn, Error but not Debug
            LogLevel::Info => match self {
                LogLevel::Debug => false,
                _ => true,
            },
            
            // If threshold is Warn, log only Warn and Error
            LogLevel::Warn => match self {
                LogLevel::Debug | LogLevel::Info => false,
                _ => true,
            },
            
            // If threshold is Error, log only Error
            LogLevel::Error => match self {
                LogLevel::Error => true,
                _ => false,
            },
        }
    }
}

/// Main configuration structure for the Rusty Logger v2
/// 
/// Contains all settings for the logger, including output destination,
/// thresholds, file paths, and HTTP endpoints for remote logging.
#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    /// The destination for log output (Console, File, or Http)
    #[serde(rename = "type")]
    pub log_type: LogType,
    
    /// Minimum severity level that will be logged
    pub threshold: LogLevel,
    
    /// Name of the log file when using File output type
    /// Defaults to "app.log" if not specified
    #[serde(default = "default_file_path")]
    pub file_path: String,
    
    /// Directory where log files will be stored
    /// Defaults to "logs" if not specified
    #[serde(default = "default_log_folder")]
    pub log_folder: String,
    
    /// Maximum size of log files in megabytes before rotation
    /// Defaults to 10 MB if not specified
    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: u64,
    
    /// URL endpoint for HTTP logging
    /// Defaults to "http://localhost:8080/logs" if not specified
    #[serde(default = "default_http_endpoint")]
    pub http_endpoint: String,
    
    /// Timeout in seconds for HTTP requests
    /// Defaults to 5 seconds if not specified
    #[serde(default = "default_http_timeout")]
    pub http_timeout_seconds: u64,
}

// Default value functions for LogConfig properties

/// Default log file name
fn default_file_path() -> String {
    "app.log".into()
}

/// Default folder for log files
fn default_log_folder() -> String {
    "logs".into()
}

/// Default maximum log file size before rotation (in MB)
fn default_max_file_size() -> u64 {
    10 // 10 MB by default
}

/// Default HTTP endpoint for remote logging
fn default_http_endpoint() -> String {
    "http://localhost:8080/logs".into()
}

/// Default HTTP timeout in seconds
fn default_http_timeout() -> u64 {
    5 // 5 seconds by default
}

impl LogConfig {
    /// Loads logger configuration from a TOML file
    /// 
    /// # Parameters
    /// - `path`: Path to the configuration file
    /// 
    /// # Returns
    /// - `Result<LogConfig, String>`: Configuration object or error message
    /// 
    /// # Format
    /// The TOML file should contain a [logging] section with configuration parameters
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let config: toml::Table = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse TOML: {}", e))?;
        
        let logging_section = config.get("logging")
            .ok_or_else(|| "Missing [logging] section in config".to_string())?;
        
        let log_config: LogConfig = toml::from_str(&toml::to_string(logging_section).unwrap())
            .map_err(|e| format!("Failed to parse logging config: {}", e))?;
        
        Ok(log_config)
    }

    /// Creates the log directory if it doesn't exist
    /// 
    /// # Returns
    /// - `Result<(), String>`: Success or error message
    pub fn ensure_log_folder_exists(&self) -> Result<(), String> {
        // Always create the log folder, regardless of log type
        let path = Path::new(&self.log_folder);
        if !path.exists() {
            println!("[Config] Creating log directory: {:?}", path);
            fs::create_dir_all(path)
                .map_err(|e| format!("Failed to create log directory: {}", e))?;
        }
        Ok(())
    }

    /// Creates a LogConfig with default values
    /// 
    /// Default values:
    /// - log_type: Console
    /// - threshold: Info
    /// - file_path: "app.log"
    /// - log_folder: "logs"
    /// - max_file_size_mb: 10
    /// - http_endpoint: "http://localhost:8080/logs"
    /// - http_timeout_seconds: 5
    pub fn default() -> Self {
        LogConfig {
            log_type: LogType::Console,
            threshold: LogLevel::Info,
            file_path: "app.log".to_string(),
            log_folder: "logs".to_string(),
            max_file_size_mb: 10,
            http_endpoint: "http://localhost:8080/logs".to_string(),
            http_timeout_seconds: 5,
        }
    }
}
