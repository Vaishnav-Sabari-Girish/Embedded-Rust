#![no_std]
#![no_main]

use nrf52833_hal::Timer;
use panic_halt as _;
use embedded_hal::{delay::DelayNs, digital::{InputPin, OutputPin}};
use cortex_m_rt::entry;
use microbit::Board;
use rtt_target::{rprintln, rtt_init_print};


#[entry]
fn main() -> ! {
    rtt_init_print!();

    let mut board = Board::take().unwrap();

    let mut btn_a = board.buttons.button_a;
    let mut btn_b = board.buttons.button_b;

    let _ = board.display_pins.col1.set_low();
    let mut led1 = board.display_pins.row1;
    let mut led2 = board.display_pins.row2;

    let mut timer = Timer::new(board.TIMER0);
    
    rprintln!("Button Press Check");
    loop{
    
        if let Ok(true) =  btn_a.is_low(){
            rprintln!("Button A is pressed");
            let _ = led1.set_high();
            let _ = led2.set_low();
            timer.delay_ms(10);
        }
        else if let Ok(true) = btn_b.is_low() {
            rprintln!("Button B is pressed");
            let _ = led1.set_low();
            let _ = led2.set_high();
            timer.delay_ms(10);
        }
        else{
            rprintln!("No buttons pressed");
            let _ = led1.set_low();
            let _ = led2.set_low();
            timer.delay_ms(10);
        }
        timer.delay_ms(100);
    }
}