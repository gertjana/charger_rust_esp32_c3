pub struct MQTTConfig {
    pub broker: String,
    pub client_id: String,
}

impl Default for MQTTConfig {
    fn default() -> Self {
        Self {
            broker: "".into(),
            client_id: "".into(),
        }
    }
}

pub struct ChargerConfig {
    pub serial: String,
    pub vendor: String,
    pub model: String,
}

impl Default for ChargerConfig {
    fn default() -> Self {
        Self {
            serial: "".into(),
            model: "".into(),
            vendor: "".into(),
        }
    }
}

pub struct Config {
    pub ssid: String,
    pub password: String,
    pub mqtt: MQTTConfig,
    pub charger: ChargerConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ssid: "".into(),
            password: "".into(),
            mqtt: MQTTConfig::default(),
            charger: ChargerConfig::default(),
        }
    }
}
