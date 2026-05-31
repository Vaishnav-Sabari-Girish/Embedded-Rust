#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::Level,
    gpio::{Input, InputConfig},
    gpio::{Output, OutputConfig},
    main,
};
use esp_println::println;
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    let button = Input::new(peripherals.GPIO0, InputConfig::default());
    let delay = Delay::new();

    loop {
        if button.is_low() {
            led.set_high();
            println!("Button Pressed");
            delay.delay_millis(50);
        } else {
            led.set_low();
        }
    }
}
