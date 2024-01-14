# Rust on ESP32-C3

Try-ing to get Rust running on a M5-STAMP-C3U I have ly-ing around

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