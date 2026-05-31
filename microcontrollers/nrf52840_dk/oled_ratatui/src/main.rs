#![no_std]
#![no_main]

extern crate alloc;

use defmt::info;
use defmt_rtt as _; // Initializes the global defmt logger
use panic_probe as _; // Catches panics and sends them through defmt

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_nrf::{bind_interrupts, peripherals, twim};

use sh1106::{prelude::*, Builder};
use mousefood::{EmbeddedBackend, EmbeddedBackendConfig};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use static_cell::StaticCell;

// Allocator
use core::mem::MaybeUninit;
use embedded_alloc::LlffHeap as Heap;

// Global allocator
#[global_allocator]
static HEAP: Heap = Heap::empty();
const  HEAP_SIZE: usize = 32 * 1024;
static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

static  TWIM_TX_BUFFER: StaticCell<[u8; 128]> = StaticCell::new();

bind_interrupts!(struct Irqs {
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the HAL and grab the peripheral singleton
    let p = embassy_nrf::init(Default::default());
    cortex_m::asm::delay(32_000_000);

    info!("Ratatui with nRF52840DK");

    unsafe {
        HEAP.init(core::ptr::addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE);
    }

    let mut twim_config = twim::Config::default();
    twim_config.frequency = twim::Frequency::K400;

    let tx_buffer = TWIM_TX_BUFFER.init([0; 128]);

    let i2c = twim::Twim::new(
        p.TWISPI0,
        Irqs,
        p.P0_26,
        p.P0_27,
        twim_config,
        tx_buffer
    );

    let mut display: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();

    display.init().unwrap();
    display.flush().unwrap();

    let backend = EmbeddedBackend::new(
        &mut display,
        EmbeddedBackendConfig { 
            flush_callback: alloc::boxed::Box::new(|d| {
                d.flush().unwrap();
            }),
            ..Default::default()
        },
    );

    let mut terminal = Terminal::new(backend).unwrap();
    let mut counter = 0;

    loop {
        terminal.draw(|f| {
            let area = f.area();

            let block = Block::default()
                .title("Embassy nRF")
                .borders(Borders::ALL);

            let text = alloc::format!("\n Uptime: \n {} ticks", counter);

            let paragraph = Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center);

            f.render_widget(paragraph, area);
        }).unwrap();

        counter += 1;
        Timer::after(Duration::from_millis(100)).await;
    }
}
