use esp_idf_svc as _;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::gpio::Pull;
use esp_idf_svc::hal::gpio::InterruptType;
use esp_idf_svc::hal::task::notification::Notification;
use esp_idf_hal::peripherals::Peripherals;

use std::num::NonZeroU32;
use std::thread;
use std::sync::{Arc, Mutex};

use anyhow::Result;

pub mod leds;
pub mod charger;


fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();


    let charger = Arc::new(Mutex::new(charger::Charger::new("1".to_string(), charger::State::Available, vec![])));

    button_thread(charger.clone());

    charger_report_thread(charger.clone());

    log::info!("Main loop, does nothing but indefinitely sleep for 1 sec ");
    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
    
}


fn button_thread(charger: Arc<Mutex<charger::Charger>>) {
    thread::spawn(move || {
        log::info!("Started button thread");
        let peripherals = Peripherals::take().unwrap();

        let mut button = PinDriver::input(peripherals.pins.gpio9).unwrap();
        button.set_pull(Pull::Up).unwrap();
        button.set_interrupt_type(InterruptType::PosEdge).unwrap();
    
        let notification = Notification::new();
        let notifier = notification.notifier();
    
        unsafe {
            button.subscribe(move || {
                notifier.notify_and_yield(NonZeroU32::new(1).unwrap());
            }).unwrap();
        }
    
        // Set up a variable that keeps track of press button count
        let mut count = 0_u32;  
        let charger_actions = ["available","occupied", "charging", "error", "off"];
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
    });
}

fn charger_report_thread(charger: Arc<Mutex<charger::Charger>>) {
    let mut old_state = charger::State::Off;
    thread::spawn(move || {
        log::info!("Started Charger report thread");
        loop {
            let state = charger.lock().unwrap().get_state();
            if state != old_state {
                log::info!("Charger State: {:?}", state);
                old_state = state;
            }
            thread::sleep(std::time::Duration::from_millis(100));    
        }
    });
}