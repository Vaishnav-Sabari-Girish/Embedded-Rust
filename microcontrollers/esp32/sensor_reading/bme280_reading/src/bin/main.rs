#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_backtrace as _;
use esp_hal::{
    i2c::master::{
        I2c, Config
    },
    clock::CpuClock,
    delay::Delay,
    main
};
use esp_println::println;

use bmpe280::bme280::BME280;
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let delay = Delay::new();

    let sda = peripherals.GPIO21;
    let scl = peripherals.GPIO22;

    let i2c = I2c::new(peripherals.I2C0, Config::default())
        .unwrap()
        .with_sda(sda)
        .with_scl(scl);

    let mut bme = BME280::new(i2c, delay);

    loop {
        let m = bme.measure();

        println!(
            "2 : Temperature: {0} C, Pressure : {1} , Humidity : {2}%",
            m.temperature, m.pressure, m.humidity
        );

        delay.delay_millis(1000);
    }

}
