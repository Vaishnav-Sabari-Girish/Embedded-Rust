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

// Embedded graphics imports
use embedded_graphics::{
    mono_font::MonoTextStyle,
    prelude::*,
    text::{Baseline, Text},
};

// Larger font
use profont::{PROFONT_18_POINT, PROFONT_24_POINT};

// SPI Stuff
use display_interface_spi::SPIInterface;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::delay::Delay;
use esp_hal::spi::master::Config as SpiConfig;
use esp_hal::spi::master::Spi;
use esp_hal::spi::Mode as SpiMode;
use esp_hal::time::Rate;
use ili9341::{DisplaySize240x320, Ili9341, Orientation};

// GPIO Stuff
use esp_hal::gpio::{Level, Output, OutputConfig};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // generator version: 0.4.0

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Initialize SPI
    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(4))
            .with_mode(SpiMode::_0),
    )
    .unwrap()
    .with_sck(peripherals.GPIO18)
    .with_mosi(peripherals.GPIO23);

    let cs = Output::new(peripherals.GPIO15, Level::Low, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    let reset = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());

    let spi_dev = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let interface = SPIInterface::new(spi_dev, dc);

    let mut display = Ili9341::new(
        interface,
        reset,
        &mut Delay::new(),
        Orientation::Landscape,
        DisplaySize240x320,
    )
    .unwrap();

    display.clear(Rgb565::WHITE).unwrap();

    let text_style = MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::BLUE);

    Text::with_baseline("Test TFT", Point::new(50, 150), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    let text_style = MonoTextStyle::new(&PROFONT_18_POINT, Rgb565::RED);

    Text::with_baseline("ESP32", Point::new(60, 180), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    loop {
        info!("Hello world!");
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(5000) {}
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.1/examples/src/bin
}
