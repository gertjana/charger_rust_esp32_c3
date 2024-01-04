use crate::evse::Evse;
use uuid::Uuid;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ChargerId {
    pub id: String,
}

impl ChargerId {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
        }
    }
}

impl Default for ChargerId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum State {
    Available,
    Occupied,
    Charging,
    Error,
    Off,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Charger {
    pub id: ChargerId,
    pub state: State,
    pub evses: Vec<Evse>,
}

impl Charger {
    pub fn new(id: ChargerId, state: State, evses: Vec<Evse>) -> Self {
        Self { id, state, evses }
    }

    pub fn get_state(&self) -> State {
        self.state.clone()
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    pub fn set_state_from_action(&mut self, action: &str) {
        match action {
            "error" => self.set_state(State::Error),
            "available" => self.set_state(State::Available),
            "occupied" => self.set_state(State::Occupied),
            "charging" => self.set_state(State::Charging),
            "off" => self.set_state(State::Off),
            _ => self.set_state(State::Error),
        };
    }
}

impl Default for Charger {
    fn default() -> Self {
        Self {
            id: ChargerId::new(),
            state: State::Off,
            evses: vec![Evse::default()],
        }
    }
}
