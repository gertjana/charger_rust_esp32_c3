# EV Charger in Rust on a ESP32-C3

Try-ing to get Rust running on a M5-STAMP-C3U I have ly-ing around

I blog about this here: 

https://gertjanassies.dev/blog/240101_rust_on_esp32

## Schematic

![Schematic](images/schematic.png?raw=true "Schematic")

> please note that the code is using the button (GPIO9) and multicolor led (GPIO2) that are on the M5 Stamp

## Get Started

### Prerequisites

 - Prerequisites for ESP-IDF: https://docs.espressif.com/projects/esp-idf/en/latest/esp32/get-started/linux-macos-setup.html#step-1-install-prerequisites 
 - rustup: https://rustup.rs/
 - Prereqs for esp-idf-template: https://github.com/esp-rs/esp-idf-template

### build project

#### change target on your esp-board

```
cargo build
espflash target/[xtensa-esp32-espidf|xtensa-esp32s2-espidf|xtensa-esp32s3-espidf|riscv32imc-esp-espidf]/debug/<your-project-name> --monitor
```

It will show a list of eligble usb devices for the connected board.
