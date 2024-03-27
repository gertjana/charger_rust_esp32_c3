use esp_idf_hal::i2c::*;
use ssd1306::{mode::TerminalMode, prelude::*, Ssd1306};
use std::fmt::Write;

pub struct Display {
    display: Ssd1306<I2CInterface<I2cDriver<'static>>, DisplaySize128x64, TerminalMode>,
    pub data: DisplayData,
}

impl Display {
    pub fn new(interface: I2CInterface<I2cDriver<'static>>) -> Self {
        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
            .into_terminal_mode();
        let _ = display.init();
        let _ = display.clear();

        Self {
            display,
            data: DisplayData::default(),
        }
    }

    pub fn set_data(&mut self, data: DisplayData) {
        self.data = data;
    }

    pub fn refresh(&mut self) {
        let _ = self.display.clear();
        let _ = write!(
            self.display,
            "{}\n\n{}\n\n{}\n\n{}",
            self.data.title, self.data.ip, self.data.state, self.data.message
        );
    }

    pub fn set_message(&mut self, message: String) {
        self.data.message = limit(&message, 16);
    }

    pub fn set_state(&mut self, state: String) {
        self.data.state = capitalize(&state);
    }

    pub fn set_ip(&mut self, ip: String) {
        self.data.ip = ip;
    }
}

#[derive(Debug, Clone)]
pub struct DisplayData {
    pub title: String,
    pub ip: String,
    pub message: String,
    pub state: String,
}

impl Default for DisplayData {
    fn default() -> Self {
        Self {
            title: "ESP32 EV Charger".into(),
            message: "".into(),
            state: "Available".into(),
            ip: "".into(),
        }
    }
}

impl DisplayData {
    pub fn new(title: String, message: String, state: String, ip: String) -> Self {
        Self {
            title,
            message,
            state,
            ip,
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn limit(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}
