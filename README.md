# Rust on ESP32-C3

Try-ing to get Rust running on a M5-STAMP-C3U I have ly-ing around

## Get Started

### Prerequisites

 - Prerequisites for ESP-IDF: https://docs.espressif.com/projects/esp-idf/en/latest/esp32/get-started/linux-macos-setup.html#step-1-install-prerequisites 
 - rustup: https://rustup.rs/
 - Prereqs for esp-idf-template

 ### create project

```
cargo generate esp-rs/esp-idf-template cargo
```
and answer the questions

then cd into your project and do a `cargo build`

Then flashing it to your esp device with

```
espflash -p /dev/ttyUSB0 target/[xtensa-esp32-espidf|xtensa-esp32s2-espidf|xtensa-esp32s3-espidf|riscv32imc-esp-espidf]/debug/<your-project-name> --monitor
```
