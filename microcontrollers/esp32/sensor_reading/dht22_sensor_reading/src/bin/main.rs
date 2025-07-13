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
    gpio::{DriveMode, Output, OutputConfig, Pull},
    main, 
};
use esp_println::println;
use embedded_dht_rs::dht22;
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let od_config = OutputConfig::default()
        .with_drive_mode(DriveMode::OpenDrain)
        .with_pull(Pull::None);

    let od_for_dht22 = Output::new(
            peripherals.GPIO4, esp_hal::gpio::Level::High, od_config
        )
        .into_flex();

    od_for_dht22.peripheral_input();
    

    let delay = Delay::new();

    let mut dht22 = dht22::Dht22::new(od_for_dht22, delay);

    loop {
        delay.delay_millis(2000);

        println!("");

        match dht22.read() {
            Ok(sensor_reading) => println!(
                "DHT22 Sensor - Temperature: {} C , Humidity: {} %",
                sensor_reading.temperature,
                sensor_reading.humidity
            ),
            Err(error) => println!("An error occurred while trying to read the sensor"),
        }

        println!("_____________________________________________________");
    }
}
