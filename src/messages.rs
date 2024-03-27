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

pub fn start_transaction_request() -> Result<serde_json::Value, serde_json::Error> {
    let message = rust_ocpp::v1_6::messages::start_transaction::StartTransactionRequest {
        connector_id: 1,
        id_tag: "123456".into(),
        meter_start: 0,
        timestamp: chrono::Utc::now(),
        ..Default::default()
    };
    let value = serde_json::to_value(message)?;
    Ok(value)
}

pub fn stop_transaction_request() -> Result<serde_json::Value, serde_json::Error> {
    let message = rust_ocpp::v1_6::messages::stop_transaction::StopTransactionRequest {
        id_tag: Some("123456".into()),
        meter_stop: 0,
        timestamp: chrono::Utc::now(),
        transaction_id: 1,
        reason: Some(rust_ocpp::v1_6::types::Reason::Local),
        ..Default::default()
    };
    let value = serde_json::to_value(message)?;
    Ok(value)
}
