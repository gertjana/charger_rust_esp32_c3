use smart_leds_trait::{SmartLedsWrite, White};
use std::thread::sleep;
use std::time::Duration;
use ws2812_esp32_rmt_driver::driver::color::LedPixelColorGrbw32;
use ws2812_esp32_rmt_driver::{LedPixelEsp32Rmt, RGBW8};

use crate::charger::State;

pub struct Led {
    driver: LedPixelEsp32Rmt<RGBW8, LedPixelColorGrbw32>,
}

impl Led {
    pub fn new(pin: u32) -> Self {
        Self {
            driver: LedPixelEsp32Rmt::<RGBW8, LedPixelColorGrbw32>::new(0, pin).unwrap(),
        }
    }

    pub fn set_from_rgbw(&mut self, rgbw: RGBW8) {
        let pixels = std::iter::repeat(rgbw).take(1);

        self.driver.write(pixels).unwrap();
    }

    /// Write an RGBW color to our NeoPixel Jewel.
    pub fn set_from_r_g_b_w(&mut self, red: u8, green: u8, blue: u8, white: u8) {
        let pixels = std::iter::repeat(RGBW8::from((red, green, blue, White(white)))).take(1);

        self.driver.write(pixels).unwrap();
    }

    pub fn set_from_action(&mut self, action: &str) {
        let color = self.get_charging_color(action);
        self.set_from_rgbw(color);
    }

    pub fn set_from_state(&mut self, state: State) {
        match state {
            State::Error => self.set_from_action("error"),
            State::Available => self.set_from_action("available"),
            State::Occupied => self.set_from_action("occupied"),
            State::Charging => self.set_from_action("charging"),
            State::Off => self.set_from_action("off"),
        };
    }

    pub fn get_charging_color(&self, action: &str) -> RGBW8 {
        match action {
            "error" => RGBW8::from((255, 0, 0, White(0))), // red
            "available" => RGBW8::from((0, 255, 0, White(0))), // green
            "occupied" => RGBW8::from((255, 255, 0, White(0))), // yellow
            "charging" => RGBW8::from((0, 0, 255, White(0))), // blue
            _ => RGBW8::from((0, 0, 0, White(0))),         // off
        }
    }
}

impl Default for Led {
    fn default() -> Self {
        Self::new(2)
    }
}

pub fn test_leds() {
    let mut led = Led::new(2);

    let charging_colors = ["available", "occupied", "charging", "error", "off"];

    for action in charging_colors.iter() {
        led.set_from_action(action);
        sleep(Duration::from_millis(1500));
    }
}
