#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_backtrace as _;
use esp_hal::{
    peripherals::Peripherals,
    clock::CpuClock,
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    main
};
use esp_println::print;
use esp_println::println;
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // generator version: 0.4.0

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut led = Output::new(peripherals.GPIO14, Level::Low, OutputConfig::default());
    let delay = Delay::new();

    loop {
        led.set_high();
        print!("LED : ");
        println!("{}", 1);
        delay.delay_millis(1000);
        led.set_low();
        print!("LED : ");
        println!("{}", 0);
        delay.delay_millis(1000);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.1/examples/src/bin
}
