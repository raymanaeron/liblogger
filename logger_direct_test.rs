use liblogger::{Logger, log_info, log_warn, log_error, log_debug};

fn main() {
    // Initialize the logger
    println!("Initializing logger...");
    Logger::init();
    println!("Logger initialized");

    // Direct test of log levels with no macros in between
    println!("Testing direct log calls...");
    log_debug!("DIRECT DEBUG LOG TEST");
    log_info!("DIRECT INFO LOG TEST");
    log_warn!("DIRECT WARNING LOG TEST");
    log_error!("DIRECT ERROR LOG TEST");
    println!("Direct log tests complete");
}
