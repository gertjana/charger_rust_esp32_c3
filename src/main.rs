use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc as _;
use esp_idf_svc::hal::gpio::InterruptType;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::gpio::Pull;
use esp_idf_svc::hal::task::notification::Notification;

use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use anyhow::Result;

pub mod charger;
pub mod evse;
pub mod leds;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();

    let charger = Arc::new(Mutex::new(charger::Charger::default()));

    start_gpio_thread(charger.clone())?;

    start_charger_report_thread(charger.clone())?;

    start_mqtt_thread(charger.clone())?;

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn start_gpio_thread(charger: Arc<Mutex<charger::Charger>>) -> Result<JoinHandle<()>, std::io::Error> {
    thread::Builder::new().name("gpio".to_string()).spawn(move || {
        print_thread_message(thread::current().name().unwrap(), "Started thread.");

        let peripherals = Peripherals::take().unwrap();

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

        // Set up a variable that keeps track of press button count
        let mut count = 0_u32;
        let charger_actions = ["available", "occupied", "charging", "error", "off"];
        let mut led = leds::Led::new(2);

        loop {
            button.enable_interrupt().unwrap();
            notification.wait(esp_idf_svc::hal::delay::BLOCK);
            let action = charger_actions[count as usize % charger_actions.len()];
            count = count.wrapping_add(1);

            led.set_from_action(action);

            let mut c = charger.lock().unwrap();
            c.set_state_from_action(action);
        }
    })

}

fn start_charger_report_thread(charger: Arc<Mutex<charger::Charger>>) -> Result<JoinHandle<()>, std::io::Error> {
    let mut old_state = charger::State::Off;
    thread::Builder::new().name("report".to_string()).spawn(move || {
        print_thread_message(thread::current().name().unwrap(), "Started thread.");
        loop {
            let state = charger.lock().unwrap().get_state();
            if state != old_state {
                log::info!("{:?}: Charger State: {:?}", thread::current().name().unwrap(), state);
                old_state = state;
            }
            thread::sleep(std::time::Duration::from_millis(100));
        }
    })
}

fn start_mqtt_thread(_charger: Arc<Mutex<charger::Charger>>) -> Result<JoinHandle<()>, std::io::Error> {
    thread::Builder::new().name("mqtt".to_string()).spawn(move || {
        print_thread_message(thread::current().name().unwrap(), "Started thread.");
        loop {
            thread::sleep(std::time::Duration::from_millis(100));
        }
    })
}


fn print_thread_message(thread_name: &str, message: &str) {
    let name = thread_name.replace("\"", "|");
    log::info!("{:?}: {}", name, message);
}