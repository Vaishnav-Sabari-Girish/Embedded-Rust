#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    main,
};
use esp_println::println;
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripehrals = esp_hal::init(config);

    let echo_pin = Input::new(peripehrals.GPIO4, InputConfig::default());
    let mut trig_pin = Output::new(peripehrals.GPIO2, Level::Low, OutputConfig::default());

    let delay = Delay::new();

    loop {
        trig_pin.set_low();
        delay.delay_micros(2);
        trig_pin.set_high();
        delay.delay_micros(10);
        trig_pin.set_low();

        //Wait for echo pin to go high
        while echo_pin.is_low() {}

        //Measure pulse duration
        let start = esp_hal::time::Instant::now();
        while echo_pin.is_high() {}
        let end = esp_hal::time::Instant::now();

        let pulse_duration = start.elapsed() - end.elapsed();

        let distance_cm = (pulse_duration.as_micros() as f32 * 0.0343) / 2.0;

        println!("Distance: {}", distance_cm);
        delay.delay_millis(100);
    }
}
