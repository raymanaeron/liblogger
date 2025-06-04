# LibLogger - Advanced Rust Logging Framework

LibLogger is a comprehensive logging framework for Rust applications that provides both traditional logging capabilities and advanced procedural macros for automatic instrumentation, monitoring, and observability.

## Features

### Core Logging
- **Multiple log levels**: DEBUG, INFO, WARN, ERROR
- **Structured logging**: Support for additional context fields
- **Flexible output**: Console, file, and custom sinks
- **High performance**: Minimal overhead with lazy evaluation
- **Thread-safe**: Safe for use in concurrent applications

### Procedural Macros (50+ Available)
- **Basic Instrumentation**: Entry/exit logging, argument logging, response logging
- **Performance Monitoring**: Execution timing, memory usage, CPU time tracking
- **Error Handling**: Automatic error logging, retry logic, circuit breakers
- **DevOps Infrastructure**: Disk usage, network connectivity, database pools, file descriptors
- **Distributed Systems**: Transaction monitoring, service communication, consensus operations
- **Security & Compliance**: Security events, access control, crypto operations, audit trails
- **Business Logic**: Business rule validation, data quality checks, workflow monitoring
- **Advanced Analytics**: Anomaly detection, custom metrics, health monitoring

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
liblogger = "0.1.0"
liblogger_macros = "0.1.0"

# Optional dependencies for specific macros
prometheus = "0.13"  # For metrics_counter macro
psutil = "3.2"       # For memory usage monitoring
uuid = "1.0"         # For distributed tracing
```

### Basic Logging

First, initialize the logger, then use the core logging macros:

```rust
use liblogger::*;

fn main() {
    // Initialize the logger
    Logger::init(); // or Logger::init_with_config_file("config.toml")
    
    // Core logging macros - available immediately after initialization
    log_debug!("Debug message for development");
    log_info!("Application started successfully"); 
    log_warn!("This is a warning message");
    log_error!("Error occurred during processing");
    
    // Logging with context
    log_info!(
        "Processing user request", 
        Some("user_id=123,action=login".to_string())
    );
    
    log_warn!(
        "High memory usage detected", 
        Some("memory_usage=85%,threshold=80%".to_string())
    );
    
    log_error!(
        "Database connection failed", 
        Some("host=localhost,port=5432,retry_count=3".to_string())
    );
}
```

### Procedural Macros

For advanced instrumentation, add the procedural macros:

```rust
use liblogger::*;
use liblogger_macros::*;

// Required initialization for procedural macros
initialize_logger_attributes!();

// Basic function instrumentation
#[log_entry_exit]
#[measure_time]
fn process_user_data(user_id: u64) {
    // Function automatically logs entry, exit, and execution time
    log_info!(&format!("Processing data for user {}", user_id));
}

// Advanced monitoring with multiple macros
#[log_disk_usage(threshold = 85)]
#[log_memory_usage]
#[log_retries(max_attempts = 3)]
#[audit_log]
async fn critical_operation() -> Result<(), Error> {
    // Monitors disk usage, memory, implements retries, and creates audit logs
    log_info!("Executing critical operation");
    Ok(())
}

// Distributed systems monitoring
#[log_service_communication(service_name = "user_service", timeout_ms = 2000)]
#[log_trace_correlation(service_name = "api_gateway")]
#[circuit_breaker(failure_threshold = 5)]
async fn external_api_call() -> Result<Response, ApiError> {
    // Monitors service communication, adds tracing, implements circuit breaker
    log_info!("Making external API call");
    Ok(Response::default())
}

// Business logic monitoring
#[log_business_rule(domain = "pricing")]
#[log_data_quality(domain = "product_data", threshold = 95)]
fn calculate_price(product: &Product) -> Result<Price, BusinessError> {
    // Monitors business rule execution and data quality
    log_info!("Calculating price for product");
    Ok(Price::default())
}
```

## Core Logging Macros

### Available Immediately After Logger Initialization

```rust
// Basic logging - no setup required beyond Logger::init()
log_debug!("Debug information for developers");
log_info!("General information about application operation");
log_warn!("Warning about potential issues");
log_error!("Error conditions that should be investigated");

