use crate::evse::Evse;
use uuid::Uuid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
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
    Swipe,
}
impl ChargerInput {
    fn as_str(&self) -> &str {
        match self {
            ChargerInput::PlugIn => "PlugIn",
            ChargerInput::PlugOut => "PlugOut",
            ChargerInput::Swipe => "Swipe",
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

    pub fn set_state(&mut self, state: State) -> State {
        self.state = state;
        self.state.clone()
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

    pub fn transition(&mut self, input: ChargerInput) -> Result<(State, ChargerOutput)> {
        let orginal_state = self.state.clone();

        let output = match (input, self.state.clone()) {
            (ChargerInput::PlugIn, State::Available) => {
                Ok((self.set_state(State::Occupied), ChargerOutput::Unlocked))
            }
            (ChargerInput::PlugOut, State::Occupied) => {
                Ok((self.set_state(State::Available), ChargerOutput::Unlocked))
            }
            (ChargerInput::Swipe, State::Occupied) => Ok((
                self.set_state(State::Charging),
                ChargerOutput::LockedAndPowerIsOn,
            )),
            (ChargerInput::Swipe, State::Charging) => {
                Ok((self.set_state(State::Occupied), ChargerOutput::Unlocked))
            }
            (ChargerInput::PlugOut, State::Charging) => Err("Cannot unplug while charging".into()),
            (_, State::Error) => Ok((self.set_state(State::Error), ChargerOutput::Errored)),
            _ => {
                log::warn!(
                    "{} with {} is an unknown Charger transition ",
                    input.clone().as_str(),
                    self.state.clone().as_str()
                );
                Err("Invalid transition".into())
            }
        };
        log::info!(
            "Transistion state: {} with input: {} -> state: {}, output: {:?}",
            orginal_state.as_str(),
            input.as_str(),
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
