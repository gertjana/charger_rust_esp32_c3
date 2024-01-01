

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum State {
    Available,
    Occupied,
    Charging,
    Error,
    Off
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)] 
pub enum ConnectorType {
    Type2,
    CHAdeMO,
    CCS,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Evse {
    pub id: String,
    pub connector_type: ConnectorType,
    pub power: u32
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Charger {
    pub id: String,
    pub state: State,
    pub evses: Vec<Evse>,
}

impl Charger {
    pub fn new(id: String, state: State, evses: Vec<Evse>) -> Self {
        Self {
            id,
            state,
            evses,
        }
    }

    pub fn get_state(&self) -> State {
        self.state.clone()
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    pub fn set_state_from_action(&mut self, action: &str) {
        match action {
            "error" =>      self.set_state(State::Error),
            "available" =>  self.set_state(State::Available),
            "occupied" =>   self.set_state(State::Occupied),
            "charging" =>   self.set_state(State::Charging),
            "off" =>        self.set_state(State::Off),
            _ =>            self.set_state(State::Error),
        };
    }
}