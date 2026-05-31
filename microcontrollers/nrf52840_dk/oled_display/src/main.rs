#![no_std]
#![no_main]

use core::fmt::Write;
use defmt::{info, unwrap};
use defmt_rtt as _; // Initializes the global defmt logger
use panic_probe as _; // Catches panics and sends them through defmt

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_nrf::{bind_interrupts, peripherals, temp, twim};

// UI 
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};

use heapless::String;
use sh1106::{prelude::*, Builder};

bind_interrupts!(struct Irqs {
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
    TEMP => temp::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the HAL and grab the peripheral singleton
    let p = embassy_nrf::init(Default::default());
    cortex_m::asm::delay(32_000_000);

    info!("Initializing I2C and OLED");

    let mut temp_sensor = temp::Temp::new(p.TEMP, Irqs);

    let mut twim_config = twim::Config::default();
    twim_config.frequency = twim::Frequency::K400;

    let mut twim_tx_buffer = [0u8; 128];

    let i2c = twim::Twim::new(
        p.TWISPI0, 
        Irqs, p.P0_26, 
        p.P0_27, 
        twim_config, 
        &mut twim_tx_buffer
    );

    let mut display: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();

    display.init().unwrap();
    display.flush().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    loop {
        let temperature = temp_sensor.read().await.to_bits();

        let whole = temperature / 4;
        let frac = (temperature.abs() % 4) * 25;

        let mut s: String<32> = String::new();
        unwrap!(core::write!(&mut s, "Die Temp: {}.{:.02} C", whole, frac));

        info!("{}", s.as_str());

        display.clear();

        unwrap!(Text::with_baseline(&s, Point::new(0, 16), text_style, Baseline::Top).draw(&mut display));
        display.flush().unwrap();

        Timer::after(Duration::from_secs(1)).await;
    }
}
