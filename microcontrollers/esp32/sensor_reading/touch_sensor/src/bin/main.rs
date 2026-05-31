#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, InputConfig},
    main,
};
use esp_println::println;
esp_bootloader_esp_idf::esp_app_desc!();

mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const BG_CYAN: &str = "\x1b[46m";
}

struct TouchSensor {
    is_pressed: bool,
    was_pressed: bool,
}

impl TouchSensor {
    fn new() -> Self {
        Self {
            is_pressed: false,
            was_pressed: false,
        }
    }

    fn update(&mut self, current_state: bool) {
        self.was_pressed = self.is_pressed;
        self.is_pressed = current_state;
    }

    fn just_pressed(&self) -> bool {
        self.is_pressed && !self.was_pressed
    }

    fn just_released(&self) -> bool {
        !self.is_pressed && self.was_pressed
    }
}

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let touch_pin = Input::new(peripherals.GPIO5, InputConfig::default());

    let mut touch_sensor = TouchSensor::new();

    println!(
        "{}Touch sensor initiated, waiting for touch...{}",
        colors::BG_CYAN,
        colors::RESET
    );

    let delay = Delay::new();

    loop {
        let is_touched = touch_pin.is_high();

        touch_sensor.update(is_touched);

        if touch_sensor.just_pressed() {
            println!("{}Touch Detected{}", colors::GREEN, colors::RESET);
        } else if touch_sensor.just_released() {
            println!("{}Touch released{}", colors::RED, colors::RESET);
        }

        delay.delay_millis(20u32);
    }
}
