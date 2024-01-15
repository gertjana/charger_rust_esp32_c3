use crate::evse::Evse;
use uuid::Uuid;
use rust_fsm::*;


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

#[derive(Debug)]
pub enum ChargerInput {
    PlugIn,
    PlugOut,
    SwipeCard,
}

#[derive(Debug, PartialEq)]
pub enum ChargerOutput {
    Unlocked,
    LockedAndPowerOn
}

pub struct ChargerMachine{}

impl StateMachineImpl for ChargerMachine {
    type Input = ChargerInput;
    type State = State;
    type Output = ChargerOutput;
    const INITIAL_STATE: Self::State = State::Available;

    fn transition(state: &Self::State, input: &Self::Input) -> Option<Self::State> {
        match (state, input) {
            (State::Available, ChargerInput::PlugIn) => {
                Some(State::Occupied)
            }
            (State::Occupied, ChargerInput::PlugOut) => {
                Some(State::Available)
            }
            (State::Occupied, ChargerInput::SwipeCard) => {
                Some(State::Charging)
            }
            (State::Charging, ChargerInput::SwipeCard) => {
                Some(State::Occupied)
            }
            _ => None,
        }
    }

    fn output(state: &Self::State, input: &Self::Input) -> Option<Self::Output> {
        match (state, input) {
            (State::Available, _) => {
                Some(ChargerOutput::Unlocked)
            }
            (State::Occupied, _) => {
                Some(ChargerOutput::Unlocked)
            }
            (State::Charging, _) => {
                Some(ChargerOutput::LockedAndPowerOn)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
use std::sync::{Arc, Mutex};

#[test]
fn test_charger_machine() {
    let machine = Arc::new(Mutex::new(StateMachine::<ChargerMachine>::new()));
    {
        let mut lock = machine.lock().unwrap();
        let res = lock.consume(&ChargerInput::PlugIn).unwrap();
        assert_eq!(res, Some(ChargerOutput::Unlocked));
        assert_eq!(lock.state(), &State::Occupied);    
    }

    let machine_swipe = machine.clone();
    {
        let mut lock = machine_swipe.lock().unwrap();
        let res = lock.consume(&ChargerInput::SwipeCard).unwrap();
        assert_eq!(res, Some(ChargerOutput::LockedAndPowerOn));
        assert_eq!(lock.state(), &State::Charging);    
    }

    let machine_swipe_again = machine.clone();
    {
        let mut lock = machine_swipe_again.lock().unwrap();
        let res = lock.consume(&ChargerInput::SwipeCard).unwrap();
        assert_eq!(res, Some(ChargerOutput::Unlocked));
        assert_eq!(lock.state(), &State::Occupied);    
    }
    let machine_unplug = machine.clone();
    {
        let mut lock = machine_unplug.lock().unwrap();
        let res = lock.consume(&ChargerInput::PlugOut).unwrap();
        assert_eq!(res, Some(ChargerOutput::Unlocked));
        assert_eq!(lock.state(), &State::Available);    
    }
}