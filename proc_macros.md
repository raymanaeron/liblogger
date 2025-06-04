# LibLogger Procedural Macros

This document describes all the procedural macros available in the `liblogger_macros` crate. These macros provide automatic instrumentation for functions with various logging, monitoring, and observability features.

## Table of Contents

- [Setup](#setup)
- [Basic Logging Macros](#basic-logging-macros)
- [Performance & Monitoring Macros](#performance--monitoring-macros)
- [Error Handling & Resilience Macros](#error-handling--resilience-macros)
- [DevOps Infrastructure Macros](#devops-infrastructure-macros)
- [Distributed Systems Macros](#distributed-systems-macros)
- [Advanced Analytics Macros](#advanced-analytics-macros)
- [Security & Compliance Macros](#security--compliance-macros)
- [Business Logic Macros](#business-logic-macros)

## Setup

Before using any attribute macros, you must call the initialization macro at the module level:

```rust
use liblogger_macros::*;
use liblogger;

// Required initialization - call this once per module
initialize_logger_attributes!();

#[log_entry_exit]
fn my_function() {
    // Your code here
}
```

## Basic Logging Macros

### `#[log_entry_exit]`
Automatically logs function entry and exit points.

```rust
#[log_entry_exit]
fn process_data() {
    // Automatically logs: "ENTRY: process_data"
    // ... your code ...
    // Automatically logs: "EXIT: process_data"
}
```

### `#[log_args(arg1, arg2)]`
Logs specified function arguments at entry.

```rust
#[log_args(user_id, operation)]
fn handle_request(user_id: u64, operation: &str, data: Vec<u8>) {
    // Logs: "Entering handle_request with args: user_id = 123, operation = create"
}
```

### `#[log_response]`
Logs the return value of a function.

```rust
#[log_response]
fn calculate() -> i32 {
    42 // Logs: "calculate returned: 42"
}
```

### `#[log_result(success_level = "info", error_level = "error")]`
Logs function results with different log levels for success/error cases.

```rust
#[log_result(success_level = "debug", error_level = "warn")]
fn risky_operation() -> Result<String, Error> {
    // Success: debug level, Error: warn level
}
```

## Performance & Monitoring Macros

### `#[measure_time]`
Measures and logs function execution time.

```rust
#[measure_time]
fn expensive_computation() {
    // Logs: "expensive_computation completed in 150 ms"
}
```

### `#[log_memory_usage]`
Monitors memory usage during function execution (requires `psutil`).

```rust
#[log_memory_usage]
fn memory_intensive_task() {
    // Logs memory before and after execution
}
```

### `#[log_cpu_time]`
Logs CPU time usage (currently measures wall time as approximation).

```rust
#[log_cpu_time]
fn cpu_intensive_task() {
    // Logs: "cpu_intensive_task used CPU time: approx 200 ms (wall time)"
}
```

### `#[log_concurrency]`
Tracks concurrent invocations of a function.

```rust
#[log_concurrency]
fn shared_resource_handler() {
    // Logs current concurrency level on entry and exit
}
```

### `#[dependency_latency(target = "database")]`
Measures latency to external dependencies.

```rust
#[dependency_latency(target = "redis")]
fn cache_lookup() -> Result<String, Error> {
    // Logs dependency call timing and success/failure
}
```

## Error Handling & Resilience Macros

### `#[log_errors]`
Automatically logs errors and panics.

```rust
#[log_errors]
fn fallible_operation() -> Result<(), Error> {
    // Automatically logs any errors returned or panics caught
}
```

### `#[log_retries(max_attempts = 3)]`
Implements retry logic with automatic logging.

```rust
#[log_retries(max_attempts = 5)]
fn unreliable_network_call() -> Result<Data, NetworkError> {
    // Automatically retries up to 5 times with exponential backoff
    // Logs each attempt and final outcome
}
```

### `#[circuit_breaker(failure_threshold = 5)]`
Implements circuit breaker pattern with logging.

```rust
#[circuit_breaker(failure_threshold = 3)]
fn external_service_call() -> Result<Response, Error> {
    // Opens circuit after 3 failures, logs circuit state changes
}
```

### `#[catch_panic]`
Catches panics and converts them to errors or default values.

```rust
#[catch_panic]
fn potentially_panicking_function() -> Result<String, Box<dyn std::error::Error>> {
    // Catches panics and converts to Result::Err
}
```

### `#[health_check]`
Logs health check results with timing.

```rust
#[health_check]
fn database_health() -> Result<(), HealthError> {
    // Logs health check success/failure with timing
}
```

## DevOps Infrastructure Macros

### `#[log_disk_usage(threshold = 85)]`
Monitors disk usage and alerts on threshold breaches.

```rust
#[log_disk_usage(threshold = 90)]
fn file_processing_task() {
    // Monitors disk usage, alerts if >90% full
}
```

### `#[log_network_connectivity(endpoint = "8.8.8.8:53")]`
Monitors network connectivity to specified endpoints.

```rust
#[log_network_connectivity(endpoint = "api.service.com:443")]
fn network_dependent_operation() {
    // Checks connectivity before/after operation
}
```

### `#[log_database_pool(pool_name = "primary", threshold = 80)]`
Monitors database connection pool health.

```rust
#[log_database_pool(pool_name = "user_db", threshold = 75)]
fn database_operation() {
    // Monitors connection pool utilization
}
```

### `#[log_file_descriptors(threshold = 1000)]`
Monitors file descriptor usage to detect resource leaks.

```rust
#[log_file_descriptors(threshold = 800)]
fn file_intensive_operation() {
    // Tracks file descriptor count changes
}
```

### `#[log_cache_hit_ratio(cache_name = "redis", threshold = 70)]`
Monitors cache performance metrics.

```rust
#[log_cache_hit_ratio(cache_name = "session_cache", threshold = 85)]
fn cached_data_access() {
    // Monitors cache hit ratios and performance
}
```

### `#[log_queue_depth(queue_name = "tasks", threshold = 500)]`
Monitors message queue depth and processing rates.

```rust
#[log_queue_depth(queue_name = "email_queue", threshold = 1000)]
fn queue_processor() {
    // Monitors queue depth and processing performance
}
```

### `#[log_gc_pressure(threshold = 150)]`
Monitors garbage collection pressure (for GC-enabled environments).

```rust
#[log_gc_pressure(threshold = 100)]
fn memory_allocating_function() {
    // Monitors GC activity during execution
}
```

### `#[log_thread_pool_utilization(thread_pool_name = "workers", threshold = 85)]`
Monitors thread pool utilization and performance.

```rust
#[log_thread_pool_utilization(thread_pool_name = "http_workers", threshold = 90)]
fn concurrent_task() {
    // Monitors thread pool utilization
}
```

## Distributed Systems Macros

### `#[log_transaction(domain = "payment", timeout_ms = 5000)]`
Monitors transaction processing with timeout warnings.

```rust
#[log_transaction(domain = "order_processing", timeout_ms = 3000)]
fn process_payment() -> Result<Receipt, PaymentError> {
    // Monitors transaction state and timing
}
```

### `#[log_service_communication(service_name = "user_service", timeout_ms = 2000)]`
Monitors inter-service communication and RPC calls.

```rust
#[log_service_communication(service_name = "inventory_service", timeout_ms = 1500)]
fn check_stock() -> Result<StockLevel, ServiceError> {
    // Monitors service-to-service communication
}
```

### `#[log_consensus_operation(domain = "cluster", timeout_ms = 10000)]`
Monitors consensus algorithm operations in distributed systems.

```rust
#[log_consensus_operation(domain = "raft", timeout_ms = 5000)]
fn elect_leader() -> Result<LeaderInfo, ConsensusError> {
    // Monitors consensus protocol operations
}
```

### `#[log_cluster_health(domain = "kubernetes", threshold = 80)]`
Monitors cluster health and node membership.

```rust
#[log_cluster_health(domain = "docker_swarm", threshold = 70)]
fn cluster_operation() {
    // Monitors overall cluster health metrics
}
```

### `#[log_distributed_lock(domain = "resource_lock", timeout_ms = 30000)]`
Monitors distributed lock operations and coordination.

```rust
#[log_distributed_lock(domain = "file_processing", timeout_ms = 15000)]
fn exclusive_operation() -> Result<(), LockError> {
    // Monitors distributed lock acquisition and release
}
```

### `#[log_trace_correlation(service_name = "api_gateway")]`
Implements distributed tracing with correlation IDs.

```rust
#[log_trace_correlation(service_name = "order_service")]
fn handle_order() {
    // Adds distributed tracing context to logs
}
```

## Advanced Analytics Macros

### `#[log_anomaly_detection(service_name = "api", max_utilization = 85)]`
Implements anomaly detection for function behavior patterns.

```rust
#[log_anomaly_detection(service_name = "recommendation_engine", max_utilization = 90)]
fn generate_recommendations() {
    // Monitors for anomalous behavior patterns
}
```

### `#[log_custom_metrics(metric_name = "business_kpi")]`
Collects custom metrics and dimensional data.

```rust
#[log_custom_metrics(metric_name = "conversion_rate")]
fn track_conversion() {
    // Collects custom business metrics
}
```

### `#[metrics_counter(counter_name = "api_calls")]`
Increments Prometheus metrics counters (requires `prometheus` crate).

```rust
#[metrics_counter(counter_name = "user_registrations")]
fn register_user() {
    // Increments Prometheus counter
}
```

### `#[log_health_check(service_name = "api", threshold = 95)]`
Comprehensive health monitoring with multiple checkpoints.

```rust
#[log_health_check(service_name = "payment_service", threshold = 99)]
fn comprehensive_health_check() -> Result<HealthStatus, HealthError> {
    // Monitors multiple health indicators
}
```

## Security & Compliance Macros

### `#[log_security_event(warning_level = "high")]`
Logs security-related events and violations.

```rust
#[log_security_event(warning_level = "critical")]
fn privileged_operation() {
    // Logs security events for audit trails
}
```

### `#[log_compliance_check(domain = "gdpr")]`
Monitors compliance-related operations.

```rust
#[log_compliance_check(domain = "pci_dss")]
fn process_payment_data() {
    // Logs compliance-related activities
}
```

### `#[log_access_control(domain = "admin_panel")]`
Monitors access control and authorization events.

```rust
#[log_access_control(domain = "user_management")]
fn modify_user_permissions() {
    // Logs access control decisions
}
```

### `#[log_crypto_operation(domain = "encryption")]`
Monitors cryptographic operations for security auditing.

```rust
#[log_crypto_operation(domain = "key_generation")]
fn generate_encryption_key() {
    // Logs cryptographic operations
}
```

### `#[audit_log]`
Creates detailed audit trails with user context.

```rust
#[audit_log]
fn sensitive_operation() {
    // Creates comprehensive audit logs with user context
}
```

## Business Logic Macros

### `#[log_business_rule(domain = "order_processing")]`
Monitors business rule execution and validation.

```rust
#[log_business_rule(domain = "pricing")]
fn apply_discount_rules() -> Result<Price, BusinessRuleError> {
    // Monitors business rule execution and compliance
}
```

### `#[log_data_quality(domain = "customer_data", threshold = 98)]`
Monitors data quality checks and validation processes.

```rust
#[log_data_quality(domain = "product_catalog", threshold = 95)]
fn validate_product_data() -> Result<ValidationReport, DataError> {
    // Monitors data quality metrics and validation results
}
```

### `#[log_workflow_step(domain = "payment_flow", max_depth = 5)]`
Monitors workflow and process execution steps.

```rust
#[log_workflow_step(domain = "order_fulfillment", max_depth = 10)]
fn process_order_step() -> Result<StepResult, WorkflowError> {
    // Monitors workflow execution and step progression
}
```

## Configuration & Infrastructure Macros

### `#[log_config_change(domain = "app_config")]`
Monitors configuration changes and updates.

```rust
#[log_config_change(domain = "feature_flags")]
fn update_configuration() {
    // Logs configuration changes for audit trails
}
```

### `#[log_deployment(service_name = "web_service")]`
Monitors deployment processes and changes.

```rust
#[log_deployment(service_name = "api_service")]
fn deploy_new_version() {
    // Logs deployment activities and status
}
```

### `#[log_environment_validation(service_name = "api")]`
Monitors environment validation and health checks.

```rust
#[log_environment_validation(service_name = "database")]
fn validate_environment() {
    // Validates environment configuration and dependencies
}
```

### `#[log_feature_flag_change(min_percentage = 10, max_percentage = 90)]`
Monitors feature flag changes and rollout percentages.

```rust
#[log_feature_flag_change(min_percentage = 0, max_percentage = 100)]
fn toggle_feature() {
    // Monitors feature flag state changes
}
```

### `#[log_api_rate_limits(service_name = "external_api", threshold = 90)]`
Monitors API rate limiting and usage patterns.

```rust
#[log_api_rate_limits(service_name = "payment_gateway", threshold = 80)]
fn api_call() {
    // Monitors API rate limit consumption
}
```

### `#[log_ssl_certificate_expiry(domain = "api.example.com", days_warning = 30)]`
Monitors SSL certificate expiration and renewal needs.

```rust
#[log_ssl_certificate_expiry(domain = "secure.myapp.com", days_warning = 60)]
fn check_certificates() {
    // Monitors SSL certificate expiration dates
}
```

### `#[log_service_discovery(service_name = "user_service")]`
Monitors service discovery and registration processes.

```rust
#[log_service_discovery(service_name = "notification_service")]
fn discover_services() {
    // Monitors service discovery operations
}
```

### `#[log_load_balancer_health(service_name = "api_lb", threshold = 3)]`
Monitors load balancer health and backend availability.

```rust
#[log_load_balancer_health(service_name = "web_lb", threshold = 2)]
fn check_load_balancer() {
    // Monitors load balancer health and backend status
}
```

## Utility & Context Macros

### `#[trace_span]`
Creates distributed tracing spans with UUID generation (requires `uuid` crate).

```rust
#[trace_span]
fn traced_operation() {
    // Creates trace spans for request flow tracking
}
```

### `#[feature_flag(flag_name = "new_algorithm")]`
Logs feature flag state during execution.

```rust
#[feature_flag(flag_name = "experimental_feature")]
fn conditional_feature() {
    // Logs feature flag state for debugging
}
```

### `#[request_context]`
Attaches request context (user_id, session_id, etc.) to logs.

```rust
#[request_context]
fn handle_user_request() {
    // Adds request context to all log messages
}
```

### `#[version_tag]`
Includes version information in logs.

```rust
#[version_tag]
fn versioned_operation() {
    // Includes build version in log messages
}
```

### `#[throttle_log(rate = 5)]`
Throttles log output to prevent flooding during incidents.

```rust
#[throttle_log(rate = 10)]
fn high_frequency_operation() {
    // Limits logging to 10 messages per minute
}
```

## Best Practices

1. **Always initialize**: Call `initialize_logger_attributes!()` before using any attribute macros
2. **Combine macros**: Multiple macros can be applied to the same function
3. **Choose appropriate thresholds**: Set realistic thresholds for alerting macros
4. **Monitor performance impact**: Some macros add overhead - use judiciously in hot paths
5. **Use structured logging**: Take advantage of the context fields for better log analysis
6. **Configure log levels**: Ensure your logging framework is configured to handle the log levels used

## Error Handling

All macros are designed to be non-intrusive. If logging fails, the original function execution continues normally. Macro-generated code includes error handling to prevent logging issues from affecting application functionality.

## Dependencies

Some macros require additional dependencies:
- `#[metrics_counter]`: Requires `prometheus` crate
- `#[log_memory_usage]`: Requires `psutil` crate  
- `#[trace_span]`: Requires `uuid` crate

Make sure to add these to your `Cargo.toml` when using the corresponding macros.
