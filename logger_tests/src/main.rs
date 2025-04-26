use liblogger::{Logger, log_info, log_warn, log_error, log_debug};

fn main() {
    // Initialize the logger from default config file
    Logger::init();
    
    // Now using the macros instead of direct Logger calls
    log_info!("Application started");

    if let Err(err) = perform_database_connection() {
        log_error!(&format!("Database connection failed: {:?}", err));
    }

    log_warn!("Disk space running low", Some("disk=/dev/sda1 free=2%".to_string()));

    log_debug!("User login attempt", Some("user_id=12345 ip=192.168.1.1".to_string()));
    
    // Demonstrate module path and filename enrichment
    test_module::log_something();
}

fn perform_database_connection() -> Result<(), String> {
    Err("timeout after 10s".to_string())
}

mod test_module {
    use liblogger::{log_info};
    
    pub fn log_something() {
        log_info!("Log from a nested module", Some("module=test_module".to_string()));
    }
}
