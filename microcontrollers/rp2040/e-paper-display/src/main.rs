#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_10X20},
    prelude::*,
    text::{Baseline, Text, TextStyleBuilder},
};

use epd_waveshare::{
    epd1in54_v2::{Display1in54, Epd1in54},
    prelude::*,
};

use fugit::RateExtU32;
use rp2040_hal::{
    Sio,
    clocks::{Clock, init_clocks_and_plls},
    gpio::{FunctionSio, FunctionSpi, Pin, PullDown, PullUp, SioInput, SioOutput},
    pac,
    spi::Spi,
    timer::Timer,
    watchdog::Watchdog,
};

#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let _cp = pac::CorePeripherals::take().unwrap();

    let mut wdt = Watchdog::new(pac.WATCHDOG);
    let clocks = init_clocks_and_plls(
        12_000_000u32,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut wdt,
    )
    .ok()
    .unwrap();

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let sio = Sio::new(pac.SIO);
    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // SPI Pins
    // GP2 = SCK, GP3 = MOSI (TX), no MISO needed
    let sclk = pins.gpio2.into_function::<FunctionSpi>();
    let mosi = pins.gpio3.into_function::<FunctionSpi>();

    let spi = Spi::<_, _, _, 8>::new(pac.SPI0, (mosi, sclk)).init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        4_000_000u32.Hz(),
        embedded_hal::spi::MODE_0,
    );

    // Control Pins
    let cs: Pin<_, FunctionSio<SioOutput>, PullDown> = pins.gpio5.into_push_pull_output();
    let dc: Pin<_, FunctionSio<SioOutput>, PullDown> = pins.gpio6.into_push_pull_output();
    let rst: Pin<_, FunctionSio<SioOutput>, PullDown> = pins.gpio10.into_push_pull_output();
    let busy: Pin<_, FunctionSio<SioInput>, PullUp> = pins.gpio11.into_pull_up_input();

    let mut spi_bus = embedded_hal_bus::spi::ExclusiveDevice::new(spi, cs, timer).unwrap();

    // Re-create delay (Consumed above) using timer

    // EPD initialization
    let mut epd = Epd1in54::new(
        &mut spi_bus,
        busy,
        dc,
        rst,
        &mut timer,
        None, // Use default LUT
    )
    .unwrap();

    // Framebuffer
    let mut display = Display1in54::default();
    display.set_rotation(DisplayRotation::Rotate0);

    display.clear(Color::White).unwrap();

    // Draw text
    let style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Color::Black)
        .background_color(Color::White)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    Text::with_text_style("Hello", Point::new(10, 40), style, text_style)
        .draw(&mut display)
        .unwrap();

    Text::with_text_style("Rust on RP2040!", Point::new(10, 70), style, text_style)
        .draw(&mut display)
        .unwrap();

    Text::with_text_style("E-Paper Works !", Point::new(10, 100), style, text_style)
        .draw(&mut display)
        .unwrap();

    epd.update_frame(&mut spi_bus, display.buffer(), &mut timer)
        .unwrap();

    epd.display_frame(&mut spi_bus, &mut timer).unwrap();

    epd.sleep(&mut spi_bus, &mut timer).unwrap();

    loop {
        cortex_m::asm::wfi();
    }
}
