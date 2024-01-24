use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc as _;
use esp_idf_svc::hal::gpio::InterruptType;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::gpio::Pull;
use esp_idf_svc::hal::task::notification::Notification;

use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::Result;

pub mod charger;
pub mod evse;
pub mod leds;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let org_charger = Arc::new(Mutex::new(charger::Charger::default()));
    let org_relay = Arc::new(Mutex::new(
        PinDriver::output(peripherals.pins.gpio8).unwrap(),
    ));

    let charger = org_charger.clone();
    charger.lock().unwrap().set_state(charger::State::Available);

    let relay = org_relay.clone();
    relay.lock().unwrap().set_low()?;

    // onboard button thread
    let relay = org_relay.clone();
    let charger = org_charger.clone();
    thread::spawn(move || {
        let mut button = PinDriver::input(peripherals.pins.gpio9).unwrap();
        button.set_pull(Pull::Up).unwrap();
        button.set_interrupt_type(InterruptType::PosEdge).unwrap();

        let notification = Notification::new();
        let notifier = notification.notifier();

        unsafe {
            button
                .subscribe(move || {
                    notifier.notify_and_yield(NonZeroU32::new(1).unwrap());
                })
                .unwrap();
        }

        loop {
            button.enable_interrupt().unwrap();
            notification.wait(esp_idf_svc::hal::delay::BLOCK);

            let mut c = charger.lock().unwrap();
            let mut r = relay.lock().unwrap();
            let res = c.transition(charger::ChargerInput::SwipeCard);
            match res {
                Some(charger::ChargerOutput::LockedAndPowerIsOn) => {
                    log::info!("Charger locked and power is on.");
                    r.set_high().unwrap();
                }
                Some(charger::ChargerOutput::Unlocked) => {
                    log::info!("Charger unlocked.");
                    r.set_low().unwrap();
                }
                Some(charger::ChargerOutput::Errored) => {
                    log::info!("Charger errored.");
                    r.set_low().unwrap();
                    c.set_state(charger::State::Error);
                    thread::sleep(Duration::from_secs(5));
                    c.set_state(charger::State::Available);
                }
                None => {
                    log::warn!("Charger transition failed.");
                }
            }
        }
    });

    // cable switch thread
    let relay = org_relay.clone();
    let charger = org_charger.clone();
    thread::spawn(move || {
        let mut button = PinDriver::input(peripherals.pins.gpio10).unwrap();
        button.set_pull(Pull::Up).unwrap();

        let mut old_state = false;
        loop {
            let state = button.is_low();
            if state != old_state {
                old_state = state;

                let mut c = charger.lock().unwrap();
                let res = if state {
                    c.transition(charger::ChargerInput::PlugIn)
                } else {
                    c.transition(charger::ChargerInput::PlugOut)
                };
                match res {
                    Some(charger::ChargerOutput::Errored) => {
                        log::info!("Charger errored.");
                        relay.lock().unwrap().set_low().unwrap();
                        c.set_state(charger::State::Error);
                    }
                    None => {
                        log::warn!("Charger transition failed.");
                    }
                    _ => {}
                }
                //debounce
                thread::sleep(Duration::from_millis(400));
            }
            thread::sleep(Duration::from_millis(100));
        }
    });
    // report thread
    let charger = org_charger.clone();
    thread::spawn(move || {
        let mut led = leds::Led::new(2);

        loop {
            led.set_from_state(charger.lock().unwrap().get_state());

            thread::sleep(Duration::from_millis(100));
        }
    });

    // mqtt thread
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(10));
            // TODO mqtt stuff
        }
    });

    loop {
        // Change to blocking wait
        thread::sleep(Duration::from_secs(1));
    }
}
