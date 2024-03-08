// log.rs
/// Helper function to log in JSON format
pub fn log_info(message: &str, value: i32) {
    log::info!("{}", json!({"message": message, "value": value}));
}
