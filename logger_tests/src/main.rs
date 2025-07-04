/*
 * Test application for Rusty Logger v2
 * 
 * This comprehensive test suite demonstrates and validates:
 * - Basic synchronous logging at all severity levels (debug, info, warn, error)
 * - Asynchronous logging with high message volume
 * - Over 20 procedural macros for automatic context capturing
 * - Performance monitoring and instrumentation capabilities
 * - Advanced patterns like circuit breaker, retry logic, and log throttling
 * 
 * All tests use the same app_config.toml for configuration to ensure
 * consistent testing across synchronous and asynchronous logging paths.
 */

use liblogger::{Logger, log_info, log_warn, log_error, log_debug};
use liblogger_macros::*;
use rand::Rng;

// Initialize helper functions required by the procedural macros
initialize_logger_attributes!();

// Import the asynchronous logging test module
mod async_test;
use async_test::test_async_logger;

// Import the DevOps macro tests module  
mod devops_macro_tests;
use devops_macro_tests::run_all_devops_tests;

fn main() {
    // Initialize the logger from default config file
    // Replace the default initialization with a custom one
    initialize_custom_logger();
    
    log_info!("Application started with enhanced logging macros");
    
    // Test various logging macros
    test_log_entry_exit();
    
    if let Err(err) = test_log_errors() {
        log_error!(&format!("Error test function returned: {:?}", err));
    }
    
    test_measure_time();
    
    test_log_args(123, "test-session".to_string(), 42);
    
    if let Err(err) = test_log_retries() {
        log_warn!(&format!("Retry function ultimately failed: {:?}", err));
    }
    
    // Handle Result from test_catch_panic
    if let Err(err) = test_catch_panic() {
        log_warn!(&format!("Panic catching test failed: {:?}", err));
    }
    
    // Fix function calls that were generating errors
    if let Ok(value) = log_result_test() {
        log_info!(&format!("Result test returned: {:?}", value));
    }
    
    audit_log_test(123, "update profile");
    
    if let Err(err) = test_circuit_breaker(true) {
        log_warn!(&format!("Circuit breaker test: {:?}", err));
    }
    
    // Handle Result from test_health_check
    if let Err(err) = test_health_check() {
        log_warn!(&format!("Health check failed: {:?}", err));
    }
    
    test_throttle_log();
    
    // Handle Result from dependency_latency_test
    if let Err(err) = dependency_latency_test() {
        log_warn!(&format!("Dependency latency test failed: {:?}", err));
    }
    
    test_log_response();
    
    test_log_concurrency();
    
    test_trace_span();
    
    feature_flag_test();
    
    metrics_counter_test();
    
    test_log_memory_usage();
    
    test_log_cpu_time();
    
    test_version_tag();
    
    test_request_context();
    
    // Test the async logger
    test_async_logger();
    
    // Test all DevOps monitoring macros
    run_all_devops_tests();
    
    log_info!("All tests completed!");
}

// Example functions demonstrating each of the procedural macros

#[log_entry_exit]
fn test_log_entry_exit() {
    log_info!("Inside the entry_exit test function");
    std::thread::sleep(std::time::Duration::from_millis(50));
}

#[log_errors]
fn test_log_errors() -> Result<(), String> {
    if rand::random::<bool>() {
        Err("Simulated error for testing".to_string())
    } else {
        Ok(())
    }
}

#[measure_time]
fn test_measure_time() {
    log_info!("Testing time measurement");
    std::thread::sleep(std::time::Duration::from_millis(100));
}

#[log_args(user_id, session_id)]
fn test_log_args(user_id: i32, session_id: String, other: i32) {
    log_info!(&format!("Function with logged args called, other={}", other));
}

#[log_retries(max_attempts=3)]
fn test_log_retries() -> Result<(), String> {
    // Simulate random failures
    if rand::thread_rng().gen_range(0..3) != 0 {
        Err("Temporary failure, please retry".to_string())
    } else {
        Ok(())
    }
}

