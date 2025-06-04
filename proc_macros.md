# Procedural Macros Documentation

This document provides comprehensive documentation for all procedural macros available in the `liblogger_macros` crate. These macros provide automatic logging, monitoring, and instrumentation capabilities for Rust functions.

## Quick Start

### 1. Add Dependencies

Add these dependencies to your `Cargo.toml`:

```toml
[dependencies]
liblogger = { path = "../path/to/liblogger" }
liblogger_macros = { path = "../path/to/liblogger_macros" }
uuid = "1.0"  # Required for trace_span macro
prometheus = "0.13"  # Required for metrics_counter macro
psutil = "3.2"  # Required for log_memory_usage macro
```

### 2. Initialize in Your Code

At the top of your source file, add:

```rust
use liblogger_macros::*;
use liblogger::{Logger, log_info, log_debug, log_warn, log_error};

// Initialize the logger (call once at application startup)
Logger::init();

// Initialize macro support (must be at module level)
initialize_logger_attributes!();
```

### 3. Apply Macros to Functions

```rust
#[log_entry_exit]
#[measure_time]
fn my_function() {
    // Your code here
}
```

---

## All Available Macros

### 1. `initialize_logger_attributes!()`

**Type**: Function-like macro (required)  
**Purpose**: Defines helper functions needed by attribute macros  
**Usage**: Must be called at module level before using any attribute macros

```rust
use liblogger_macros::*;

initialize_logger_attributes!();
```

---

## Attribute Macros

### 2. `log_entry_exit`

**Purpose**: Logs function entry and exit points  
**Parameters**: None  
**Async Support**: ✅ Full support

```rust
#[log_entry_exit]
fn process_data(user_id: &str) {
    // Function implementation
}

#[log_entry_exit]
async fn async_process_data(user_id: &str) {
    // Async function implementation
}
```

**Generated Logs**:
```
ENTRY: process_data
EXIT: process_data
```

---

### 3. `log_errors`

**Purpose**: Automatically logs errors and panics from functions  
**Parameters**: None  
**Async Support**: ✅ Full support  
**Works With**: Any `Result<T, E>` return type

```rust
#[log_errors]
fn validate_input(data: &str) -> Result<(), ValidationError> {
    if data.is_empty() {
        return Err(ValidationError::Empty);
    }
    Ok(())
}

#[log_errors]
async fn async_validate_input(data: &str) -> Result<(), ValidationError> {
    // Async validation logic
    Ok(())
}
```

**Generated Logs** (on error):
```
validate_input returned error: Empty
```

---

### 4. `measure_time`

**Purpose**: Measures and logs function execution time  
**Parameters**: None  
**Async Support**: ✅ Full support

```rust
#[measure_time]
fn generate_report() -> Report {
    // Time-consuming operation
    Report::new()
}

#[measure_time]
async fn async_generate_report() -> Report {
    // Async time-consuming operation
    Report::new()
}
```

**Generated Logs**:
```
generate_report completed in 1250 ms
```

---

### 5. `log_args`

**Purpose**: Logs specified function arguments  
**Parameters**: List of argument names to log  
**Async Support**: ⚠️ Partial (no async detection, but works)

```rust
#[log_args(user_id, action)]
fn audit_user_action(user_id: &str, action: &str, details: &ActionDetails) {
    // Only user_id and action will be logged
}
```

**Generated Logs**:
```
Entering audit_user_action with args: user_id = "12345", action = "delete_account"
```

---

### 6. `log_retries`

**Purpose**: Implements retry logic with comprehensive logging  
**Parameters**: `max_attempts=N` (default: 3)  
**Async Support**: ✅ Full support (skips sleep delays)  
**Works With**: Any `Result<T, E>` return type

```rust
#[log_retries(max_attempts=3)]
fn connect_to_database() -> Result<Connection, DbError> {
    // Function will be retried up to 3 times if it fails
    database::connect()
}

#[log_retries(max_attempts=5)]
async fn async_connect_to_api() -> Result<ApiResponse, ApiError> {
    // Async retry with no sleep delays (implement your own if needed)
    api_client::fetch_data().await
}
```

**Generated Logs**:
```
Retry attempt 2 of 3 for connect_to_database
connect_to_database attempt 1 failed: connection refused
connect_to_database succeeded after 2 attempts
```

---

### 7. `audit_log`

**Purpose**: Creates detailed audit logs for security-sensitive operations  
**Parameters**: None  
**Async Support**: ✅ Full support

```rust
#[audit_log]
fn change_permissions(user_id: &str, new_role: Role) {
    // Security-sensitive operation
}

#[audit_log]
async fn async_change_permissions(user_id: &str, new_role: Role) {
    // Async security-sensitive operation
}
```

