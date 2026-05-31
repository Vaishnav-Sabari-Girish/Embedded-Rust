#![no_std]
#![no_main]

use defmt::{info, unwrap};
use defmt_rtt as _; // Initializes the global defmt logger
use panic_probe as _; // Catches panics and sends them through defmt

use embassy_executor::Spawner;
use embassy_nrf::Peri;
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive, Pull};

// pool_size = 4 tells Embassy to reserve enough memory
// to run up to 4 instances of this specific task concurrently
#[embassy_executor::task(pool_size = 4)]
async fn button_handler(id: u8, btn_pin: Peri<'static, AnyPin>, led_pin: Peri<'static, AnyPin>) {
    let mut btn = Input::new(btn_pin, Pull::Up);
    let mut led = Output::new(led_pin, Level::High, OutputDrive::Standard);

    info!("Task for button {} has started!", id);

    loop {
        btn.wait_for_low().await;
        info!("Button {} PRESSED", id);
        led.set_low();

        btn.wait_for_high().await;
        info!("Button {} RELEASED", id);
        led.set_high();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    // Keep the debugger stable at startup before the executor goes to sleep
    cortex_m::asm::delay(32_000_000);

    info!("Starting 4-Button Async Matrix....");

    // 1. .into() cleanly erases the specific pin type into a generic Peri<'static, AnyPin>
    // 2. We unwrap!() the task function itself to extract the SpawnToken
    // 3. We pass the token to spawner.spawn()
    spawner.spawn(unwrap!(button_handler(1, p.P0_11.into(), p.P0_13.into())));
    spawner.spawn(unwrap!(button_handler(2, p.P0_12.into(), p.P0_14.into())));
    spawner.spawn(unwrap!(button_handler(3, p.P0_24.into(), p.P0_15.into())));
    spawner.spawn(unwrap!(button_handler(4, p.P0_25.into(), p.P0_16.into())));
}