// With context (optional second parameter)
log_info!("User login", Some("user_id=123,ip=192.168.1.1".to_string()));
log_error!("Database error", Some("table=users,operation=insert,error_code=23505".to_string()));
```

### Log Levels

- **DEBUG**: Detailed information for diagnosing problems
- **INFO**: General information about application operation
- **WARN**: Warning messages for potentially harmful situations
- **ERROR**: Error conditions that should be investigated

## Comprehensive Macro Categories

### Basic Instrumentation
- `#[log_entry_exit]` - Function entry/exit logging
- `#[log_args(arg1, arg2)]` - Argument logging
- `#[log_response]` - Return value logging
- `#[measure_time]` - Execution timing

### Performance & Monitoring  
- `#[log_memory_usage]` - Memory usage tracking
- `#[log_cpu_time]` - CPU time monitoring
- `#[log_concurrency]` - Concurrent execution tracking
- `#[dependency_latency(target = "db")]` - External dependency timing

### Error Handling & Resilience
- `#[log_errors]` - Automatic error logging
- `#[log_retries(max_attempts = 3)]` - Retry logic with logging
- `#[circuit_breaker(failure_threshold = 5)]` - Circuit breaker pattern
- `#[catch_panic]` - Panic recovery and logging

### DevOps Infrastructure (15+ macros)
- `#[log_disk_usage(threshold = 85)]` - Disk space monitoring
- `#[log_network_connectivity(endpoint = "api.com")]` - Network health
- `#[log_database_pool(pool_name = "primary")]` - Connection pool monitoring
- `#[log_cache_hit_ratio(cache_name = "redis")]` - Cache performance
- `#[log_queue_depth(queue_name = "tasks")]` - Message queue monitoring
- `#[log_file_descriptors(threshold = 1000)]` - Resource leak detection

### Distributed Systems (10+ macros)
- `#[log_transaction(domain = "payment")]` - Transaction monitoring
- `#[log_service_communication(service_name = "api")]` - Inter-service calls
- `#[log_consensus_operation(domain = "raft")]` - Consensus algorithm monitoring
- `#[log_cluster_health(domain = "k8s")]` - Cluster health monitoring
- `#[log_distributed_lock(domain = "resources")]` - Distributed locking
- `#[log_trace_correlation(service_name = "gateway")]` - Distributed tracing

### Security & Compliance (8+ macros)
- `#[log_security_event(warning_level = "high")]` - Security event logging
- `#[log_access_control(domain = "admin")]` - Access control monitoring
- `#[log_compliance_check(domain = "gdpr")]` - Compliance validation
- `#[log_crypto_operation(domain = "encryption")]` - Crypto operation auditing
- `#[audit_log]` - Comprehensive audit trails

### Business Logic (5+ macros)
- `#[log_business_rule(domain = "pricing")]` - Business rule monitoring
- `#[log_data_quality(domain = "customer_data")]` - Data quality checks
- `#[log_workflow_step(domain = "order_flow")]` - Workflow monitoring

### Advanced Analytics (5+ macros)
- `#[log_anomaly_detection(service_name = "api")]` - Anomaly detection
- `#[log_custom_metrics(metric_name = "kpi")]` - Custom metrics collection
- `#[log_health_check(service_name = "api")]` - Health monitoring
- `#[metrics_counter(counter_name = "requests")]` - Prometheus integration

## Configuration Examples

### Production Configuration
```rust
use liblogger_macros::*;

initialize_logger_attributes!();

// High-traffic API endpoint with comprehensive monitoring
#[log_entry_exit]
#[measure_time]
#[log_api_rate_limits(service_name = "public_api", threshold = 90)]
#[log_anomaly_detection(service_name = "api", max_utilization = 85)]
#[throttle_log(rate = 100)]
#[circuit_breaker(failure_threshold = 10)]
#[request_context]
async fn handle_api_request(req: Request) -> Result<Response, ApiError> {
    // Production-ready endpoint with full observability
}

// Critical data processing with full monitoring
#[log_disk_usage(threshold = 80)]
#[log_memory_usage]
#[log_database_pool(pool_name = "analytics", threshold = 75)]
#[log_data_quality(domain = "analytics", threshold = 98)]
#[audit_log]
#[log_retries(max_attempts = 5)]
fn process_analytics_data(data: &[Record]) -> Result<AnalyticsResult, ProcessingError> {
    // Critical data processing with comprehensive monitoring
}
```

