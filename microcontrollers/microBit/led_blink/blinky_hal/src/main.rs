#![no_std]
#![no_main]

use cortex_m::asm::nop;
use cortex_m_rt::entry;
use embedded_hal::digital::{OutputPin, PinState};
use hal::{gpio::Level, pac};
use nrf52833_pac::Peripherals;
use nrf52833_hal as hal;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let p: Peripherals = pac::Peripherals::take().unwrap();
    let port0 = hal::gpio::p0::Parts::new(p.P0);
    let _col1 = port0.p0_28.into_push_pull_output(Level::Low);    //Underscore makes sure that col1
                                                                  //is grounded
    let mut row1 = port0.p0_21.into_push_pull_output(Level::Low);

    let mut is_on: bool = false;

    loop{
        let _ = row1.set_state(PinState::from(is_on));

        for _ in 0..700_000{
            nop();
        }

        is_on = !is_on;
    }
}
