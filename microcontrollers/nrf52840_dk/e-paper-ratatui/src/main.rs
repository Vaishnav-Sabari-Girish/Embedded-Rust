#![no_std]
#![no_main]

extern crate alloc;

use core::{convert::Infallible, mem::MaybeUninit};

use alloc::boxed::Box;
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
    pixelcolor::Rgb888,
    prelude::*,
};
use epd_waveshare::{
    epd1in54_v2::{Display1in54, Epd1in54},
    prelude::*,
};

use mousefood::prelude::{
    EmbeddedBackendConfig,
    EmbeddedBackend
};

use ratatui::{
    style::{Color, Style},
    widgets::{Block, Paragraph, Wrap},
    Frame,
    Terminal
};

use embedded_alloc::LlffHeap as Heap;

// Global Allocator
#[global_allocator]
static HEAP: Heap = Heap::empty();
const  HEAP_SIZE: usize = 128 * 1024;
static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

pub struct DisplayAdapter(pub Display1in54);

impl Dimensions for DisplayAdapter {
    fn bounding_box(&self) -> embedded_graphics::primitives::Rectangle {
        self.0.bounding_box()
    }
}

impl DrawTarget for DisplayAdapter {
    type Color = Rgb888;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = Pixel<Self::Color>> {
        let converted = pixels.into_iter().map(|Pixel(p, c)| Pixel(p, c.into()));
        self.0.draw_iter(converted)
    }
}

bind_interrupts!(struct Irqs {
    TWISPI0 => spim::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the HAL and grab the peripheral singleton
    let p = embassy_nrf::init(Default::default());

    info!("Initializing E-Paper display");

    unsafe {
        HEAP.init(core::ptr::addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE);
    }

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
    
    // Uncomment these lines to clear display
    //epd.clear_frame(&mut spi_bus, &mut delay).unwrap();
    //epd.display_frame(&mut spi_bus, &mut delay).unwrap();
    //epd.sleep(&mut spi_bus, &mut delay).unwrap();

    let mut display_adapter = DisplayAdapter(display);

    let backend_config = EmbeddedBackendConfig {
        flush_callback: Box::new(move |adapter: &mut DisplayAdapter| {
            info!("Flushing Ratatui frame to EPD");
            epd.update_and_display_frame(&mut spi_bus, adapter.0.buffer(), &mut delay).unwrap();
        }),
        ..Default::default()
    };

    let backend = EmbeddedBackend::new(&mut display_adapter, backend_config);
    let mut terminal = Terminal::new(backend).unwrap();

    // Comment this line when clearing the display
    terminal.draw(draw_ui).unwrap();

    info!("Update complete. CPU going to sleep");

    loop {
        cortex_m::asm::wfi();
    }
}

fn draw_ui(frame: &mut Frame) {
    let text = "nRF52840 + Ratatui";

    let epaper_theme = Style::default().fg(Color::Black).bg(Color::White);

    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });

    let bordered_block = Block::bordered()
        .title("nRF-Ratatui-Embassy")
        .style(epaper_theme);

    frame.render_widget(paragraph.block(bordered_block), frame.area());
}