### Microservices Configuration
```rust
// Service-to-service communication
#[log_service_communication(service_name = "user_service", timeout_ms = 1500)]
#[log_trace_correlation(service_name = "order_service")]
#[circuit_breaker(failure_threshold = 3)]
#[log_health_check(service_name = "user_service", threshold = 99)]
async fn call_user_service(user_id: u64) -> Result<User, ServiceError> {
    // Monitored inter-service communication
}

// Distributed transaction processing
#[log_transaction(domain = "payment", timeout_ms = 5000)]
#[log_distributed_lock(domain = "payment_lock", timeout_ms = 10000)]
#[log_consensus_operation(domain = "payment_consensus", timeout_ms = 3000)]
#[audit_log]
async fn process_payment(payment: PaymentRequest) -> Result<Receipt, PaymentError> {
    // Distributed payment processing with full observability
}
```

## Performance Considerations

- **Minimal Overhead**: Most macros add < 1μs overhead per function call
- **Lazy Evaluation**: Log messages are only formatted when needed
- **Conditional Compilation**: Debug macros can be compiled out in release builds
- **Throttling**: Built-in rate limiting prevents log flooding
- **Non-blocking**: Logging operations don't block application execution

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                          Application Code                       │
├─────────────────────────────────────────────────────────────────┤
│                     Procedural Macros                           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐│
│  │   DevOps    │ │ Distributed │ │  Security   │ │  Business   ││
│  │    Infra    │ │   Systems   │ │    &        │ │    Logic    ││
│  │ Monitoring  │ │ Monitoring  │ │ Compliance  │ │ Monitoring  ││
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘│
├─────────────────────────────────────────────────────────────────┤
│                      LibLogger Core                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐│
│  │   Logging   │ │  Context    │ │   Output    │ │    Utils    ││
│  │    API      │ │ Management  │ │   Sinks     │ │     &       ││
│  │             │ │             │ │             │ │  Helpers    ││
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

## Integration Examples

### With Popular Crates

```rust
// Tokio integration
#[measure_time]
#[log_concurrency]
async fn async_task() {
    tokio::time::sleep(Duration::from_millis(100)).await;
}

// Serde integration
#[log_args(request)]
#[log_response]
fn api_handler(request: ApiRequest) -> Result<ApiResponse, ApiError> {
    // Automatically logs serializable structs
}

// Error handling with anyhow
#[log_errors]
#[log_retries(max_attempts = 3)]
fn fallible_operation() -> anyhow::Result<String> {
    // Works seamlessly with anyhow error types
}
```

### Observability Stack Integration

```rust
// Prometheus metrics
#[metrics_counter(counter_name = "http_requests_total")]
#[log_custom_metrics(metric_name = "response_time")]
fn http_handler() {
    // Integrates with Prometheus monitoring
}

// Distributed tracing
#[log_trace_correlation(service_name = "api")]
#[trace_span]
fn traced_operation() {
    // Compatible with OpenTelemetry and Jaeger
}
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Adding New Macros

1. Define the macro in `liblogger_macros/src/lib.rs`
2. Add utility functions in `liblogger_macros/src/macro_utils.rs`
3. Write tests in `logger_tests/src/`
4. Update documentation in `proc_macros.md`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Roadmap

- [ ] **Async Improvements**: Better async/await support across all macros
- [ ] **Custom Sinks**: Pluggable output destinations (Elasticsearch, Kafka, etc.)
- [ ] **Configuration Management**: Runtime configuration of macro behavior
- [ ] **Performance Optimizations**: Zero-cost abstractions for hot paths
- [ ] **Cloud Integration**: Native support for AWS CloudWatch, GCP Logging
- [ ] **AI/ML Integration**: Intelligent anomaly detection and pattern recognition
- [ ] **Visual Dashboards**: Web-based monitoring and alerting interface

## Examples

Check out the `examples/` directory for comprehensive usage examples:

- `basic_usage.rs` - Getting started with core logging
- `macro_showcase.rs` - Demonstration of all available macros
- `microservice_example.rs` - Full microservice with comprehensive monitoring
- `performance_monitoring.rs` - Performance-focused instrumentation
- `security_auditing.rs` - Security and compliance logging examples

## Support

- Documentation: https://docs.rs/liblogger
- Issue Tracker: https://github.com/yourusername/liblogger/issues
- Discussions: https://github.com/yourusername/liblogger/discussions
- Email Support: support@liblogger.dev

## Acknowledgments

- Inspired by the observability needs of modern distributed systems
- Built on the shoulders of the excellent Rust ecosystem
- Special thanks to the Rust macro system that makes this level of instrumentation possible

---

*LibLogger - Making Rust applications observable, one function at a time.*