**Generated Logs**:
```
AUDIT: change_permissions called
AUDIT: change_permissions completed in 45 ms
```

---

### 8. `circuit_breaker`

**Purpose**: Implements circuit breaker pattern with failure tracking  
**Parameters**: `failure_threshold=N` (default: 3)  
**Async Support**: ✅ Full support  
**Works With**: Any `Result<T, E>` return type

```rust
#[circuit_breaker(failure_threshold=5)]
fn call_external_service() -> Result<Response, ServiceError> {
    // Will stop calling after 5 consecutive failures
    external_service::call()
}

#[circuit_breaker(failure_threshold=3)]
async fn async_call_external_service() -> Result<Response, ServiceError> {
    // Async circuit breaker
    external_service::call().await
}
```

**Generated Logs**:
```
Circuit breaker: call_external_service failed (3/5 failures)
Circuit breaker open for call_external_service: 5 failures exceeded threshold 5
```

---

### 9. `throttle_log`

**Purpose**: Throttles logs to prevent flooding during incidents  
**Parameters**: `rate=N` (default: 5 logs per minute)  
**Async Support**: ✅ Works with any function

```rust
#[throttle_log(rate=10)]
fn process_high_volume_events(event: &Event) {
    // Will only log 10 times per minute
    process_event(event);
}
```

**Generated Logs**:
```
process_high_volume_events executed
Throttled logs for process_high_volume_events: skipped 45 logs in previous minute
```

---

### 10. `dependency_latency`

**Purpose**: Measures latency to external dependencies  
**Parameters**: `target="service_name"` or first string argument  
**Async Support**: ⚠️ Partial (no async detection, but works)

```rust
#[dependency_latency(target="database")]
fn fetch_user_data(user_id: &str) -> Result<User, DbError> {
    database::get_user(user_id)
}

// Alternative syntax
#[dependency_latency("payment_service")]
fn process_payment(amount: f64) -> Result<Receipt, PaymentError> {
    payment_service::charge(amount)
}
```

**Generated Logs**:
```
Dependency call to database started for fetch_user_data
Dependency call to database completed in 125 ms
```

---

### 11. `log_response`

**Purpose**: Logs the returned value from functions  
**Parameters**: None  
**Async Support**: ⚠️ Partial (no async detection, but works)

```rust
#[log_response]
fn calculate_total(items: &[Item]) -> f64 {
    items.iter().map(|i| i.price).sum()
}
```

**Generated Logs**:
```
calculate_total returned: 45.67
```

---

### 12. `log_concurrency`

**Purpose**: Tracks concurrent invocations of a function  
**Parameters**: None  
**Async Support**: ⚠️ Partial (no async detection, but works)

```rust
#[log_concurrency]
fn handle_request(request: &Request) -> Response {
    // Tracks how many times this function is running simultaneously
    process_request(request)
}
```

**Generated Logs**:
```
handle_request concurrent invocations: 3
handle_request concurrent invocations after exit: 2
```

---

### 13. `trace_span`

**Purpose**: Creates and propagates trace IDs for request flow tracking  
**Parameters**: None  
**Async Support**: ⚠️ Partial (no async detection, but works)  
**Requires**: `uuid` crate dependency

```rust
#[trace_span]
fn handle_api_request(request: &Request) -> Response {
    // Generates a trace ID that can be used by nested functions
    process_request_data(request)
}

#[trace_span]
fn process_request_data(request: &Request) {
    // Uses the same trace ID as the parent function
}
```

**Generated Logs**:
```
[TraceID: 748405dd-ce44-48bd-9f1a-86fdb5eae237] handle_api_request started
[TraceID: 748405dd-ce44-48bd-9f1a-86fdb5eae237] process_request_data started
[TraceID: 748405dd-ce44-48bd-9f1a-86fdb5eae237] handle_api_request completed
```

---

### 14. `feature_flag`

**Purpose**: Logs feature flag state  
**Parameters**: `flag_name="feature_name"` or first string argument  
**Async Support**: ⚠️ Partial (no async detection, but works)

```rust
#[feature_flag(flag_name="new_ui")]
fn render_dashboard() -> Html {
    // Logs whether the feature flag is enabled
    render_ui()
}

// Alternative syntax
#[feature_flag("experimental_feature")]
fn experimental_algorithm() -> Result<Output, Error> {
    new_algorithm()
}
```

**Generated Logs**:
```
render_dashboard called with feature flag new_ui = true
```

---

### 15. `metrics_counter`

