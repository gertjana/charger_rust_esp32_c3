use crate::config::Config;

pub fn boot_notification_request() -> Result<serde_json::Value, serde_json::Error> {
    let message = rust_ocpp::v1_6::messages::boot_notification::BootNotificationRequest {
        charge_point_vendor: Config::default().charger.vendor,
        charge_point_model: Config::default().charger.model,
        charge_point_serial_number: Some(Config::default().charger.serial),
        ..Default::default()
    };
    let value = serde_json::to_value(message)?;
    Ok(value)
}

pub fn heartbeat_request() -> Result<serde_json::Value, serde_json::Error> {
    let message = rust_ocpp::v1_6::messages::heart_beat::HeartbeatRequest {};
    let value = serde_json::to_value(message)?;
    Ok(value)
}
