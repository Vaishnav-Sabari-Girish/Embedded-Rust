#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use rp2040_hal::{
    Sio,
    clocks::{Clock, init_clocks_and_plls},
    pac,
    pio::PIOExt,
    timer::Timer,
    usb::UsbBus,
    watchdog::Watchdog,
};
use smart_leds::{RGB8, SmartLedsWrite};
use ws2812_pio::Ws2812;

// USB Imports
use static_cell::StaticCell;
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

// --- THE IGNITION KEY ---
// This places the 256-byte bootloader at the very start of the flash memory.
// Without this, the RP2040 ROM refuses to jump to our code.
#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;
// ------------------------

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();

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

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut _delay = cortex_m::delay::Delay::new(cp.SYST, clocks.system_clock.freq().to_Hz());

    let sio = Sio::new(pac.SIO);
    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // USB Setup
    // The USB driver needa a space of memory that lives forever ('static lifetime)
    // We use StaticCell to safely provide this without using unsafe static mut
    static USB_BUS: StaticCell<UsbBusAllocator<UsbBus>> = StaticCell::new();

    let usb_bus = USB_BUS.init(UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    )));

    let mut serial = SerialPort::new(usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("My Rust Lab")
            .product("RP2040-Zero Serial")
            .serial_number("TEST")])
        .unwrap()
        .device_class(2) // 2 represents CDC (Communication Device Class)
        .build();

    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);

    // Set up the WS2812 NeoPixel on GP16
    let mut ws = Ws2812::new(
        pins.gpio16.into_function(),
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
        timer.count_down(),
    );

    let mut color_state = 0;
    // timer.get_counter() returns microseconds; 500ms = 500_000 microseconds
    let delay_us = 500_000;
    let mut last_time = timer.get_counter();

    loop {
        // POLL the USB device
        // This must be called as often as possible
        if usb_dev.poll(&mut [&mut serial]) {
            let mut buf = [0u8; 64];

            // If user types something in the serial monitor read it
            if let Ok(count) = serial.read(&mut buf)
                && count > 0
            {
                // Echo it back
                let _ = serial.write(b"\r\nYou typed something!\r\n");
            }
        }

        // TIMER Check
        // We use a non-blocking delay
        let current_time = timer.get_counter();
        if current_time.ticks().wrapping_sub(last_time.ticks()) >= delay_us {
            last_time = current_time;

            match color_state {
                0 => {
                    ws.write([RGB8::new(32, 0, 0)].iter().cloned()).unwrap();
                    let _ = serial.write(b"LED is RED\r\n");
                    color_state = 1;
                }
                1 => {
                    ws.write([RGB8::new(0, 32, 0)].iter().cloned()).unwrap();
                    let _ = serial.write(b"LED is GREEN\r\n");
                    color_state = 2;
                }
                _ => {
                    ws.write([RGB8::new(0, 0, 32)].iter().cloned()).unwrap();
                    let _ = serial.write(b"LED is BLUE\r\n");
                    color_state = 0;
                }
            }
        }
    }
}
