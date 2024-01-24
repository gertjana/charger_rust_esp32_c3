use crate::evse::Evse;
use uuid::Uuid;


/// ChargerId
/// an UUID based id for a charger
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

/// State
/// The different states a charger can be in
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum State {
    Available,
    Occupied,
    Charging,
    Error,
    Off,
}

impl State {
    pub fn as_str(&self) -> &str {
        match self {
            State::Available => "available",
            State::Occupied => "occupied",
            State::Charging => "charging",
            State::Error => "error",
            State::Off => "off",
        }
    }
}

/// ChargerInput
/// Charger state machine inputs
#[derive(Debug, Clone, Copy)]
pub enum ChargerInput {
    PlugIn,
    PlugOut,
    SwipeCard,
}
impl ChargerInput {
    fn as_str(&self) -> &str {
        match self {
            ChargerInput::PlugIn => "PlugIn",
            ChargerInput::PlugOut => "PlugOut",
            ChargerInput::SwipeCard => "SwipeCard",
        }
    }
}

/// ChargerOutput
/// Charger state machine outputs
#[derive(Debug)]
pub enum ChargerOutput {
    Unlocked,
    LockedAndPowerIsOn,
    Errored,
}

/// Charger
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

    pub fn transition(&mut self, input: ChargerInput) -> Option<ChargerOutput> {
        let log_input = format!(
            "Transitioning state: {:?}, input: {:?}",
            self.state.clone(),
            input
        );

        let output = match (input, self.state.clone()) {
            (ChargerInput::PlugIn, State::Available) => {
                self.set_state(State::Occupied);
                Some(ChargerOutput::Unlocked)
            }
            (ChargerInput::PlugOut, State::Occupied) => {
                self.set_state(State::Available);
                Some(ChargerOutput::Unlocked)
            }
            (ChargerInput::SwipeCard, State::Occupied) => {
                self.set_state(State::Charging);
                Some(ChargerOutput::LockedAndPowerIsOn)
            }
            (ChargerInput::SwipeCard, State::Charging) => {
                self.set_state(State::Occupied);
                Some(ChargerOutput::Unlocked)
            }
            (ChargerInput::PlugOut, State::Charging) => {
                self.set_state(State::Error);
                Some(ChargerOutput::Errored)
            }
            _ => {
                log::warn!(
                    "{} with {} is an unknown Charger transition ",
                    input.clone().as_str(),
                    self.state.clone().as_str()
                );
                None
            }
        };
        log::info!(
            "to state: {} -> {}, output: {:?}",
            log_input.as_str(),
            self.state.clone().as_str(),
            output,
        );
        output
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
