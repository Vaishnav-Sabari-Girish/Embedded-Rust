#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embedded_graphics::pixelcolor::Rgb565;
use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use {esp_backtrace as _, esp_println as _};

// Embedded graphics stuff
use embedded_graphics::{
    mono_font::MonoTextStyle,
    prelude::*,
    text::{Baseline, Text},
};

// Larger Font
use profont::PROFONT_24_POINT;

// SPI Stuff
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::delay::Delay;
use esp_hal::spi::master::Config as SpiConfig;
use esp_hal::spi::master::Spi;
use esp_hal::spi::Mode as SpiMode;
use esp_hal::time::Rate;
use mipidsi::{
    interface::SpiInterface,
    models::ILI9341Rgb565,
    options::{Orientation, Rotation},
    Builder,
};

// GPIO Stuff
use esp_hal::gpio::{Level, Output, OutputConfig};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Initialize spi
    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(4))
            .with_mode(SpiMode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO18)
    .with_mosi(peripherals.GPIO23);

    let cs = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    let reset = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());

    let mut buffer = [0u8; 512];

    let spi_dev = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let interface = SpiInterface::new(spi_dev, dc, &mut buffer);

    let mut display = Builder::new(ILI9341Rgb565, interface)
        .reset_pin(reset)
        .init(&mut Delay::new())
        .unwrap();

    display.clear(Rgb565::BLACK).unwrap();
    display
        .set_orientation(Orientation::default().rotate(Rotation::Deg270))
        .unwrap();

    let text_style = MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::GREEN);

    Text::with_baseline("Test TFT 2", Point::new(60, 80), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    loop {
        info!("Hello world!");
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.1/examples/src/bin
}
