# A Logging Framework For Rust

 ## 1. Problem Statement
 
 Modern distributed systems require effective logging for observability and debugging. Organizations face specific challenges when logging is implemented manually or inconsistently across services.
 
 Developers often omit critical logs during development, leading to blind spots during incident response. When logs are present, they frequently lack essential context such as precise location, timing, and error cause. The formats vary between services, complicating automated parsing and analysis.
 
 Without consistent entry and exit tracking in functions, tracing request flows becomes difficult during high-severity incidents. Many logs also miss crucial correlation identifiers like user IDs, request IDs, and session IDs that connect related events across distributed components.
 
 These logging gaps directly impact operational metrics. Incident triage takes longer when investigators must reconstruct what happened from incomplete data. Root cause analysis becomes a process of educated guesswork rather than evidence-based investigation. System downtime extends while teams gather missing information, resulting in increased Mean Time to Recovery (MTTR) metrics.
 
 The Rust Logger Framework addresses these operational challenges by providing structured, consistent logging capabilities that capture required context automatically.
 
 ---
 
 ## 2. Architecture Overview
 
 The Rust Logger Framework provides logging capabilities with multiple output options. The system supports three output targets: Console for standard output, File with rotation functionality, and HTTP endpoints for remote logging aggregation.
 
 Configuration occurs through an `app_config.toml` file, where users specify output type, log level thresholds, and target-specific parameters. The framework implements a singleton pattern, requiring a single `Logger::init()` call during program initialization, after which any component can use static methods like `Logger::info()` to record events.
 
 Each log entry follows a consistent format that includes timestamp, severity level, file name, line number, module path, function name, and additional context. The framework employs procedural macros to capture this metadata automatically without requiring developers to specify it manually.
 
 This architecture ensures that logs contain necessary debugging information regardless of developer attention to detail. The consistent format enables reliable parsing for analytics and monitoring tools. The comprehensive context collection reduces the time needed to understand system behavior during incidents.
 
 ---
 
 ## 3. Installation
 
 To use the Rust Logger Framework, add the following to your `Cargo.toml`:
 
 ```toml
 [dependencies]
 logger = "0.1"
 once_cell = "1.17"
 serde = { version = "1.0", features = ["derive"] }
 reqwest = "0.11"
 tokio = { version = "1", features = ["full"] }
 ```
 
 ---
 
 ## 4. Configuration
 
 The logger is configured via `app_config.toml`. Below is an example configuration:
 
 ```toml
 [logging]
 type = "file" # console | file | http
 threshold = "debug" # debug | info | warn | error
 file_path = "workflow.log"
 log_folder = "logs"
 max_file_size_mb = 1
 http_endpoint = "https://logs.example.com"
 http_timeout_seconds = 5
 ```
 
 ### Key Points:
 - **File Rotation**: Automatically rotates files when they exceed 1MB.
 - **Console Output**: Logs to `stdout`.
 - **HTTP Output**: Sends logs as JSON to the specified endpoint.
 
 ---
 
 ## 5. Usaage and Initialization

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