**Purpose**: Increments metrics counters for function calls  
**Parameters**: `counter_name="counter_name"` (default: "function_calls")  
**Async Support**: ⚠️ Partial (no async detection, but works)  
**Requires**: `prometheus` crate

```rust
#[metrics_counter(counter_name="api_calls")]
fn handle_api_call() {
    // Increments Prometheus counter
}
```

**Note**: Always available - no feature flags required.

---

### 16. `log_memory_usage`

**Purpose**: Logs memory usage during function execution  
**Parameters**: None  
**Async Support**: ⚠️ Partial (no async detection, but works)  
**Requires**: `psutil` crate

```rust
#[log_memory_usage]
fn memory_intensive_operation(data: &[u8]) -> ProcessedData {
    // Logs memory usage before and after execution
    process_large_dataset(data)
}
```

**Generated Logs**:
```
memory_intensive_operation starting memory usage - RSS: 1048576 bytes, VMS: 2097152 bytes
memory_intensive_operation ending memory usage - RSS: 2097152 bytes (delta: 1048576 bytes), VMS: 3145728 bytes (delta: 1048576 bytes)
```

---

### 17. `log_cpu_time`

**Purpose**: Logs CPU time used during function execution  
**Parameters**: None  
**Async Support**: ⚠️ Partial (no async detection, but works)  
**Note**: Currently measures wall time as CPU time is not directly available

```rust
#[log_cpu_time]
fn cpu_intensive_task() {
    // Logs approximate CPU time (actually wall time)
    perform_calculations()
}
```

**Generated Logs**:
```
cpu_intensive_task used CPU time: approx 1250 ms (wall time)
```

---

### 18. `version_tag`

**Purpose**: Includes version information in logs  
**Parameters**: None  
**Async Support**: ⚠️ Partial (no async detection, but works)  
**Uses**: `BUILD_VERSION` environment variable

```rust
#[version_tag]
fn initialize_service() {
    // Logs the build version with the function call
    setup_service()
}
```

**Generated Logs**:
```
[Version: 1.2.3] initialize_service called
```

---

### 19. `request_context`

**Purpose**: Attaches request context to logs  
**Parameters**: None  
**Async Support**: ⚠️ Partial (no async detection, but works)  
**Uses**: Thread-local storage for context (placeholder implementation)

```rust
#[request_context]
fn process_user_request(data: &RequestData) -> Response {
    // Logs context like user_id, session_id, request_id
    handle_request(data)
}
```

**Generated Logs**:
```
process_user_request called | Context: user_id=12345, session_id=abcd-1234-xyz, request_id=req-789
```

---

### 20. `catch_panic`

**Purpose**: Catches and logs panics without crashing  
**Parameters**: None  
**Async Support**: ✅ Full support (limited for async - catches at Result level)  
**Works With**: Functions that return `Result<T, E>` (recommended)

```rust
#[catch_panic]
fn risky_operation() -> Result<Output, Error> {
    // If this panics, it will be caught and logged
    potentially_panicking_code()
}

#[catch_panic]
async fn async_risky_operation() -> Result<Output, Error> {
    // Async version - catches errors at Result level
    potentially_failing_async_code().await
}
```

**Generated Logs** (on panic):
```
risky_operation caught panic: index out of bounds
```

---

### 21. `health_check`

**Purpose**: Logs health check results with timing  
**Parameters**: None  
**Async Support**: ⚠️ Partial (no async detection, but works)  
**Works With**: Functions returning `Result<T, E>`

```rust
#[health_check]
fn database_health_check() -> Result<(), HealthError> {
    // Logs success/failure with timing
    database::ping()
}
```

**Generated Logs**:
```
Health check database_health_check passed in 25 ms
Health check database_health_check failed in 5000 ms: connection timeout
```

---

### 22. `log_result`

**Purpose**: Logs function results with custom log levels for success/error  
**Parameters**: `success_level="level"`, `error_level="level"` (default: "info", "error")  
**Async Support**: ⚠️ Partial (no async detection, but works)  
**Works With**: Functions returning `Result<T, E>`

```rust
#[log_result(success_level="debug", error_level="warn")]
fn batch_process() -> Result<BatchStats, ProcessError> {
    // Logs successes at DEBUG level, failures at WARN level
    process_batch_data()
}

// Using default levels (info for success, error for failure)
#[log_result]
fn critical_operation() -> Result<(), CriticalError> {
    perform_operation()
}
```

**Generated Logs**:
```
batch_process succeeded with result: BatchStats { processed: 100, failed: 2 }
batch_process failed with error: ProcessError::InvalidData
```

---

## Async Support Summary

