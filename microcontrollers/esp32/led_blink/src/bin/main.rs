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
    gpio::{Level, Output, OutputConfig},
    main,
};

use esp_println::println;
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    println!("Hello World");

    let mut led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    let delay = Delay::new();

    loop {
        led.set_high();
        println!("LED HIGH");
        delay.delay_millis(1000);
        led.set_low();
        println!("LED LOW");
        delay.delay_millis(1000);
    }
}
