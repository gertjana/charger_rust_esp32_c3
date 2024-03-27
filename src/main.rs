use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};

use esp_idf_hal::i2c::*;
use esp_idf_hal::peripherals::Peripherals;

use esp_idf_svc as _;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::gpio::{InterruptType, PinDriver, Pull};
use esp_idf_svc::hal::task::notification::Notification;
use esp_idf_svc::mqtt::client::*;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::EspWifi;

use ssd1306::{prelude::*, I2CDisplayInterface};

use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::commands::{OCPPResponse, UniqueId};
use crate::display::{Display, DisplayData};
use crate::messages::heartbeat_request;
use crate::queue::{FifoQueue, Queue};

pub mod charger;
pub mod commands;
pub mod config;
pub mod display;
pub mod evse;
pub mod leds;
pub mod messages;
pub mod queue;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let config = config::Config::default();

    let peripherals = Peripherals::take().unwrap();

    let org_charger = Arc::new(Mutex::new(charger::Charger::default()));

    let org_command_queue_send = Arc::new(FifoQueue::<commands::OCPPRequest>::new());

    let org_command_queue_recieve = Arc::new(FifoQueue::<commands::OCPPResponse>::new());

    let org_unique_id = Arc::new(Mutex::new(UniqueId::new()));

    let org_relay = Arc::new(Mutex::new(
        PinDriver::output(peripherals.pins.gpio8).unwrap(),
    ));

    let charger = org_charger.clone();
    charger.lock().unwrap().set_state(charger::State::Available);

    let relay = org_relay.clone();
    relay.lock().unwrap().set_low()?;

    // Wifi

    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = EspWifi::new(peripherals.modem, sysloop, Some(nvs))?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: config.ssid.as_str().into(),
        password: config.password.as_str().into(),
        auth_method: AuthMethod::None,
        ..Default::default()
    }))?;

    wifi.start()?;
    wifi.connect()?;

    let mut cnt = 0;
    while !wifi.is_connected().unwrap() && cnt < 10 {
        cnt += 1;
        let config = wifi.get_configuration().unwrap();
        log::info!("Waiting for station {:?}", config);
        thread::sleep(Duration::from_secs(1));
    }

    let ip = wifi.get_configuration().unwrap();
    log::info!("Connected to wifi {:?}", ip);

    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio20;

    let i2c_config = I2cConfig::new().baudrate(100_000.into());
    let i2c = I2cDriver::new(i2c, sda, scl, &i2c_config)?;

    let interface: I2CInterface<I2cDriver<'_>> = I2CDisplayInterface::new(i2c);

    let display = Arc::new(Mutex::new(Display::new(interface)));

    thread::sleep(Duration::from_millis(2000));
    let ip = wifi.sta_netif().get_ip_info().unwrap().ip;

    let display_data = DisplayData::new(
        "ESP32 EV Charger".into(),
        "Initializing..".into(),
        "Available".into(),
        ip.to_string(),
    );

    let d = display.clone();
    d.lock().unwrap().set_data(display_data);
    d.lock().unwrap().refresh();
    // MQTT

    let conf = MqttClientConfiguration {
        client_id: Some(&config.mqtt.client_id),
        ..Default::default()
    };

    let broker = config.mqtt.broker.clone();
    let command_receive_queue = org_command_queue_recieve.clone();
    let mut client = EspMqttClient::new(&config.mqtt.broker, &conf, move |message_event| {
        match message_event.as_ref().unwrap() {
            Event::Connected(_) => log::info!("Connected to MQTT {}", broker),
            Event::Subscribed(id) => log::info!("Subscribed to {} id", id),
            Event::Received(msg) => {
                log::info!("Received message: {}", String::from_utf8_lossy(msg.data()));
                if !msg.data().is_empty() {
                    match OCPPResponse::from_ocpp_json_message(msg.data()) {
                        Ok(response) => {
                            command_receive_queue.push(response);
                        }
                        Err(e) => {
                            log::error!("Failed to parse message: {:?}", e);
                        }
                    }
                }
            }
            _ => log::info!("Unhandled event: {:?}", message_event),
        };
    })?;

    let topic = format!(
        "/system/{}/{}",
        &config.charger.model, &config.charger.serial
    );
    client.subscribe(&topic, QoS::AtLeastOnce)?;

    let d = display.clone();
    d.lock().unwrap().set_message("MQTT Connected".to_string());
    d.lock().unwrap().refresh();

    thread::sleep(Duration::from_millis(500));

    let unique_id = org_unique_id.clone();
    let command = commands::OCPPRequest {
        message_type_id: commands::MessageType::Call,
        unique_id: unique_id.lock().unwrap().next_id().to_string(),
        action: "BootNotification".to_string(),
        payload: messages::boot_notification_request()?,
    };

    log::info!("sending BootNotification: {:?}", command);

    org_command_queue_send.clone().push(command);

    // onboard button thread
    let send_queue = org_command_queue_send.clone();
    let unique_id = org_unique_id.clone();
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
            let res = c.transition(charger::ChargerInput::Swipe);
            match res {
                Ok((_, charger::ChargerOutput::LockedAndPowerIsOn)) => {
                    r.set_high().unwrap();
                    send_queue.push(commands::OCPPRequest {
                        message_type_id: commands::MessageType::Call,
                        unique_id: unique_id.lock().unwrap().next_id().to_string(),
                        action: "StartTransaction".to_string(),
                        payload: messages::start_transaction_request().unwrap(),
                    });
                }
                Ok((_, charger::ChargerOutput::Unlocked)) => {
                    r.set_low().unwrap();
                    send_queue.push(commands::OCPPRequest {
                        message_type_id: commands::MessageType::Call,
                        unique_id: unique_id.lock().unwrap().next_id().to_string(),
                        action: "StopTransaction".to_string(),
                        payload: messages::stop_transaction_request().unwrap(),
                    });
                }
                Ok((_, charger::ChargerOutput::Errored)) => {
                    r.set_low().unwrap();
                    c.set_state(charger::State::Error);
                    thread::sleep(Duration::from_secs(5));
                    c.set_state(charger::State::Available);
                }
                Err(e) => {
                    log::warn!("Charger transition failed: {}", e);
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
                    Ok((_, charger::ChargerOutput::Errored)) => {
                        log::info!("Charger errored.");
                        relay.lock().unwrap().set_low().unwrap();
                        c.set_state(charger::State::Error);
                    }
                    Err(e) => {
                        log::warn!("Charger transition failed. {:?}", e);
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
    let d = display.clone();
    let charger = org_charger.clone();
    thread::spawn(move || {
        let mut led = leds::Led::new(2);
        let mut old_state = charger::State::Off;
        loop {
            let new_state = charger.lock().unwrap().get_state();
            led.set_from_state(new_state);
            if old_state != charger.lock().unwrap().get_state().clone() {
                d.lock()
                    .unwrap()
                    .set_state(charger.lock().unwrap().get_state().as_str().to_string());
                d.lock().unwrap().refresh();
                old_state = charger.lock().unwrap().get_state();
            }
            thread::sleep(Duration::from_millis(100));
        }
    });

    // mqtt thread publish when send queue is not empty
    let d = display.clone();
    let send_queue = org_command_queue_send.clone();
    thread::spawn(move || loop {
        if !send_queue.is_empty() {
            let command = send_queue.pop();
            let topic = format!(
                "/charger/{}/{}",
                &config.charger.model, &config.charger.serial
            );
            log::info!("Publishing {} to topic: {}", command.action, &topic);
            let result = client.enqueue(
                &topic,
                QoS::AtMostOnce,
                false,
                command.to_ocpp_json_message().unwrap().as_bytes(),
            );
            d.lock()
                .unwrap()
                .set_message(format!("-> {}", command.action));
            d.lock().unwrap().refresh();
            if let Err(e) = result {
                log::error!("Failed to publish message: {:?}", e);
            }
        }
        thread::sleep(Duration::from_millis(100));
    });

    // Handle retrieve queue thread

    let d = display.clone();
    let receive_queue = org_command_queue_recieve.clone();
    thread::spawn(move || loop {
        if !receive_queue.is_empty() {
            let response = receive_queue.pop();
            log::info!("Processing Response: {:?}", response);
            match response.action.as_str() {
                "BootNotification" => {
                    let payload = serde_json::from_value::<
                        rust_ocpp::v1_6::messages::boot_notification::BootNotificationResponse,
                    >(response.payload)
                    .unwrap();
                    log::info!("BootNotificationResponse: {:?}", payload);
                }
                "Heartbeat" => {
                    let payload = serde_json::from_value::<
                        rust_ocpp::v1_6::messages::heart_beat::HeartbeatResponse,
                    >(response.payload)
                    .unwrap();
                    log::info!("HeartbeatResponse: {:?}", payload);
                }
                _ => {
                    log::info!("Unhandled response: {:?}", response);
                }
            }
            d.lock()
                .unwrap()
                .set_message(format!("<- {}", response.action));
            d.lock().unwrap().refresh();
        }
        thread::sleep(Duration::from_millis(100));
    });

    // Heartbeat thread
    let unique_id = org_unique_id.clone();
    let send_queue = org_command_queue_send.clone();
    thread::spawn(move || loop {
        let command = commands::OCPPRequest {
            message_type_id: commands::MessageType::Call,
            unique_id: unique_id.lock().unwrap().next_id().to_string(),
            action: "Heartbeat".to_string(),
            payload: heartbeat_request().unwrap(),
        };
        send_queue.push(command);

        thread::sleep(Duration::from_secs(60));
    });

    let d = display.clone();
    loop {
        thread::sleep(Duration::from_secs(10));
        let mut disp = d.lock().unwrap();
        disp.set_message("".into());
        disp.set_ip(ip.to_string());
        disp.refresh();
    }
}