---
 
 ## 6. Output Target Examples
 
 ### Console Output
 ```toml
 type = "console"
 ```
 
 ### File Output
 ```toml
 type = "file"
 file_path = "workflow.log"
 log_folder = "logs"
 max_file_size_mb = 1
 ```
 
 ### HTTP Output
 ```toml
 type = "http"
 http_endpoint = "https://your.logging.service/endpoint"
 ```
 
 ---
 
 ## 7. Built-in Procedural Macros
 
 The Rust Logger Framework includes numerous procedural macros that simplify logging and ensure critical metadata is captured automatically. These macros can be applied to functions to enhance observability with minimal code changes.
 
 ### `#[log_entry_exit]`
 - **Description**: Logs function entry and exit points.
 - **Example**:
   ```rust
   #[log_entry_exit]
   fn process_data(user_id: &str) {
       // Function implementation
   }
   ```
 - **Captures**: Function name at entry and exit points.
 
 ### `#[log_errors]`
 - **Description**: Logs errors returned by a function and captures panics.
 - **Example**:
   ```rust
   #[log_errors]
   fn database_operation() -> Result<Data, Error> {
       // Function that might fail
   }
   ```
 - **Captures**: Error details or panic information with context.
 
 ### `#[measure_time]`
 - **Description**: Measures and logs the execution time of a function.
 - **Example**:
   ```rust
   #[measure_time]
   fn expensive_calculation() -> u64 {
       // Time-consuming operation
   }
   ```
 - **Captures**: Function name and execution duration in milliseconds.
 
 ### `#[log_args]`
 - **Description**: Logs specified function arguments for debugging and tracing.
 - **Example**:
   ```rust
   #[log_args(user_id, action_type)]
   fn audit_user_action(user_id: &str, action_type: ActionType, metadata: &str) {
       // Only user_id and action_type will be logged
   }
   ```
 - **Captures**: Values of specified function arguments.
 
 ### `#[log_retries]`
 - **Description**: Automatically implements retry logic and logs retry attempts.
 - **Example**:
   ```rust
   #[log_retries(max_attempts = 5)]
   fn unreliable_network_call() -> Result<Response, Error> {
       // Will be retried up to 5 times with exponential backoff
   }
   ```
 - **Captures**: Retry attempt number, success/failure information.
 
 ### `#[audit_log]`
 - **Description**: Creates audit logs for security-critical operations.
 - **Example**:
   ```rust
   #[audit_log(category = "authentication")]
   fn change_user_permissions(user_id: &str, new_role: Role) -> Result<(), Error> {
       // Security-sensitive operation
   }
   ```
 - **Captures**: Operation category, function name, and result type.
 
 ### `#[circuit_breaker]`
 - **Description**: Implements the circuit breaker pattern to prevent cascading failures.
 - **Example**:
   ```rust
   #[circuit_breaker(failure_threshold = 5)]
   fn call_external_service() -> Result<Response, Error> {
       // Call to external dependency
   }
   ```
 - **Captures**: Failure counts, circuit state changes.
 
 ### `#[throttle_log]`
 - **Description**: Limits log frequency to prevent log flooding during incidents.
 - **Example**:
   ```rust
   #[throttle_log(rate = 10)]
   fn high_volume_operation() -> Result<(), Error> {
       // Operation that might generate many logs
   }
   ```
 - **Captures**: Function success/failure while limiting to specified rate per minute.
 
 ### `#[dependency_latency]`
 - **Description**: Measures and logs latency to external dependencies.
 - **Example**:
   ```rust
   #[dependency_latency(target = "payment_gateway")]
   fn process_payment(payment: Payment) -> Result<Receipt, PaymentError> {
       // External API call
   }
   ```
 - **Captures**: Target name, call duration, and result status.
 
 ### `#[log_response]`
 - **Description**: Logs the return value of a function.
 - **Example**:
   ```rust
   #[log_response]
   fn get_user_preferences(user_id: &str) -> UserPreferences {
       // Return value will be logged at debug level
   }
   ```
 - **Captures**: Function name and returned value.
 
 ### `#[log_concurrency]`
 - **Description**: Tracks concurrent invocations of a function.
 - **Example**:
   ```rust
   #[log_concurrency]
   fn handle_request(request: Request) -> Response {
       // Concurrent handling of requests
   }
   ```
 - **Captures**: Counter of concurrent invocations before and after execution.
 
 ### `#[trace_span]`
 - **Description**: Creates and propagates a trace ID for request flow tracking.
 - **Example**:
   ```rust
   #[trace_span]
   fn process_order(order: Order) -> Result<OrderConfirmation, OrderError> {
       // Will generate or reuse a trace ID
   }
   ```
 - **Captures**: Trace ID, function entry and exit.
 
 ### `#[feature_flag]`
 - **Description**: Logs feature flag state when function is called.
 - **Example**:
   ```rust
   #[feature_flag(flag_name = "new_pricing_algorithm")]
   fn calculate_price(product: &Product) -> Price {
       // Function that depends on a feature flag
   }
   ```
 - **Captures**: Feature flag name and its enabled/disabled state.
 
 ### `#[metrics_counter]`
 - **Description**: Increments a metrics counter for function calls (supports Prometheus).
 - **Example**:
   ```rust
   #[metrics_counter(counter_name = "api_requests_total")]
   fn api_endpoint(params: Params) -> ApiResponse {
       // Function whose calls should be counted
   }
   ```
 - **Captures**: Increments specified counter on each call.
 
 ### `#[log_memory_usage]`
 - **Description**: Logs memory usage before and after function execution.
 - **Example**:
   ```rust
   #[log_memory_usage]
   fn memory_intensive_operation(data: &[u8]) -> ProcessedData {
       // Memory-heavy processing
   }
   ```
 - **Captures**: RSS and VMS memory usage before and after execution, with deltas.
 
 ### `#[log_cpu_time]`
 - **Description**: Logs CPU time used during function execution.
 - **Example**:
   ```rust
   #[log_cpu_time]
   fn cpu_intensive_task() -> Result<(), Error> {
       // CPU-heavy computation
   }
   ```
 - **Captures**: Approximate CPU time used (wall time).
 
 ### `#[version_tag]`
 - **Description**: Includes application version information in logs.
 - **Example**:
   ```rust
   #[version_tag]
   fn startup_procedure() {
       // Application initialization
   }
   ```
 - **Captures**: Build version from environment variables.
 
 ### `#[request_context]`
 - **Description**: Attaches request context from thread-local storage to logs.
 - **Example**:
   ```rust
   #[request_context]
   fn handle_api_request(request_data: &str) -> Response {
       // API request processing
   }
   ```
 - **Captures**: User ID, session ID, request ID if available in thread-local storage.
 
 ### `#[catch_panic]`
 - **Description**: Catches and logs panics without allowing the application to crash.
 - **Example**:
   ```rust
   #[catch_panic]
   fn potentially_unstable_code() -> Result<Output, Error> {
       // Code that might panic
   }
   ```
 - **Captures**: Panic message and converts it to a proper error.
 
 ### `#[health_check]`
 - **Description**: Logs health check results and execution time.
 - **Example**:
   ```rust
   #[health_check]
   fn check_database_connection() -> Result<(), Error> {
       // Database connectivity check
   }
   ```
 - **Captures**: Check duration and pass/fail status with error details.
 
 ### `#[log_result]`
 - **Description**: Logs function results with configurable log levels for success and error cases.
 - **Example**:
   ```rust
   #[log_result(success_level = "info", error_level = "error")]
   fn critical_operation() -> Result<Success, Failure> {
       // Operation whose result should always be logged
   }
   ```
 - **Captures**: Function name, result or error details at specified log levels.
 
 ---
 
 ## 8. Example Combining Macros
 
 You can stack multiple macros for deep observability:
 
 ```rust
 #[log_entry_exit]
 #[log_error]
 fn perform_task(task_id: u32) -> Result<(), String> {
     // Function logic
     Ok(())
 }
 ```
 
 This ensures that both entry/exit and errors are logged automatically.
 
 ---
 
 ## 9. Final Notes
 
 The Rust Logger Framework implements a zero-trust approach to observability, capturing critical metadata regardless of developer diligence. By enforcing consistent log structure and format across all application components, the framework enables reliable parsing and analysis during incident investigation. 
 
 Organizations using this framework typically experience accelerated time-to-resolution during severity incidents due to the comprehensive context available in logs. The architecture supports extension through additional output targets or custom macros when specific requirements arise.
 
 Teams can integrate this framework into both new and existing Rust applications to establish reliable, consistent logging practices across their services. This consistency becomes particularly valuable in microservice architectures where request flows span multiple components and where operational excellence depends on complete observability.