#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::delay::DelayNs;
use panic_halt as _;

use embedded_graphics::{
    geometry::Size,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
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
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

// 50x50 grid
const GRID_W: usize = 50;
const GRID_H: usize = 50;
const CELL_SIZE: i32 = 4;

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

    //epd.clear_frame(&mut spi_bus, &mut timer).unwrap();
    //epd.display_frame(&mut spi_bus, &mut timer).unwrap();
    //epd.sleep(&mut spi_bus, &mut timer).unwrap();

    // Framebuffer
    let mut display = Display1in54::default();
    display.set_rotation(DisplayRotation::Rotate0);

    let mut current_grid = [[false; GRID_W]; GRID_H];
    let mut next_grid = [[false; GRID_W]; GRID_H];

    // Glider pattern
    current_grid[1][2] = true;
    current_grid[2][3] = true;
    current_grid[3][1] = true;
    current_grid[3][2] = true;
    current_grid[3][3] = true;

    display.clear(Color::White).unwrap();

    loop {
        // Calculates next generation
        for y in 0..GRID_H {
            for x in 0..GRID_W {
                let mut neighbors = 0;

                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }

                        let nx = x as isize + dx;
                        let ny = y as isize + dy;

                        if nx >= 0
                            && nx < GRID_W as isize
                            && ny >= 0
                            && ny < GRID_H as isize
                            && current_grid[ny as usize][nx as usize]
                        {
                            neighbors += 1;
                        }
                    }
                }

                // Conway's rules
                let is_alive = current_grid[y][x];
                next_grid[y][x] = match (is_alive, neighbors) {
                    (true, 2) | (true, 3) => true, // Survival
                    (false, 3) => true,            // Reproduction
                    _ => false,                    // Under/Overpopulation
                };
            }
        }

        display.clear(Color::White).unwrap();

        for (y, row) in next_grid.iter().enumerate() {
            for (x, &is_alive) in row.iter().enumerate() {
                if is_alive {
                    let rect = Rectangle::new(
                        Point::new(x as i32 * CELL_SIZE, y as i32 * CELL_SIZE),
                        Size::new(CELL_SIZE as u32, CELL_SIZE as u32),
                    );

                    rect.into_styled(PrimitiveStyle::with_fill(Color::Black))
                        .draw(&mut display)
                        .unwrap();
                }
            }
        }

        epd.update_frame(&mut spi_bus, display.buffer(), &mut timer)
            .unwrap();
        epd.display_frame(&mut spi_bus, &mut timer).unwrap();

        current_grid = next_grid;

        timer.delay_ms(1000);
    }
}
