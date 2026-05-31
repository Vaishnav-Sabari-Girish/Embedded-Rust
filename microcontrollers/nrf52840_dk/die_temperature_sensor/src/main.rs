#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _; // Initializes the global defmt logger
use panic_probe as _; // Catches panics and sends them through defmt

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use embassy_nrf::bind_interrupts;
use embassy_nrf::temp::{InterruptHandler, Temp};

/*
* This macro wires up the hardware interrupt so that the Temp driver
* knows exactly when the physical sensor finishes a reading
*/
bind_interrupts!(struct Irqs {
    TEMP => InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the HAL and grab the peripheral singleton
    let p = embassy_nrf::init(Default::default());

    cortex_m::asm::delay(32_000_000);

    info!("Starting Die Temperature Monitor...");

    let mut temp_sensor = Temp::new(p.TEMP, Irqs);

    loop {
        let fixed_temp = temp_sensor.read().await; // Returns a value of type I30F2
        let raw_temp: i32 = fixed_temp.to_bits(); // Converts I30F2 to i32

        /*
         * The nrf52 temperature sensor returns an integer representing 0.25 C steps.
         * For example: a raw value of 64 equals 16.0 C (64 * 0.25)
         */
        let whole_degrees = raw_temp / 4;
        let fractional_part = (raw_temp.abs() % 4) * 25; // Converts remainder to .00, .25, .5, .75

        info!("Die Temperature: {}.{} C", whole_degrees, fractional_part);

        Timer::after(Duration::from_secs(1)).await;
    }
}
