use uuid::Uuid;

/// ConnectorType
/// The specific connector type of an EVSE
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ConnectorType {
    Type2,
    CHAdeMO,
    CCS,
}

/// EvseId
/// an UUID based id for an EVSE
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct EvseId {
    pub id: String,
}

impl EvseId {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
        }
    }
}

impl Default for EvseId {
    fn default() -> Self {
        Self::new()
    }
}

/// Evse
/// Electric Vehicle Supply Equipment (The part with the connector and the kWh meter)
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Evse {
    pub id: EvseId,
    pub connector_type: ConnectorType,
    pub power: u32,
}

impl Evse {
    pub fn new(id: EvseId, connector_type: ConnectorType, power: u32) -> Self {
        Self {
            id,
            connector_type,
            power,
        }
    }
}

impl Default for Evse {
    fn default() -> Self {
        Self {
            id: EvseId::new(),
            connector_type: ConnectorType::Type2,
            power: 11,
        }
    }
}
