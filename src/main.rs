use embedded_svc::mqtt::client::QoS;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};

use esp_idf_hal::peripherals::Peripherals;

use esp_idf_svc as _;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::gpio::InterruptType;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::gpio::Pull;
use esp_idf_svc::hal::task::notification::Notification;
use esp_idf_svc::mqtt::client::*;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::EspWifi;

use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::commands::OCPPResponse;
use crate::messages::heartbeat_request;
use crate::queue::{FifoQueue, Queue};

pub mod charger;
pub mod commands;
pub mod config;
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

    while !wifi.is_connected().unwrap() {
        // Get and print connection configuration
        let config = wifi.get_configuration().unwrap();
        log::info!("Waiting for station {:?}", config);
        thread::sleep(Duration::from_secs(1));
    }

    let ip = wifi.get_configuration().unwrap();
    log::info!("Connected to wifi {:?}", ip);

    // MQTT

    let conf = MqttClientConfiguration {
        client_id: Some(&config.mqtt.client_id),
        ..Default::default()
    };

    let command_receive_queue = org_command_queue_recieve.clone();
    let mut client = EspMqttClient::new(&config.mqtt.broker, &conf, move |message_event| {
        match message_event.as_ref().unwrap() {
            Event::Connected(_) => println!("Connected"),
            Event::Subscribed(id) => println!("Subscribed to {} id", id),
            Event::Received(msg) => {
                log::info!("Received message: {}", String::from_utf8_lossy(msg.data()));
                if !msg.data().is_empty() {
                    match serde_json::from_slice(msg.data()) {
                        Ok(response) => {
                            println!("Recieved {}", std::str::from_utf8(msg.data()).unwrap());
                            command_receive_queue
                                .push(OCPPResponse::from_ocpp_json_message(response).unwrap());
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

    //bootnotification
    let command = commands::OCPPRequest {
        message_type_id: commands::MessageType::Call,
        unique_id: "1".to_string(),
        action: "BootNotification".to_string(),
        payload: messages::boot_notification_request()?,
    };
    log::info!("sending BootNotification: {:?}", command);
    org_command_queue_send.clone().push(command);

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
            let res = c.transition(charger::ChargerInput::Swipe);
            match res {
                Ok((_, charger::ChargerOutput::LockedAndPowerIsOn)) => {
                    r.set_high().unwrap();
                }
                Ok((_, charger::ChargerOutput::Unlocked)) => {
                    r.set_low().unwrap();
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
    let charger = org_charger.clone();
    thread::spawn(move || {
        let mut led = leds::Led::new(2);

        loop {
            led.set_from_state(charger.lock().unwrap().get_state());

            thread::sleep(Duration::from_millis(100));
        }
    });

    // mqtt thread publish when send queue is not empty
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
                command.to_ocpp_json_messagce().unwrap().as_bytes(),
            );
            if let Err(e) = result {
                log::error!("Failed to publish message: {:?}", e);
            }
        }
        thread::sleep(Duration::from_millis(100));
    });

    let receive_queue = org_command_queue_recieve.clone();
    thread::spawn(move || loop {
        if !receive_queue.is_empty() {
            let response = receive_queue.pop();
            log::info!("Got a response back: {:?}", response);
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
        }
        thread::sleep(Duration::from_millis(100));
    });

    let send_queue = org_command_queue_send.clone();
    thread::spawn(move || loop {
        log::info!("Sending heartbeat");

        let command = commands::OCPPRequest {
            message_type_id: commands::MessageType::Call,
            unique_id: "1".to_string(),
            action: "Heartbeat".to_string(),
            payload: heartbeat_request().unwrap(),
        };

        send_queue.push(command);
        thread::sleep(Duration::from_secs(900));
    });

    loop {
        thread::sleep(Duration::from_secs(10));
    }
}
