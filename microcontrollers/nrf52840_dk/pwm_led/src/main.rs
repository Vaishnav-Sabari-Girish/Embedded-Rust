#![no_std]
#![no_main]

use core::range::RangeInclusive;
use defmt::info;
use defmt_rtt as _; // Initializes the global defmt logger
use panic_probe as _; // Catches panics and sends them through defmt

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_nrf::pwm::{DutyCycle, Prescaler, SimpleConfig, SimplePwm};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the HAL and grab the peripheral singleton
    let p = embassy_nrf::init(Default::default());

    cortex_m::asm::delay(32_000_000);

    info!("PWM LED fader");

    let mut config = SimpleConfig::default();

    // Duration of PWM
    config.max_duty = 1000;

    // Clock prescaler. Div1 means the base clock is 16MHz
    // PWm frequency = 16 MHz / max_duty = 16 KHz
    config.prescaler = Prescaler::Div1;

    // 4 channel PWM instance on LED 1 (P0.13), LED 2 (P0.14), LED 3 (P0.15), LED 4 (P0.16)
    let mut pwm = SimplePwm::new_4ch(p.PWM0, p.P0_13, p.P0_14, p.P0_15, p.P0_16, &config);

    let fade_range = RangeInclusive::from(0..=1000u16);

    loop {
        info!("Fading in");

        for duty in fade_range {
            // Invert the duty value so that 0 is OFF and 1000 is fully ON
            let dc = DutyCycle::inverted(duty);
            pwm.set_all_duties([dc, dc, dc, dc]);

            Timer::after(Duration::from_millis(2)).await;
        }

        info!("Fading out");
            
        for duty in fade_range.into_iter().rev() {
            // Invert the duty value so that 0 is OFF and 1000 is fully ON
            let dc = DutyCycle::inverted(duty);
            pwm.set_all_duties([dc, dc, dc, dc]);

            Timer::after(Duration::from_millis(2)).await;
        }
    }
}