#[catch_panic]
fn test_catch_panic() -> Result<(), String> {
    log_info!("Testing panic catching");
    
    if rand::random::<bool>() {
        // Uncomment to test panic handling
        // panic!("Test panic that should be caught");
    }
    
    Ok(())
}

// Rename to avoid the "expected identifier" errors
#[log_result]
fn log_result_test() -> Result<String, String> {
    if rand::random::<bool>() {
        Ok("Success result value".to_string())
    } else {
        Err("Failure with detailed error info".to_string())
    }
}

// Rename to avoid the "expected identifier" errors
#[audit_log]
fn audit_log_test(_user_id: i32, action: &str) {
    log_info!(&format!("User performing action: {}", action));
}

#[circuit_breaker(failure_threshold=2)]
fn test_circuit_breaker(should_fail: bool) -> Result<(), String> {
    if should_fail {
        Err("Simulated failure for circuit breaker".to_string())
    } else {
        Ok(())
    }
}

#[health_check]
fn test_health_check() -> Result<(), String> {
    // Simulate health check with some delay
    std::thread::sleep(std::time::Duration::from_millis(30));
    
    if rand::random::<bool>() {
        Ok(())
    } else {
        Err("Health check failed: database unreachable".to_string())
    }
}

#[throttle_log(rate=5)]
fn test_throttle_log() {
    // Call multiple times to test throttling
    for i in 0..10 {
        log_info!(&format!("Log message {}", i));
    }
}

// Rename to avoid the "expected identifier" errors
#[dependency_latency]
fn dependency_latency_test() -> Result<(), String> {
    // Simulate database call
    std::thread::sleep(std::time::Duration::from_millis(120));
    
    if rand::random::<bool>() {
        Ok(())
    } else {
        Err("Database timeout after 2s".to_string())
    }
}

#[log_response]
fn test_log_response() -> String {
    "This response will be logged".to_string()
}

#[log_concurrency]
fn test_log_concurrency() {
    log_info!("Testing concurrency logging");
    std::thread::sleep(std::time::Duration::from_millis(50));
}

#[trace_span]
fn test_trace_span() {
    log_info!("Function with trace ID");
    
    // Nested function call should use same trace ID
    nested_trace_function();
}

#[trace_span]
fn nested_trace_function() {
    log_info!("Nested function with same trace ID");
}

// Rename to avoid the "expected identifier" errors
#[feature_flag]
fn feature_flag_test() {
    log_info!("Function with feature flag");
}

// Rename to avoid the "expected identifier" errors
#[metrics_counter]
fn metrics_counter_test() {
    log_info!("Function with metrics counter");
}

#[log_memory_usage]
fn test_log_memory_usage() {
    log_info!("Testing memory usage logging");
    
    // Allocate some memory to see change
    let _data = vec![0u8; 1024 * 1024];
    std::thread::sleep(std::time::Duration::from_millis(50));
}

#[log_cpu_time]
fn test_log_cpu_time() {
    log_info!("Testing CPU time logging");
    
    // Do some CPU-intensive work
    // Fix: Use u64 to prevent overflow and use wrapping_add to safely handle potential overflow
    let mut sum: u64 = 0;
    for i in 0..1000000u64 {
        sum = sum.wrapping_add(i);
    }
    log_debug!(&format!("Sum: {}", sum));
}

#[version_tag]
fn test_version_tag() {
    log_info!("Function with version tag");
}

#[request_context]
fn test_request_context() {
    log_info!("Function with request context");
}

// Custom logger initialization to ensure all logs are displayed
fn initialize_custom_logger() {
    // Initialize logger with debug threshold to ensure all logs are shown
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
    
    // Print a clear marker to see if logger is working
    log_info!("======== LOGGER TEST STARTED ========");
    log_debug!("Debug logging is enabled");
    log_info!("Info logging is enabled");
    log_warn!("Warning logging is enabled");
    log_error!("Error logging is enabled");
}
