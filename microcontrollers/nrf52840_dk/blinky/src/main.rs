#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _; // Initializes the global defmt logger
use panic_probe as _; // Catches panics and sends them through defmt

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::{Duration, Timer};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the HAL and grab the peripheral singleton
    let p = embassy_nrf::init(Default::default());

    info!("Starting Embassy Blinky!");

    // On the nRF52840-DK, LED 1 is connected to pin P0.13.
    let mut led = Output::new(p.P0_13, Level::High, OutputDrive::Standard);

    loop {
        info!("LED ON");
        led.set_low();
        
        Timer::after(Duration::from_millis(500)).await;

        info!("LED OFF");
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;
    }
}
