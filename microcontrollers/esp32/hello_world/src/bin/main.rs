#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{delay::Delay, main};
use esp_println::println;
esp_bootloader_esp_idf::esp_app_desc!();


#[main]
fn main() -> ! {
    // generator version: 0.4.0
    let delay = Delay::new();

    loop {
        println!("Hello World");
        delay.delay_millis(500);
    }
    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.1/examples/src/bin
}
