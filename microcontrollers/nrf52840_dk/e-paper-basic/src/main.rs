#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _; // Initializes the global defmt logger
use panic_probe as _; // Catches panics and sends them through defmt

use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Level, Output, OutputDrive, Pull},
    peripherals, spim,
};
use embassy_time::Delay;

use embedded_hal_bus::spi::ExclusiveDevice;

use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_10X20},
    prelude::*,
    text::{Baseline, Text, TextStyleBuilder},
};
use epd_waveshare::{
    epd1in54_v2::{Display1in54, Epd1in54},
    prelude::*,
};

bind_interrupts!(struct Irqs {
    TWISPI0 => spim::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the HAL and grab the peripheral singleton
    let p = embassy_nrf::init(Default::default());

    info!("Initializing E-Paper display");

    let mut spim_config = spim::Config::default();
    spim_config.frequency = spim::Frequency::M4; // 4MHz clock

    let spi = spim::Spim::new_txonly(p.TWISPI0, Irqs, p.P1_15, p.P1_13, spim_config);

    let cs = Output::new(p.P1_12, Level::High, OutputDrive::Standard);
    let dc = Output::new(p.P1_11, Level::Low, OutputDrive::Standard);
    let rst = Output::new(p.P1_10, Level::High, OutputDrive::Standard);

    let busy = Input::new(p.P1_08, Pull::Up);

    let mut spi_bus = ExclusiveDevice::new(spi, cs, Delay).unwrap();

    let mut delay = Delay;

    let mut epd = Epd1in54::new(&mut spi_bus, busy, dc, rst, &mut delay, None).unwrap();

    let mut display = Display1in54::default();
    display.set_rotation(DisplayRotation::Rotate0);
    display.clear(Color::White).unwrap();

    let style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Color::Black)
        .background_color(Color::White)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();
    // Uncomment these lines to clear display
    //epd.clear_frame(&mut spi_bus, &mut delay).unwrap();
    //epd.display_frame(&mut spi_bus, &mut delay).unwrap();
    //epd.sleep(&mut spi_bus, &mut delay).unwrap();

    // Comment this line when clearing the display
    Text::with_text_style("Hello", Point::new(10, 40), style, text_style)
        .draw(&mut display)
        .unwrap();

    info!("Pushing frame to display");

    epd.update_frame(&mut spi_bus, display.buffer(), &mut delay)
        .unwrap();
    epd.display_frame(&mut spi_bus, &mut delay).unwrap();
    epd.sleep(&mut spi_bus, &mut delay).unwrap();

    info!("Update complete. CPU going to sleep");

    loop {
        cortex_m::asm::wfi();
    }
}
