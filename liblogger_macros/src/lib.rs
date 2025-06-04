/*
 * Procedural macros for enhanced logging capabilities
 *
 * This module provides procedural macros that can be applied to functions
 * for various logging, monitoring, and instrumentation purposes.
 * 
 * These macros work with the liblogger crate to provide automatic context
 * capturing, timing measurements, and other advanced logging features.
 */

extern crate proc_macro;

// Import our utils module (keep it private)
mod macro_utils;

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, parse_quote, ItemFn};

// Import helpers from our utils module
use crate::macro_utils::{get_fn_name, IdList, MacroArgs, define_helper_functions, generate_utility_functions};

/// Initialization macro that must be called at the module level to enable attribute macros
///
/// This macro defines helper functions needed by the attribute macros, such as
/// error extraction, success checking, trace ID management, and feature flag checking.
///
#[proc_macro]
pub fn initialize_logger_attributes(_input: TokenStream) -> TokenStream {
    TokenStream::from(define_helper_functions())
}

/// Logs function entry and exit points to track execution flow
///
/// Automatically adds INFO level logs at the start and end of the function.
/// Useful for tracing code execution paths during debugging and in production.
///
#[proc_macro_attribute]
pub fn log_entry_exit(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        liblogger::log_info!(&format!("ENTRY: {}", #fn_name));
        
        let result = (|| #orig_block)();
        
        liblogger::log_info!(&format!("EXIT: {}", #fn_name));
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Log errors and panics
#[proc_macro_attribute]
pub fn log_errors(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let is_async = input_fn.sig.asyncness.is_some();
    
    if is_async {
        input_fn.block = Box::new(parse_quote!({
            async move {
                let result = async move #orig_block.await;
                
                // Use pattern matching to handle Result types
                match &result {
                    Ok(_) => {},  // Success case, no logging needed
                    Err(err) => {
                        // Error case, log the error
                        liblogger::log_error!(&format!("{} returned error: {:?}", #fn_name, err), None);
                    }
                }
                result
            }.await
        }));
    } else {
        input_fn.block = Box::new(parse_quote!({
            use std::panic::{catch_unwind, AssertUnwindSafe};
            
            let result = catch_unwind(AssertUnwindSafe(|| #orig_block));
            
            match result {
                Ok(inner_result) => {
                    // Use pattern matching to handle Result types
                    match &inner_result {
                        Ok(_) => {},  // Success case, no logging needed
                        Err(err) => {
                            // Error case, log the error
                            liblogger::log_error!(&format!("{} returned error: {:?}", #fn_name, err), None);
                        }
                    }
                    inner_result
                },
                Err(panic_err) => {
                    let panic_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_err.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown panic".to_string()
                    };
                    
                    liblogger::log_error!(&format!("{} panicked: {}", #fn_name, panic_msg), None);
                    std::panic::resume_unwind(panic_err);
                }
            }
        }));
    }
    
    TokenStream::from(quote!(#input_fn))
}

/// Measure execution time of a function
#[proc_macro_attribute]
pub fn measure_time(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let is_async = input_fn.sig.asyncness.is_some();
    
    if is_async {
        input_fn.block = Box::new(parse_quote!({
            async move {
                use std::time::Instant;
                
                let start_time = Instant::now();
                let result = async move #orig_block.await;
                let duration = start_time.elapsed();
                let duration_ms = duration.as_millis();
                
                liblogger::log_info!(&format!("{} completed in {} ms ", #fn_name, duration_ms), None);
                result
            }.await
        }));
    } else {
        input_fn.block = Box::new(parse_quote!({
            use std::time::Instant;
            use std::panic::{catch_unwind, AssertUnwindSafe};
            
            let start_time = Instant::now();
            
            let result = catch_unwind(AssertUnwindSafe(|| #orig_block));
            
            let duration = start_time.elapsed();
            let duration_ms = duration.as_millis();
            
            match result {
                Ok(output) => {
                    liblogger::log_info!(&format!("{} completed in {} ms ", #fn_name, duration_ms), None);
                    output
                },
                Err(panic_err) => {
                    liblogger::log_error!(
                        &format!("{} panicked after {} ms ", #fn_name, duration_ms), 
                        None
                    );
                    std::panic::resume_unwind(panic_err);
                }
            }
        }));
    }
    
    TokenStream::from(quote!(#input_fn))
}

/// Log specified function arguments
#[proc_macro_attribute]
pub fn log_args(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as IdList);
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let arg_names = args.ids;
    let mut log_stmts = Vec::new();
    
    for arg_name in &arg_names {
        let arg_str = arg_name.to_string();
        log_stmts.push(quote! {
            let arg_value = format!("{:?}", #arg_name);
            args_str.push_str(&format!("{} = {}, ", #arg_str, arg_value));
        });
    }
    
    input_fn.block = Box::new(parse_quote!({
        use std::time::Instant;
        let start_time = Instant::now();
        let mut args_str = String::new();
        #(#log_stmts)*;
        // Remove trailing comma and space
        if !args_str.is_empty() {
            args_str.truncate(args_str.len() - 2);
        }
        liblogger::log_info!(&format!("Entering {} with args: {}", #fn_name, args_str), None);
        #orig_block
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Log and implement retry logic
#[proc_macro_attribute]
pub fn log_retries(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let max_attempts = args.max_attempts.unwrap_or(3);
      let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let is_async = input_fn.sig.asyncness.is_some();

    if is_async {
        input_fn.block = Box::new(parse_quote!({
            async move {
                let mut attempts = 0u32;
                loop {
                    attempts += 1;
                    if attempts > 1 {
                        liblogger::log_info!(
                            &format!("Retry attempt {} of {} for {}", attempts, #max_attempts, #fn_name), 
                            None
                        );
                        // For async functions, we skip the delay to avoid tokio dependency
                        // The user should implement their own delay if needed
                        liblogger::log_info!(
                            &format!("Async retry delay skipped for {} (implement your own async delay if needed)", #fn_name), 
                            None
                        );
                    }
                    
                    let result = async move #orig_block.await;
                    
                    // Use pattern matching to determine success or failure
                    match &result {
                        Ok(_) => {
                            // Success case
                            if attempts > 1 {
                                liblogger::log_info!(
                                    &format!("{} succeeded after {} attempts", #fn_name, attempts), 
                                    None
                                );
                            }
                            return result;
                        },
                        Err(err) => {
                            // Error case
                            if attempts >= #max_attempts {
                                liblogger::log_error!(
                                    &format!("{} failed after {} attempts: {:?}", #fn_name, attempts, err), 
                                    None
                                );
                                return result;
                            }
                            
                            liblogger::log_warn!(
                                &format!("{} attempt {} failed: {:?}", #fn_name, attempts, err), 
                                None
                            );
                            // Continue to next retry iteration
                        }
                    }
                }
            }.await
        }));
    } else {
        input_fn.block = Box::new(parse_quote!({
            let mut attempts = 0u32;
            loop {
                attempts += 1;
                if attempts > 1 {
                    liblogger::log_info!(
                        &format!("Retry attempt {} of {} for {}", attempts, #max_attempts, #fn_name), 
                        None
                    );
                    // Simple exponential backoff
                    std::thread::sleep(std::time::Duration::from_millis((2u64.pow(attempts - 1) * 50) as u64));
                }
                
                let result = (|| #orig_block)();
                
                // Use pattern matching to determine success or failure
                match &result {
                    Ok(_) => {
                        // Success case
                        if attempts > 1 {
                            liblogger::log_info!(
                                &format!("{} succeeded after {} attempts", #fn_name, attempts), 
                                None
                            );
                        }
                        return result;
                    },
                    Err(err) => {
                        // Error case
                        if attempts >= #max_attempts {
                            liblogger::log_error!(
                                &format!("{} failed after {} attempts: {:?}", #fn_name, attempts, err), 
                                None
                            );
                            return result;
                        }
                        
                        liblogger::log_warn!(
                            &format!("{} attempt {} failed: {:?}", #fn_name, attempts, err), 
                            None
                        );
                        // Continue to next retry iteration
                    }
                }
            }
        }));
    }
    
    TokenStream::from(quote!(#input_fn))
}

/// Create detailed audit logs
#[proc_macro_attribute]
pub fn audit_log(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let is_async = input_fn.sig.asyncness.is_some();
    
    if is_async {
        input_fn.block = Box::new(parse_quote!({
            async move {
                let user_id = get_thread_local_value("user_id").unwrap_or_else(|| "unknown".to_string());
                liblogger::log_info!(&format!("AUDIT: {} called", #fn_name), Some(format!("user_id={}", user_id)));
                
                let start_time = std::time::Instant::now();
                let result = async move #orig_block.await;
                let duration = start_time.elapsed();
                
                liblogger::log_info!(
                    &format!("AUDIT: {} completed in {} ms", #fn_name, duration.as_millis()),
                    Some(format!("user_id={}", user_id))
                );
                
                result
            }.await
        }));
    } else {
        input_fn.block = Box::new(parse_quote!({
            let user_id = get_thread_local_value("user_id").unwrap_or_else(|| "unknown".to_string());
            liblogger::log_info!(&format!("AUDIT: {} called", #fn_name), Some(format!("user_id={}", user_id)));
            
            let start_time = std::time::Instant::now();
            let result = #orig_block;
            let duration = start_time.elapsed();
            
            // Use pattern matching on result
            match &result {
                () => {
                    // Unit return type
                    liblogger::log_info!(
                        &format!("AUDIT: {} completed in {} ms", #fn_name, duration.as_millis()),
                        Some(format!("user_id={}", user_id))
                    );
                },
                _ => {
                    // Any other return type
                    liblogger::log_info!(
                        &format!("AUDIT: {} completed in {} ms with result: {:?}", 
                            #fn_name, duration.as_millis(), result),
                        Some(format!("user_id={}", user_id))
                    );
                }
            }
            
            result
        }));
    }
    
    TokenStream::from(quote!(#input_fn))
}

/// Circuit breaker pattern with logging
#[proc_macro_attribute]
pub fn circuit_breaker(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let threshold = args.failure_threshold.unwrap_or(3);
    
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let is_async = input_fn.sig.asyncness.is_some();
    
    if is_async {
        input_fn.block = Box::new(parse_quote!({
            async move {
                use std::sync::atomic::{AtomicU32, Ordering};
                use std::sync::Mutex;
                use std::time::{Instant, Duration};
                
                // Thread-safe failure counters
                static FAILURE_COUNT: AtomicU32 = AtomicU32::new(0);
                static LAST_SUCCESS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                
                // Reset failure count after 30 seconds of success
                let now = Instant::now();
                let last_success_time = LAST_SUCCESS.load(Ordering::Relaxed);
                
                if last_success_time > 0 {
                    let elapsed = now.duration_since(Instant::now() - Duration::from_secs(last_success_time));
                    if elapsed > Duration::from_secs(30) {
                        FAILURE_COUNT.store(0, Ordering::Relaxed);
                    }
                }
                
                // Check if circuit is open (too many failures)
                let failures = FAILURE_COUNT.load(Ordering::Relaxed);
                if failures >= #threshold {
                    liblogger::log_error!(
                        &format!("Circuit breaker open for {}: {} failures exceeded threshold {}", 
                            #fn_name, failures, #threshold),
                        None
                    );
                    return Err(format!("Circuit breaker open for {}", #fn_name).into());
                }
                
                // Call the function and track success/failure
                let result = async move #orig_block.await;
                
                // Use pattern matching for Result
                match &result {
                    Ok(_) => {
                        // Reset failure count on success
                        FAILURE_COUNT.store(0, Ordering::Relaxed);
                        LAST_SUCCESS.store(now.elapsed().as_secs(), Ordering::Relaxed);
                    },
                    Err(_) => {
                        // Increment failure count
                        FAILURE_COUNT.fetch_add(1, Ordering::Relaxed);
                        let new_count = FAILURE_COUNT.load(Ordering::Relaxed);
                        
                        liblogger::log_warn!(&format!(
                            "Circuit breaker: {} failed ({}/{} failures)", 
                            #fn_name, new_count, #threshold
                        ), None);
                    }
                }
                
                result
            }.await
        }));
    } else {
        input_fn.block = Box::new(parse_quote!({
            use std::sync::atomic::{AtomicU32, Ordering};
            use std::sync::Mutex;
            use std::time::{Instant, Duration};
            
            // Thread-safe failure counters
            static FAILURE_COUNT: AtomicU32 = AtomicU32::new(0);
            static LAST_SUCCESS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            
            // Reset failure count after 30 seconds of success
            let now = Instant::now();
            let last_success_time = LAST_SUCCESS.load(Ordering::Relaxed);
            
            if last_success_time > 0 {
                let elapsed = now.duration_since(Instant::now() - Duration::from_secs(last_success_time));
                if elapsed > Duration::from_secs(30) {
                    FAILURE_COUNT.store(0, Ordering::Relaxed);
                }
            }
            
            // Check if circuit is open (too many failures)
            let failures = FAILURE_COUNT.load(Ordering::Relaxed);
            if failures >= #threshold {
                liblogger::log_error!(
                    &format!("Circuit breaker open for {}: {} failures exceeded threshold {}", 
                        #fn_name, failures, #threshold),
                    None
                );
                return Err(format!("Circuit breaker open for {}", #fn_name).into());
            }
            
            // Call the function and track success/failure
            let result = #orig_block;
            
            // Use pattern matching for Result
            match &result {
                Ok(_) => {
                    // Reset failure count on success
                    FAILURE_COUNT.store(0, Ordering::Relaxed);
                    LAST_SUCCESS.store(now.elapsed().as_secs(), Ordering::Relaxed);
                },
                Err(_) => {
                    // Increment failure count
                    FAILURE_COUNT.fetch_add(1, Ordering::Relaxed);
                    let new_count = FAILURE_COUNT.load(Ordering::Relaxed);
                    
                    liblogger::log_warn!(&format!(
                        "Circuit breaker: {} failed ({}/{} failures)", 
                        #fn_name, new_count, #threshold
                    ), None);
                }
            }
            
            result
        }));
    }
    
    TokenStream::from(quote!(#input_fn))
}

/// Throttle logs to avoid flooding during incidents
#[proc_macro_attribute]
pub fn throttle_log(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let rate = args.rate.unwrap_or(5);
    
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        static LAST_MINUTE: AtomicUsize = AtomicUsize::new(0);
        static SKIPPED_COUNT: AtomicUsize = AtomicUsize::new(0);
        
        // Get current minute for rate limiting window
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let current_minute = (now.as_secs() / 60) as usize;
        
        // Check if we're in a new minute or still in the rate limit
        let should_log = {
            let last_minute = LAST_MINUTE.load(Ordering::SeqCst);
            if last_minute != current_minute {
                // New minute, reset counter and log a summary of skipped messages
                LAST_MINUTE.store(current_minute, Ordering::SeqCst);
                let skipped = SKIPPED_COUNT.swap(0, Ordering::SeqCst);
                if skipped > 0 {
                    liblogger::log_info!(
                        &format!("Throttled logs for {}: skipped {} logs in previous minute", 
                            #fn_name, skipped),
                        None
                    );
                }
                COUNTER.store(1, Ordering::SeqCst);
                true
            } else {
                // Same minute, check counter
                let count = COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
                if count <= #rate as usize {
                    true
                } else {
                    SKIPPED_COUNT.fetch_add(1, Ordering::SeqCst);
                    false
                }
            }
        };
        
        let result = #orig_block;
        
        // Only log if within rate limits
        if should_log {
            // Simple logging without trying to match on the result type
            liblogger::log_info!(&format!("{} executed", #fn_name), None);
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Measure latency to external dependencies
#[proc_macro_attribute]
pub fn dependency_latency(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let target = args.target.unwrap_or_else(|| "unknown".to_string());
    
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        use std::time::Instant;
        liblogger::log_info!(
            &format!("Dependency call to {} started for {}", #target, #fn_name),
            None
        );
        let start_time = Instant::now();
        let result = #orig_block;
        let duration_ms = start_time.elapsed().as_millis();
        
        // Use pattern matching to handle different result types
        match &result {
            Ok(_) => {
                liblogger::log_info!(&format!("Dependency call to {} completed in {} ms", #target, duration_ms), None);
            },
            Err(err) => {
                liblogger::log_error!(
                    &format!("Dependency call to {} failed after {} ms with error: {:?}",
                        #target, duration_ms, err),
                    None
                );
            },
            _ => {
                // For non-Result types
                liblogger::log_info!(&format!("Dependency call to {} completed in {} ms", #target, duration_ms), None);
            }
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Log the returned value from a function
#[proc_macro_attribute]
pub fn log_response(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let result = #orig_block;
        liblogger::log_debug!(&format!("{} returned: {:?}", #fn_name, result), None);
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Track concurrent invocations of a function
#[proc_macro_attribute]
pub fn log_concurrency(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let counter_var = format_ident!("CONCURRENCY_{}", fn_name.to_uppercase());
    
    input_fn.block = Box::new(parse_quote!({
        use std::sync::atomic::{AtomicU32, Ordering};
        static #counter_var: AtomicU32 = AtomicU32::new(0);
        
        let current = #counter_var.fetch_add(1, Ordering::SeqCst) + 1;
        liblogger::log_debug!(
            &format!("{} concurrent invocations: {}", #fn_name, current),
            None
        );
        
        let result = #orig_block;
        
        let after = #counter_var.fetch_sub(1, Ordering::SeqCst) - 1;
        liblogger::log_debug!(
            &format!("{} concurrent invocations after exit: {}", #fn_name, after),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Create and propagate a trace ID for request flow tracking
#[proc_macro_attribute]
pub fn trace_span(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        use uuid::Uuid;
        // Generate or reuse trace ID
        let trace_id = if let Some(existing_id) = get_trace_id() {
            existing_id
        } else {
            let new_id = Uuid::new_v4().to_string();
            set_trace_id(&new_id);
            new_id
        };
        
        liblogger::log_info!(
            &format!("[TraceID: {}] {} started", trace_id, #fn_name),
            None
        );
        
        let result = #orig_block;
        
        liblogger::log_info!(
            &format!("[TraceID: {}] {} completed", trace_id, #fn_name),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Log feature flag state
#[proc_macro_attribute]
pub fn feature_flag(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let flag_name = args.flag_name.unwrap_or_else(|| "unknown".to_string());
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        // Check feature flag (placeholder function)
        let is_enabled = is_feature_enabled(#flag_name);
        
        liblogger::log_info!(
            &format!("{} called with feature flag {} = {}", 
                #fn_name, #flag_name, is_enabled),
            None
        );
        
        let result = #orig_block;
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Increment a metrics counter for function calls
#[proc_macro_attribute]
pub fn metrics_counter(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let counter_name = args.counter_name.unwrap_or_else(|| "function_calls".to_string());
        
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let orig_block = input_fn.block.clone();
      input_fn.block = Box::new(parse_quote!({
        // Increment counter using Prometheus
        {
            use prometheus::{Counter, register_counter};
            use std::sync::Once;
            static INIT: Once = Once::new();
            static mut COUNTER: Option<Counter> = None;
            
            INIT.call_once(|| {
                let counter = register_counter!(#counter_name, "Function call counter").unwrap();
                unsafe {
                    COUNTER = Some(counter);
                }
            });
            
            if let Some(counter) = unsafe { COUNTER.as_ref() } {
                counter.inc();
            }
        }
        
        let result = #orig_block;
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Log memory usage during function execution
#[proc_macro_attribute]
pub fn log_memory_usage(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
      input_fn.block = Box::new(parse_quote!({
        let (start_rss, start_vms) = {
            use psutil::process::Process;
            let process = Process::current().unwrap();
            let memory = process.memory_info().unwrap();
            (memory.rss(), memory.vms())
        };
        
        let result = #orig_block;
        
        {
            use psutil::process::Process;
            let process = Process::current().unwrap();
            let memory = process.memory_info().unwrap();
            let end_rss = memory.rss();
            let end_vms = memory.vms();
            
            liblogger::log_info!(
                &format!("{} starting memory usage - RSS: {} bytes, VMS: {} bytes", 
                    #fn_name, start_rss, start_vms),
                None
            );
            liblogger::log_info!(
                &format!("{} ending memory usage - RSS: {} bytes (delta: {} bytes), VMS: {} bytes (delta: {} bytes)", 
                    #fn_name, end_rss, end_rss as i64 - start_rss as i64, 
                    end_vms, end_vms as i64 - start_vms as i64),
                None
            );
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Log CPU time used during function execution
#[proc_macro_attribute]
pub fn log_cpu_time(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        use std::time::Instant;
        let wall_time_start = Instant::now();
        
        // There's no direct CPU time measurement in standard Rust
        // This is just a placeholder that measures wall time
        let result = #orig_block;
        let wall_time = wall_time_start.elapsed();
        
        liblogger::log_info!(
            &format!("{} used CPU time: approx {} ms (wall time)", 
                #fn_name, wall_time.as_millis()),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Include version information in logs
#[proc_macro_attribute]
pub fn version_tag(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let version = std::env::var("BUILD_VERSION").unwrap_or_else(|_| "unknown".to_string());
        liblogger::log_info!(
            &format!("[Version: {}] {} called", version, #fn_name),
            None
        );
        
        let result = #orig_block;
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Attach request context to logs
#[proc_macro_attribute]
pub fn request_context(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        // Get context from thread-local storage (placeholder)
        let user_id = get_thread_local_value("user_id");
        let session_id = get_thread_local_value("session_id");
        let request_id = get_thread_local_value("request_id");
        
        let mut context_parts = Vec::new();
        if let Some(id) = user_id {
            context_parts.push(format!("user_id={}", id));
        }
        if let Some(id) = session_id {
            context_parts.push(format!("session_id={}", id));
        }
        if let Some(id) = request_id {
            context_parts.push(format!("request_id={}", id));
        }
        
        let context_str = if !context_parts.is_empty() {
            context_parts.join(", ")
        } else {
            "No context available".to_string()
        };
        
        liblogger::log_info!(
            &format!("{} called", #fn_name),
            Some(context_str)
        );
        
        let result = #orig_block;
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Catch and log panics but don't crash
#[proc_macro_attribute]
pub fn catch_panic(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let is_async = input_fn.sig.asyncness.is_some();
    
    // Determine return type
    let returns_result = if let syn::ReturnType::Type(_, ty) = &input_fn.sig.output {
        if let syn::Type::Path(type_path) = ty.as_ref() {
            let last_segment = type_path.path.segments.last().unwrap();
            last_segment.ident.to_string() == "Result"
        } else {
            false
        }
    } else {
        false
    };
    
    if is_async {
        // For async functions, we can't use catch_unwind effectively
        // Instead, we just wrap the execution and handle errors at the Result level
        if returns_result {
            input_fn.block = Box::new(parse_quote!({
                async move {
                    let result = async move #orig_block.await;
                    
                    // Log errors if they occur
                    if let Err(ref err) = result {
                        liblogger::log_error!(&format!("{} returned error: {:?}", #fn_name, err), None);
                    }
                    
                    result
                }.await
            }));
        } else {
            input_fn.block = Box::new(parse_quote!({
                async move {
                    let result = async move #orig_block.await;
                    result
                }.await
            }));
        }
    } else {
        input_fn.block = if returns_result {
            Box::new(parse_quote!({
                use std::panic::{catch_unwind, AssertUnwindSafe};
                
                match catch_unwind(AssertUnwindSafe(|| #orig_block)) {
                    Ok(result) => result,
                    Err(panic_err) => {
                        let panic_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                            s.to_string()
                        } else if let Some(s) = panic_err.downcast_ref::<String>() {
                            s.clone()
                        } else {
                            "Unknown panic ".to_string()
                        };
                        
                        liblogger::log_error!(&format!("{} caught panic: {}", #fn_name, panic_msg), None);
                        Err(format!("Panic in {}: {}", #fn_name, panic_msg).into())
                    }
                }
            }))
        } else {
            Box::new(parse_quote!({
                use std::panic::{catch_unwind, AssertUnwindSafe};
                
                match catch_unwind(AssertUnwindSafe(|| #orig_block)) {
                    Ok(result) => result,
                    Err(panic_err) => {
                        let panic_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                            s.to_string()
                        } else if let Some(s) = panic_err.downcast_ref::<String>() {
                            s.clone()
                        } else {
                            "Unknown panic ".to_string()
                        };
                        
                        liblogger::log_error!(&format!("{} caught panic: {}", #fn_name, panic_msg), None);
                        // Return default value as fallback
                        Default::default()
                    }
                }
            }))
        };
    }
    
    TokenStream::from(quote!(#input_fn))
}

/// Log health check results
#[proc_macro_attribute]
pub fn health_check(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        use std::time::Instant;
        
        let start_time = Instant::now();
        let result = #orig_block;
        let duration = start_time.elapsed();
        
        // Use pattern matching to determine success or failure
        match &result {
            Ok(_) => {
                liblogger::log_info!(
                    &format!("Health check {} passed in {} ms", #fn_name, duration.as_millis()),
                    None
                );
            },
            Err(err) => {
                liblogger::log_error!(
                    &format!("Health check {} failed in {} ms: {:?}", 
                        #fn_name, duration.as_millis(), err),
                    None
                );
            }
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Log function result with different levels for success/error
#[proc_macro_attribute] 
pub fn log_result(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let success_level = args.success_level.unwrap_or_else(|| "info".to_string());
    let error_level = args.error_level.unwrap_or_else(|| "error".to_string());
    
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    // Create string literals for the different log levels to avoid str_as_str
    let success_level_str = success_level.clone();
    let error_level_str = error_level.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let result = #orig_block;
        
        // Use pattern matching to handle the Result
        match &result {
            Ok(val) => {
                // Success case with different log levels
                let level = #success_level_str;
                if level == "debug" {
                    liblogger::log_debug!(&format!("{} succeeded with result: {:?}", #fn_name, val), None);
                } else if level == "warn" {
                    liblogger::log_warn!(&format!("{} succeeded with result: {:?}", #fn_name, val), None);
                } else if level == "error" {
                    liblogger::log_error!(&format!("{} succeeded with result: {:?}", #fn_name, val), None);
                } else {
                    liblogger::log_info!(&format!("{} succeeded with result: {:?}", #fn_name, val), None);
                }
            },
            Err(err) => {
                // Error case with different log levels
                let level = #error_level_str;
                if level == "debug" {
                    liblogger::log_debug!(&format!("{} failed with error: {:?}", #fn_name, err), None);
                } else if level == "info" {
                    liblogger::log_info!(&format!("{} failed with error: {:?}", #fn_name, err), None);
                } else if level == "warn" {
                    liblogger::log_warn!(&format!("{} failed with error: {:?}", #fn_name, err), None);
                } else {
                    liblogger::log_error!(&format!("{} failed with error: {:?}", #fn_name, err), None);
                }
            }
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

// ====================
// DevOps Infrastructure Macros
// ====================

/// Monitor disk usage and alert on threshold breaches
#[proc_macro_attribute]
pub fn log_disk_usage(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let threshold = args.threshold.unwrap_or(80) as u64; // Convert to u64
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        // Inject utility functions directly into the generated code
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let disk_info_before = get_disk_info();
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let disk_info_after = get_disk_info();
        let disk_change = if disk_info_after.used_percentage > disk_info_before.used_percentage {
            disk_info_after.used_percentage - disk_info_before.used_percentage
        } else {
            0.0
        };
        
        let current_usage = disk_info_after.used_percentage as u64;
        let formatted_disk_info = format_disk_info(&disk_info_after);
        
        if current_usage >= #threshold {
            liblogger::log_warn!(
                &format!("DISK_ALERT: {} - High disk usage detected: {}% (threshold: {}%) | {} | Change: +{:.1}% | Duration: {}ms", 
                    #fn_name, current_usage, #threshold, formatted_disk_info, disk_change, duration.as_millis()),
                None
            );
        } else {
            liblogger::log_info!(
                &format!("DISK_MONITOR: {} - Disk usage: {}% (threshold: {}%) | {} | Change: +{:.1}% | Duration: {}ms", 
                    #fn_name, current_usage, #threshold, formatted_disk_info, disk_change, duration.as_millis()),
                None
            );
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor network connectivity and detect connection issues
#[proc_macro_attribute]
pub fn log_network_connectivity(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let endpoint = args.endpoint.unwrap_or_else(|| "8.8.8.8:53".to_string());
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        // Inject utility functions directly into the generated code
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let network_info_before = get_network_interfaces();
        let connectivity_before = check_network_connectivity(&#endpoint);
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let network_info_after = get_network_interfaces();
        let connectivity_after = check_network_connectivity(&#endpoint);
        let formatted_network_info = format_network_info(&network_info_after);
        
        if connectivity_before && connectivity_after {
            liblogger::log_info!(
                &format!("NETWORK_OK: {} - Connectivity maintained to {} | {} | Duration: {}ms", 
                    #fn_name, #endpoint, formatted_network_info, duration.as_millis()),
                None
            );
        } else if !connectivity_before && connectivity_after {
            liblogger::log_info!(
                &format!("NETWORK_RECOVERED: {} - Connectivity restored to {} | {} | Duration: {}ms", 
                    #fn_name, #endpoint, formatted_network_info, duration.as_millis()),
                None
            );
        } else if connectivity_before && !connectivity_after {
            liblogger::log_error!(
                &format!("NETWORK_LOST: {} - Connectivity lost to {} | {} | Duration: {}ms", 
                    #fn_name, #endpoint, formatted_network_info, duration.as_millis()),
                None
            );
        } else {
            liblogger::log_warn!(
                &format!("NETWORK_DOWN: {} - No connectivity to {} | {} | Duration: {}ms", 
                    #fn_name, #endpoint, formatted_network_info, duration.as_millis()),
                None
            );
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor database connection pool health and performance
#[proc_macro_attribute]
pub fn log_database_pool(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let pool_name = args.pool_name.unwrap_or_else(|| "default".to_string());
    let threshold = args.threshold.unwrap_or(80) as u64; // Convert to u64
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        // Inject utility functions directly into the generated code
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let pool_stats_before = get_db_pool_stats(&#pool_name);
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let pool_stats_after = get_db_pool_stats(&#pool_name);
        let formatted_pool_info = format_db_pool_info(&pool_stats_after);
        
        let utilization = pool_stats_after.utilization_percentage;
        
        if utilization >= #threshold as f64 {
            liblogger::log_warn!(
                &format!("DB_POOL_ALERT: {} - High pool utilization: {:.1}% (threshold: {}%) | Pool: {} | {} | Duration: {}ms", 
                    #fn_name, utilization, #threshold, #pool_name, formatted_pool_info, duration.as_millis()),
                None
            );
        } else {
            liblogger::log_info!(
                &format!("DB_POOL_MONITOR: {} - Pool utilization: {:.1}% | Pool: {} | {} | Duration: {}ms", 
                    #fn_name, utilization, #pool_name, formatted_pool_info, duration.as_millis()),
                None
            );
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor file descriptor usage and detect resource leaks
#[proc_macro_attribute]
pub fn log_file_descriptors(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let threshold = args.threshold.unwrap_or(1000) as u64; // Convert to u64
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        // Inject utility functions directly into the generated code
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let fd_count_before = get_fd_count();
        let fd_limit = get_fd_limit();
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let fd_count_after = get_fd_count();
        let fd_change = if fd_count_after > fd_count_before { 
            fd_count_after - fd_count_before 
        } else { 
            0 
        };
        let formatted_fd_info = format_fd_info(fd_count_after, fd_limit);
        
        if fd_count_after >= #threshold {
            liblogger::log_warn!(
                &format!("FD_ALERT: {} - High file descriptor usage: {} (threshold: {}) | {} | Change: +{} | Duration: {}ms", 
                    #fn_name, fd_count_after, #threshold, formatted_fd_info, fd_change, duration.as_millis()),
                None
            );
        } else {
            liblogger::log_info!(
                &format!("FD_MONITOR: {} - File descriptors: {} | {} | Change: +{} | Duration: {}ms", 
                    #fn_name, fd_count_after, formatted_fd_info, fd_change, duration.as_millis()),
                None
            );
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor cache hit ratio and performance metrics
#[proc_macro_attribute]
pub fn log_cache_hit_ratio(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let threshold = args.threshold.unwrap_or(70);
    let cache_name = args.cache_name.unwrap_or_else(|| "default".to_string());
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let cache_stats_before = get_cache_stats(&#cache_name);
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let cache_stats_after = get_cache_stats(&#cache_name);
        let formatted_cache_info = format_cache_info(&cache_stats_after);
        
        let hit_ratio = cache_stats_after.hit_ratio_percentage;
        
        if hit_ratio < #threshold as f64 {
            liblogger::log_warn!(
                &format!("CACHE_ALERT: {} - Low cache hit ratio: {:.1}% (threshold: {}%) | Cache: {} | {} | Duration: {}ms", 
                    #fn_name, hit_ratio, #threshold, #cache_name, formatted_cache_info, duration.as_millis()),
                None
            );
        } else {
            liblogger::log_info!(
                &format!("CACHE_MONITOR: {} - Cache hit ratio: {:.1}% | Cache: {} | {} | Duration: {}ms", 
                    #fn_name, hit_ratio, #cache_name, formatted_cache_info, duration.as_millis()),
                None
            );
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor queue depth and processing performance
#[proc_macro_attribute]
pub fn log_queue_depth(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let queue_name = args.queue_name.unwrap_or_else(|| "default".to_string());
    let threshold = args.threshold.unwrap_or(1000) as u64; // Convert to u64
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let queue_stats_before = get_queue_stats(&#queue_name);
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let queue_stats_after = get_queue_stats(&#queue_name);
        let formatted_queue_info = format_queue_info(&queue_stats_after);
        
        let queue_depth = queue_stats_after.depth;
        let processing_rate = queue_stats_after.processing_rate;
        
        if queue_depth >= #threshold {
            liblogger::log_warn!(
                &format!("QUEUE_ALERT: {} - High queue depth: {} (threshold: {}) | Queue: {} | {} | Processing: {:.1}/sec | Duration: {}ms", 
                    #fn_name, queue_depth, #threshold, #queue_name, formatted_queue_info, processing_rate, duration.as_millis()),
                None
            );
        } else {
            liblogger::log_info!(
                &format!("QUEUE_MONITOR: {} - Queue depth: {} | Queue: {} | {} | Processing: {:.1}/sec | Duration: {}ms", 
                    #fn_name, queue_depth, #queue_name, formatted_queue_info, processing_rate, duration.as_millis()),
                None
            );
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor garbage collection pressure and memory management
#[proc_macro_attribute]
pub fn log_gc_pressure(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let threshold = args.threshold.unwrap_or(100) as u64; // Convert to u64
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let gc_stats_before = get_gc_stats();
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let gc_stats_after = get_gc_stats();
        let formatted_gc_info = format_gc_info(&gc_stats_after);
        
        let gc_time_delta = gc_stats_after.total_gc_time_ms - gc_stats_before.total_gc_time_ms;
        let gc_collections_delta = gc_stats_after.gc_collections - gc_stats_before.gc_collections;
        
        if gc_time_delta >= #threshold {
            liblogger::log_warn!(
                &format!("GC_PRESSURE_ALERT: {} - High GC activity: {}ms GC time (threshold: {}ms) | {} | Collections: +{} | Duration: {}ms", 
                    #fn_name, gc_time_delta, #threshold, formatted_gc_info, gc_collections_delta, duration.as_millis()),
                None
            );
        } else {
            liblogger::log_info!(
                &format!("GC_MONITOR: {} - GC time: {}ms | {} | Collections: +{} | Duration: {}ms", 
                    #fn_name, gc_time_delta, formatted_gc_info, gc_collections_delta, duration.as_millis()),
                None
            );
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Implement anomaly detection for function behavior patterns
#[proc_macro_attribute]
pub fn log_anomaly_detection(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let service_name = args.service_name.unwrap_or_else(|| "default".to_string());
    let max_utilization = args.max_utilization.unwrap_or(90) as f64; // Convert to f64
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let anomaly_context_before = get_anomaly_detection_context(&#service_name, &#fn_name);
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let anomaly_context_after = get_anomaly_detection_context(&#service_name, &#fn_name);
        let formatted_anomaly_info = format_anomaly_detection_info(&anomaly_context_after);
        
        let anomaly_score = anomaly_context_after.anomaly_score;
        let baseline_duration_ms = anomaly_context_after.baseline_duration_ms;
        let resource_utilization = anomaly_context_after.resource_utilization_percentage;
        let pattern_deviation = anomaly_context_after.pattern_deviation_percentage;
        
        let duration_anomaly = if baseline_duration_ms > 0.0 {
            ((duration.as_millis() as f64 - baseline_duration_ms) / baseline_duration_ms) * 100.0
        } else {
            0.0
        };
        
        if anomaly_score > 0.8 || resource_utilization > #max_utilization || duration_anomaly > 200.0 {
            liblogger::log_warn!(
                &format!("ANOMALY_DETECTED: {} - Anomalous behavior detected | Service: {} | {} | Score: {:.2} | Duration anomaly: {:.1}% | Resource util: {:.1}% | Pattern deviation: {:.1}% | Duration: {}ms (baseline: {:.0}ms)", 
                    #fn_name, #service_name, formatted_anomaly_info, anomaly_score, duration_anomaly, resource_utilization, pattern_deviation, duration.as_millis(), baseline_duration_ms),
                None
            );
        } else if anomaly_score > 0.5 || resource_utilization > 70.0 {
            liblogger::log_info!(
                &format!("ANOMALY_WATCH: {} - Elevated anomaly metrics | Service: {} | {} | Score: {:.2} | Duration anomaly: {:.1}% | Resource util: {:.1}% | Pattern deviation: {:.1}% | Duration: {}ms (baseline: {:.0}ms)", 
                    #fn_name, #service_name, formatted_anomaly_info, anomaly_score, duration_anomaly, resource_utilization, pattern_deviation, duration.as_millis(), baseline_duration_ms),
                None
            );
        } else {
            liblogger::log_info!(
                &format!("ANOMALY_BASELINE: {} - Normal behavior pattern | Service: {} | {} | Score: {:.2} | Resource util: {:.1}% | Duration: {}ms", 
                    #fn_name, #service_name, formatted_anomaly_info, anomaly_score, resource_utilization, duration.as_millis()),
                None
            );
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor API rate limits
#[proc_macro_attribute]
pub fn log_api_rate_limits(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let service_name = args.service_name.unwrap_or_else(|| "default".to_string());
    let threshold = args.threshold.unwrap_or(90);
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let start_time = std::time::Instant::now();
        let result = #orig_block;
        let duration = start_time.elapsed();
        
        liblogger::log_info!(
            &format!("API_RATE_LIMITS: {} - Service: {} | Threshold: {}% | Duration: {}ms", 
                #fn_name, #service_name, #threshold, duration.as_millis()),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor SSL certificate expiry
#[proc_macro_attribute]
pub fn log_ssl_certificate_expiry(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let domain = args.domain.unwrap_or_else(|| "example.com".to_string());
    let days_warning = args.days_warning.unwrap_or(30);
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let start_time = std::time::Instant::now();
        let result = #orig_block;
        let duration = start_time.elapsed();
        
        liblogger::log_info!(
            &format!("SSL_CERTIFICATE_EXPIRY: {} - Domain: {} | Warning threshold: {} days | Duration: {}ms", 
                #fn_name, #domain, #days_warning, duration.as_millis()),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor service discovery
#[proc_macro_attribute]
pub fn log_service_discovery(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let service_name = args.service_name.unwrap_or_else(|| "default".to_string());
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let start_time = std::time::Instant::now();
        let result = #orig_block;
        let duration = start_time.elapsed();
        
        liblogger::log_info!(
            &format!("SERVICE_DISCOVERY: {} - Service: {} | Duration: {}ms", 
                #fn_name, #service_name, duration.as_millis()),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor load balancer health
#[proc_macro_attribute]
pub fn log_load_balancer_health(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let service_name = args.service_name.unwrap_or_else(|| "default".to_string());
    let threshold = args.threshold.unwrap_or(3);
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let start_time = std::time::Instant::now();
        let result = #orig_block;
        let duration = start_time.elapsed();
        
        liblogger::log_info!(
            &format!("LOAD_BALANCER_HEALTH: {} - Service: {} | Threshold: {} | Duration: {}ms", 
                #fn_name, #service_name, #threshold, duration.as_millis()),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor security events
#[proc_macro_attribute]
pub fn log_security_event(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let warning_level = args.warning_level.unwrap_or_else(|| "medium".to_string());
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let start_time = std::time::Instant::now();
        let result = #orig_block;
        let duration = start_time.elapsed();
        
        liblogger::log_warn!(
            &format!("SECURITY_EVENT: {} - Warning level: {} | Duration: {}ms", 
                #fn_name, #warning_level, duration.as_millis()),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor compliance checks
#[proc_macro_attribute]
pub fn log_compliance_check(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let domain = args.domain.unwrap_or_else(|| "default".to_string());
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let start_time = std::time::Instant::now();
        let result = #orig_block;
        let duration = start_time.elapsed();
        
        liblogger::log_info!(
            &format!("COMPLIANCE_CHECK: {} - Domain: {} | Duration: {}ms", 
                #fn_name, #domain, duration.as_millis()),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor access control
#[proc_macro_attribute]
pub fn log_access_control(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let domain = args.domain.unwrap_or_else(|| "default".to_string());
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let start_time = std::time::Instant::now();
        let result = #orig_block;
        let duration = start_time.elapsed();
        
        liblogger::log_info!(
            &format!("ACCESS_CONTROL: {} - Domain: {} | Duration: {}ms", 
                #fn_name, #domain, duration.as_millis()),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor crypto operations
#[proc_macro_attribute]
pub fn log_crypto_operation(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let domain = args.domain.unwrap_or_else(|| "default".to_string());
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let start_time = std::time::Instant::now();
        let result = #orig_block;
        let duration = start_time.elapsed();
        
        liblogger::log_info!(
            &format!("CRYPTO_OPERATION: {} - Domain: {} | Duration: {}ms", 
                #fn_name, #domain, duration.as_millis()),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor config changes
#[proc_macro_attribute]
pub fn log_config_change(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let domain = args.domain.unwrap_or_else(|| "default".to_string());
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let start_time = std::time::Instant::now();
        let result = #orig_block;
        let duration = start_time.elapsed();
        
        liblogger::log_info!(
            &format!("CONFIG_CHANGE: {} - Domain: {} | Duration: {}ms", 
                #fn_name, #domain, duration.as_millis()),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor deployments
#[proc_macro_attribute]
pub fn log_deployment(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let service_name = args.service_name.unwrap_or_else(|| "default".to_string());
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let start_time = std::time::Instant::now();
        let result = #orig_block;
        let duration = start_time.elapsed();
        
        liblogger::log_info!(
            &format!("DEPLOYMENT: {} - Service: {} | Duration: {}ms", 
                #fn_name, #service_name, duration.as_millis()),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor environment validation
#[proc_macro_attribute]
pub fn log_environment_validation(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let service_name = args.service_name.unwrap_or_else(|| "default".to_string());
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let start_time = std::time::Instant::now();
        let result = #orig_block;
        let duration = start_time.elapsed();
        
        liblogger::log_info!(
            &format!("ENVIRONMENT_VALIDATION: {} - Service: {} | Duration: {}ms", 
                #fn_name, #service_name, duration.as_millis()),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor feature flag changes
#[proc_macro_attribute]
pub fn log_feature_flag_change(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let min_percentage = args.min_percentage.unwrap_or(0);
    let max_percentage = args.max_percentage.unwrap_or(100);
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    
    input_fn.block = Box::new(parse_quote!({
        let start_time = std::time::Instant::now();
        let result = #orig_block;
        let duration = start_time.elapsed();
        
        liblogger::log_info!(
            &format!("FEATURE_FLAG_CHANGE: {} - Min: {}% | Max: {}% | Duration: {}ms", 
                #fn_name, #min_percentage, #max_percentage, duration.as_millis()),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor thread pool utilization and performance
#[proc_macro_attribute]
pub fn log_thread_pool_utilization(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let thread_pool_name = args.thread_pool_name.unwrap_or_else(|| "default".to_string());
    let threshold = args.threshold.unwrap_or(90);
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let pool_stats_before = get_thread_pool_stats(&#thread_pool_name);
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let pool_stats_after = get_thread_pool_stats(&#thread_pool_name);
        let formatted_pool_info = format_thread_pool_info(&pool_stats_after);
        
        let utilization = pool_stats_after.utilization_percentage;
        
        if utilization >= #threshold as f64 {
            liblogger::log_warn!(
                &format!("THREAD_POOL_ALERT: {} - High utilization: {:.1}% (threshold: {}%) | Pool: {} | {} | Duration: {}ms", 
                    #fn_name, utilization, #threshold, #thread_pool_name, formatted_pool_info, duration.as_millis()),
                None
            );
        } else {
            liblogger::log_info!(
                &format!("THREAD_POOL_MONITOR: {} - Utilization: {:.1}% | Pool: {} | {} | Duration: {}ms", 
                    #fn_name, utilization, #thread_pool_name, formatted_pool_info, duration.as_millis()),
                None
            );
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor business rule execution and validation
#[proc_macro_attribute]
pub fn log_business_rule(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let domain = args.domain.unwrap_or_else(|| "default".to_string());
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let rule_context = get_business_rule_context(&#domain, &#fn_name);
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let formatted_rule_info = format_business_rule_info(&rule_context);
        
        let rule_name = &rule_context.rule_name;
        let rule_version = &rule_context.rule_version;
        let execution_count = rule_context.execution_count;
        
        match &result {
            Ok(_) => {
                liblogger::log_info!(
                    &format!("BUSINESS_RULE_PASS: {} - Business rule validation passed | Domain: {} | Rule: {} | {} | Version: {} | Executions: {} | Duration: {}ms", 
                        #fn_name, #domain, rule_name, formatted_rule_info, rule_version, execution_count, duration.as_millis()),
                    None
                );
            },
            Err(_) => {
                liblogger::log_warn!(
                    &format!("BUSINESS_RULE_FAIL: {} - Business rule validation failed | Domain: {} | Rule: {} | {} | Version: {} | Executions: {} | Duration: {}ms", 
                        #fn_name, #domain, rule_name, formatted_rule_info, rule_version, execution_count, duration.as_millis()),
                    None
                );
            }
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor data quality checks and validation processes
#[proc_macro_attribute]
pub fn log_data_quality(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let domain = args.domain.unwrap_or_else(|| "default".to_string());
    let threshold = args.threshold.unwrap_or(95);
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let quality_metrics_before = get_data_quality_metrics(&#domain);
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let quality_metrics_after = get_data_quality_metrics(&#domain);
        let formatted_quality_info = format_data_quality_info(&quality_metrics_after);
        
        let quality_score = quality_metrics_after.quality_score_percentage;
        let records_processed = quality_metrics_after.records_processed;
        let validation_rules_passed = quality_metrics_after.validation_rules_passed;
        let total_validation_rules = quality_metrics_after.total_validation_rules;
        
        if quality_score < #threshold as f64 {
            liblogger::log_warn!(
                &format!("DATA_QUALITY_ALERT: {} - Low data quality score: {:.1}% (threshold: {}%) | Domain: {} | {} | Records: {} | Rules: {}/{} | Duration: {}ms", 
                    #fn_name, quality_score, #threshold, #domain, formatted_quality_info, records_processed, validation_rules_passed, total_validation_rules, duration.as_millis()),
                None
            );
        } else {
            liblogger::log_info!(
                &format!("DATA_QUALITY_OK: {} - Data quality score: {:.1}% | Domain: {} | {} | Records: {} | Rules: {}/{} | Duration: {}ms", 
                    #fn_name, quality_score, #domain, formatted_quality_info, records_processed, validation_rules_passed, total_validation_rules, duration.as_millis()),
                None
            );
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor workflow and process execution steps
#[proc_macro_attribute]
pub fn log_workflow_step(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let domain = args.domain.unwrap_or_else(|| "default".to_string());
    let max_depth = args.max_depth.unwrap_or(10);
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let workflow_state_before = get_workflow_state(&#domain, &#fn_name);
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let workflow_state_after = get_workflow_state(&#domain, &#fn_name);
        let formatted_workflow_info = format_workflow_info(&workflow_state_after);
        
        let workflow_id = &workflow_state_after.workflow_id;
        let step_name = &workflow_state_after.current_step;
        let step_depth = workflow_state_after.step_depth;
        let total_steps = workflow_state_after.total_steps;
        let completed_steps = workflow_state_after.completed_steps;
        
        if step_depth > #max_depth {
            liblogger::log_warn!(
                &format!("WORKFLOW_DEPTH_ALERT: {} - Workflow depth exceeded | Domain: {} | Workflow: {} | {} | Step: {} | Depth: {} (max: {}) | Progress: {}/{} | Duration: {}ms", 
                    #fn_name, #domain, workflow_id, formatted_workflow_info, step_name, step_depth, #max_depth, completed_steps, total_steps, duration.as_millis()),
                None
            );
        } else {
            match &result {
                Ok(_) => {
                    liblogger::log_info!(
                        &format!("WORKFLOW_STEP_SUCCESS: {} - Workflow step completed | Domain: {} | Workflow: {} | {} | Step: {} | Depth: {} | Progress: {}/{} | Duration: {}ms", 
                            #fn_name, #domain, workflow_id, formatted_workflow_info, step_name, step_depth, completed_steps, total_steps, duration.as_millis()),
                        None
                    );
                },
                Err(_) => {
                    liblogger::log_error!(
                        &format!("WORKFLOW_STEP_FAILURE: {} - Workflow step failed | Domain: {} | Workflow: {} | {} | Step: {} | Depth: {} | Progress: {}/{} | Duration: {}ms", 
                            #fn_name, #domain, workflow_id, formatted_workflow_info, step_name, step_depth, completed_steps, total_steps, duration.as_millis()),
                        None
                    );
                }
            }
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor transaction processing and state consistency
#[proc_macro_attribute]
pub fn log_transaction(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let domain = args.domain.unwrap_or_else(|| "default".to_string());
    let timeout_ms = args.timeout_ms.unwrap_or(5000);
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let tx_context = get_transaction_context(&#domain);
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let formatted_tx_info = format_transaction_info(&tx_context);
        
        let transaction_id = &tx_context.transaction_id;
        let isolation_level = &tx_context.isolation_level;
        let participant_count = tx_context.participant_count;
        
        if duration.as_millis() > #timeout_ms as u128 {
            liblogger::log_warn!(
                &format!("TRANSACTION_TIMEOUT_WARNING: {} - Transaction exceeded timeout | Domain: {} | Tx ID: {} | {} | Isolation: {} | Participants: {} | Duration: {}ms", 
                    #fn_name, #domain, transaction_id, formatted_tx_info, isolation_level, participant_count, duration.as_millis()),
                None
            );
        } else {
            match &result {
                Ok(_) => {
                    liblogger::log_info!(
                        &format!("TRANSACTION_SUCCESS: {} - Transaction completed successfully | Domain: {} | Tx ID: {} | {} | Isolation: {} | Participants: {} | Duration: {}ms", 
                            #fn_name, #domain, transaction_id, formatted_tx_info, isolation_level, participant_count, duration.as_millis()),
                        None
                    );
                },
                Err(_) => {
                    liblogger::log_error!(
                        &format!("TRANSACTION_FAILURE: {} - Transaction failed | Domain: {} | Tx ID: {} | {} | Isolation: {} | Participants: {} | Duration: {}ms", 
                            #fn_name, #domain, transaction_id, formatted_tx_info, isolation_level, participant_count, duration.as_millis()),
                        None
                    );
                }
            }
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor inter-service communication and RPC calls
#[proc_macro_attribute]
pub fn log_service_communication(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let service_name = args.service_name.unwrap_or_else(|| "unknown".to_string());
    let timeout_ms = args.timeout_ms.unwrap_or(5000);
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let comm_context = get_service_communication_context(&#service_name);
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let formatted_comm_info = format_service_communication_info(&comm_context);
        
        let target_service = &comm_context.target_service;
        let protocol = &comm_context.protocol;
        let circuit_breaker_state = &comm_context.circuit_breaker_state;
        
        if duration.as_millis() > #timeout_ms as u128 {
            liblogger::log_warn!(
                &format!("SERVICE_COMM_TIMEOUT: {} - Service communication timeout | Target: {} | {} | Protocol: {} | Circuit Breaker: {} | Duration: {}ms (timeout: {}ms)", 
                    #fn_name, target_service, formatted_comm_info, protocol, circuit_breaker_state, duration.as_millis(), #timeout_ms),
                None
            );
        } else {
            match &result {
                Ok(_) => {
                    liblogger::log_info!(
                        &format!("SERVICE_COMM_SUCCESS: {} - Service communication successful | Target: {} | {} | Protocol: {} | Circuit Breaker: {} | Duration: {}ms", 
                            #fn_name, target_service, formatted_comm_info, protocol, circuit_breaker_state, duration.as_millis()),
                        None
                    );
                },
                Err(_) => {
                    liblogger::log_error!(
                        &format!("SERVICE_COMM_FAILURE: {} - Service communication failed | Target: {} | {} | Protocol: {} | Circuit Breaker: {} | Duration: {}ms", 
                            #fn_name, target_service, formatted_comm_info, protocol, circuit_breaker_state, duration.as_millis()),
                        None
                    );
                }
            }
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor consensus algorithm operations and cluster decisions
#[proc_macro_attribute]
pub fn log_consensus_operation(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let domain = args.domain.unwrap_or_else(|| "default".to_string());
    let timeout_ms = args.timeout_ms.unwrap_or(10000);
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let consensus_context = get_consensus_context(&#domain);
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let formatted_consensus_info = format_consensus_info(&consensus_context);
        
        let term = consensus_context.term;
        let leader_id = &consensus_context.leader_id;
        let node_count = consensus_context.node_count;
        let votes_received = consensus_context.votes_received;
        
        if duration.as_millis() > #timeout_ms as u128 {
            liblogger::log_warn!(
                &format!("CONSENSUS_TIMEOUT: {} - Consensus operation timeout | Domain: {} | {} | Term: {} | Leader: {} | Votes: {}/{} | Duration: {}ms (timeout: {}ms)", 
                    #fn_name, #domain, formatted_consensus_info, term, leader_id, votes_received, node_count, duration.as_millis(), #timeout_ms),
                None
            );
        } else {
            match &result {
                Ok(_) => {
                    liblogger::log_info!(
                        &format!("CONSENSUS_SUCCESS: {} - Consensus achieved | Domain: {} | {} | Term: {} | Leader: {} | Votes: {}/{} | Duration: {}ms", 
                            #fn_name, #domain, formatted_consensus_info, term, leader_id, votes_received, node_count, duration.as_millis()),
                        None
                    );
                },
                Err(_) => {
                    liblogger::log_warn!(
                        &format!("CONSENSUS_FAILURE: {} - Consensus failed | Domain: {} | {} | Term: {} | Leader: {} | Votes: {}/{} | Duration: {}ms", 
                            #fn_name, #domain, formatted_consensus_info, term, leader_id, votes_received, node_count, duration.as_millis()),
                        None
                    );
                }
            }
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor cluster health and node membership changes
#[proc_macro_attribute]
pub fn log_cluster_health(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let domain = args.domain.unwrap_or_else(|| "default".to_string());
    let threshold = args.threshold.unwrap_or(70);
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let cluster_health_before = get_cluster_health_stats(&#domain);
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let cluster_health_after = get_cluster_health_stats(&#domain);
        let formatted_cluster_info = format_cluster_health_info(&cluster_health_after);
        
        let health_percentage = cluster_health_after.health_percentage;
        let healthy_nodes = cluster_health_after.healthy_nodes;
        let total_nodes = cluster_health_after.total_nodes;
        let leader_node = &cluster_health_after.leader_node;
        
        if health_percentage < #threshold as f64 {
            liblogger::log_error!(
                &format!("CLUSTER_HEALTH_CRITICAL: {} - Cluster health critical: {:.1}% (threshold: {}%) | Domain: {} | {} | Healthy: {}/{} | Leader: {} | Duration: {}ms", 
                    #fn_name, health_percentage, #threshold, #domain, formatted_cluster_info, healthy_nodes, total_nodes, leader_node, duration.as_millis()),
                None
            );
        } else if health_percentage < 90.0 {
            liblogger::log_warn!(
                &format!("CLUSTER_HEALTH_DEGRADED: {} - Cluster health degraded: {:.1}% | Domain: {} | {} | Healthy: {}/{} | Leader: {} | Duration: {}ms", 
                    #fn_name, health_percentage, #domain, formatted_cluster_info, healthy_nodes, total_nodes, leader_node, duration.as_millis()),
                None
            );
        } else {
            liblogger::log_info!(
                &format!("CLUSTER_HEALTH_OK: {} - Cluster health good: {:.1}% | Domain: {} | {} | Healthy: {}/{} | Leader: {} | Duration: {}ms", 
                    #fn_name, health_percentage, #domain, formatted_cluster_info, healthy_nodes, total_nodes, leader_node, duration.as_millis()),
                None
            );
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor distributed lock operations and resource coordination
#[proc_macro_attribute]
pub fn log_distributed_lock(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let domain = args.domain.unwrap_or_else(|| "default".to_string());
    let timeout_ms = args.timeout_ms.unwrap_or(30000);
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let lock_context = get_distributed_lock_context(&#domain, &#fn_name);
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let formatted_lock_info = format_distributed_lock_info(&lock_context);
        
        let lock_id = &lock_context.lock_id;
        let holder_node = &lock_context.holder_node;
        let lock_type = &lock_context.lock_type;
        let wait_queue_size = lock_context.wait_queue_size;
        
        if duration.as_millis() > #timeout_ms as u128 {
            liblogger::log_warn!(
                &format!("DISTRIBUTED_LOCK_TIMEOUT: {} - Lock operation timeout | Domain: {} | Lock ID: {} | {} | Holder: {} | Type: {} | Queue: {} | Duration: {}ms (timeout: {}ms)", 
                    #fn_name, #domain, lock_id, formatted_lock_info, holder_node, lock_type, wait_queue_size, duration.as_millis(), #timeout_ms),
                None
            );
        } else {
            match &result {
                Ok(_) => {
                    liblogger::log_info!(
                        &format!("DISTRIBUTED_LOCK_SUCCESS: {} - Lock operation successful | Domain: {} | Lock ID: {} | {} | Holder: {} | Type: {} | Queue: {} | Duration: {}ms", 
                            #fn_name, #domain, lock_id, formatted_lock_info, holder_node, lock_type, wait_queue_size, duration.as_millis()),
                        None
                    );
                },
                Err(_) => {
                    liblogger::log_warn!(
                        &format!("DISTRIBUTED_LOCK_FAILURE: {} - Lock operation failed | Domain: {} | Lock ID: {} | {} | Holder: {} | Type: {} | Queue: {} | Duration: {}ms", 
                            #fn_name, #domain, lock_id, formatted_lock_info, holder_node, lock_type, wait_queue_size, duration.as_millis()),
                        None
                    );
                }
            }
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Implement distributed tracing with correlation IDs
#[proc_macro_attribute]
pub fn log_trace_correlation(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let service_name = args.service_name.unwrap_or_else(|| "unknown".to_string());
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let trace_context = get_trace_context(&#service_name, &#fn_name);
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let formatted_trace_info = format_trace_info(&trace_context);
        
        let trace_id = &trace_context.trace_id;
        let span_id = &trace_context.span_id;
        let parent_span_id = &trace_context.parent_span_id;
        let baggage = &trace_context.baggage;
        
        match &result {
            Ok(_) => {
                liblogger::log_info!(
                    &format!("TRACE_SPAN_SUCCESS: {} - Span completed successfully | Service: {} | {} | Trace: {} | Span: {} | Parent: {} | Baggage: {} | Duration: {}ms", 
                        #fn_name, #service_name, formatted_trace_info, trace_id, span_id, parent_span_id, baggage, duration.as_millis()),
                    None
                );
            },
            Err(_) => {
                liblogger::log_error!(
                    &format!("TRACE_SPAN_ERROR: {} - Span completed with error | Service: {} | {} | Trace: {} | Span: {} | Parent: {} | Baggage: {} | Duration: {}ms", 
                        #fn_name, #service_name, formatted_trace_info, trace_id, span_id, parent_span_id, baggage, duration.as_millis()),
                    None
                );
            }
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Collect custom metrics and dimensional data
#[proc_macro_attribute]
pub fn log_custom_metrics(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let metric_name = args.metric_name.unwrap_or_else(|| "custom_metric".to_string());
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let metrics_context_before = get_custom_metrics_context(&#metric_name);
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let metrics_context_after = get_custom_metrics_context(&#metric_name);
        let formatted_metrics_info = format_custom_metrics_info(&metrics_context_after);
        
        let metric_value = metrics_context_after.metric_value;
        let dimensions = &metrics_context_after.dimensions;
        let metric_type = &metrics_context_after.metric_type;
        let tags = &metrics_context_after.tags;
        
        let value_delta = metric_value - metrics_context_before.metric_value;
        
        liblogger::log_info!(
            &format!("CUSTOM_METRICS: {} - Metric collected | Metric: {} | {} | Value: {:.2} (Δ{:.2}) | Type: {} | Dimensions: {} | Tags: {} | Duration: {}ms", 
                #fn_name, #metric_name, formatted_metrics_info, metric_value, value_delta, metric_type, dimensions, tags, duration.as_millis()),
            None
        );
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}

/// Monitor system health with multiple checkpoints
#[proc_macro_attribute]
pub fn log_health_check(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as MacroArgs);
    let service_name = args.service_name.unwrap_or_else(|| "default".to_string());
    let threshold = args.threshold.unwrap_or(95);
    let mut input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = get_fn_name(&input_fn);
    let orig_block = input_fn.block.clone();
    let utility_functions = generate_utility_functions();

    input_fn.block = Box::new(parse_quote!({
        #utility_functions
        
        let start_time = std::time::Instant::now();
        let health_context = get_health_check_context(&#service_name);
        
        let result = #orig_block;
        
        let duration = start_time.elapsed();
        let formatted_health_info = format_health_check_info(&health_context);
        
        let overall_health = health_context.overall_health_percentage;
        let checks_passed = health_context.checks_passed;
        let total_checks = health_context.total_checks;
        let failed_checks = &health_context.failed_checks;
        
        if overall_health < #threshold as f64 {
            liblogger::log_error!(
                &format!("HEALTH_CHECK_CRITICAL: {} - Health check failed | Service: {} | {} | Health: {:.1}% (threshold: {}%) | Passed: {}/{} | Failed: {:?} | Duration: {}ms", 
                    #fn_name, #service_name, formatted_health_info, overall_health, #threshold, checks_passed, total_checks, failed_checks, duration.as_millis()),
                None
            );
        } else if overall_health < 90.0 {
            liblogger::log_warn!(
                &format!("HEALTH_CHECK_DEGRADED: {} - Health check degraded | Service: {} | {} | Health: {:.1}% | Passed: {}/{} | Failed: {:?} | Duration: {}ms", 
                    #fn_name, #service_name, formatted_health_info, overall_health, checks_passed, total_checks, failed_checks, duration.as_millis()),
                None
            );
        } else {
            liblogger::log_info!(
                &format!("HEALTH_CHECK_OK: {} - Health check passed | Service: {} | {} | Health: {:.1}% | Passed: {}/{} | Duration: {}ms", 
                    #fn_name, #service_name, formatted_health_info, overall_health, checks_passed, total_checks, duration.as_millis()),
                None
            );
        }
        
        result
    }));
    
    TokenStream::from(quote!(#input_fn))
}