| Macro | Async Support | Notes |
|-------|---------------|-------|
| `log_entry_exit` | ✅ Full | Properly detects and handles async functions |
| `log_errors` | ✅ Full | Handles async errors correctly |
| `measure_time` | ✅ Full | Accurate timing for async functions |
| `log_args` | ⚠️ Partial | Works but doesn't detect async |
| `log_retries` | ✅ Full | Skips sleep delays for async functions |
| `audit_log` | ✅ Full | Properly handles async execution |
| `circuit_breaker` | ✅ Full | Thread-safe failure tracking |
| `throttle_log` | ✅ Works | Rate limiting works for any function |
| `dependency_latency` | ⚠️ Partial | Works but doesn't detect async |
| `log_response` | ⚠️ Partial | Works but doesn't detect async |
| `log_concurrency` | ⚠️ Partial | Works but doesn't detect async |
| `trace_span` | ⚠️ Partial | Works but doesn't detect async |
| `feature_flag` | ⚠️ Partial | Works but doesn't detect async |
| `metrics_counter` | ⚠️ Partial | Works but doesn't detect async |
| `log_memory_usage` | ⚠️ Partial | Works but doesn't detect async |
| `log_cpu_time` | ⚠️ Partial | Works but doesn't detect async |
| `version_tag` | ⚠️ Partial | Works but doesn't detect async |
| `request_context` | ⚠️ Partial | Works but doesn't detect async |
| `catch_panic` | ✅ Full | Limited async support (Result-level) |
| `health_check` | ⚠️ Partial | Works but doesn't detect async |
| `log_result` | ⚠️ Partial | Works but doesn't detect async |

---

## Combining Multiple Macros

You can stack multiple macros on the same function:

```rust
#[log_entry_exit]
#[measure_time]
#[log_errors]
#[audit_log]
async fn critical_async_operation(user_id: &str) -> Result<Output, Error> {
    // This function will:
    // 1. Log entry and exit
    // 2. Measure execution time
    // 3. Log any errors that occur
    // 4. Create audit trail
    perform_critical_work(user_id).await
}
```

---

## Error Handling and Type Safety

All macros use pattern matching instead of reflection, making them:
- **Type-safe**: Work with any `Result<T, E>` type
- **Performance-friendly**: No runtime type inspection
- **Reliable**: Won't break with custom error types

```rust
// Works with any error type
#[log_errors]
fn custom_error_function() -> Result<(), MyCustomError> {
    Err(MyCustomError::SomeVariant)
}

#[log_retries(max_attempts=3)]
fn another_error_type() -> Result<String, AnotherError> {
    Err(AnotherError::NetworkTimeout)
}
```

---

## Performance Considerations

- **Async macros** have minimal overhead and don't block async runtime
- **Retry logic** skips sleep delays in async functions (implement your own if needed)
- **Memory/CPU tracking** macros require optional dependencies
- **Throttling** uses atomic counters for thread-safe rate limiting
- **Pattern matching** is used instead of reflection for better performance

---

## Configuration Dependencies

Some macros require additional crates to be added to your dependencies:

### For `trace_span`:
```toml
[dependencies]
uuid = "1.0"
```

### For `metrics_counter`:
```toml
[dependencies]
prometheus = "0.13"
```

### For `log_memory_usage`:
```toml
[dependencies]
psutil = "3.2"
```

**Note**: All features are now enabled by default. No feature flags are required.

---

## Best Practices

1. **Initialize once**: Call `initialize_logger_attributes!()` at module level
2. **Combine wisely**: Stack complementary macros (e.g., `log_entry_exit` + `measure_time`)
3. **Use appropriate levels**: Configure `log_result` with suitable success/error levels
4. **Async awareness**: Prefer macros with full async support for async functions
5. **Error handling**: Use macros with `Result<T, E>` return types for best results
6. **Dependencies**: Add required crates (`uuid`, `prometheus`, `psutil`) as needed

---

## Troubleshooting

### Common Issues:

1. **"Helper functions not found"**: Ensure `initialize_logger_attributes!()` is called
2. **Compile errors with async**: Use macros with full async support
3. **Missing dependencies**: Add required crates (`uuid`, `prometheus`, `psutil`) for specific macros
4. **Pattern matching errors**: Ensure functions return appropriate types

### Debug Tips:

1. Check that `liblogger::Logger::init()` is called before using macros
2. Verify macro order when stacking (some combinations work better than others)
3. Use `cargo expand` to see generated code if debugging macro behavior
4. Ensure all required dependencies are added to your `Cargo.toml`

---

This documentation covers all 22 procedural macros available in the `liblogger_macros` crate. Each macro is designed to work seamlessly with the liblogger framework while providing specialized logging and monitoring capabilities for different use cases.
