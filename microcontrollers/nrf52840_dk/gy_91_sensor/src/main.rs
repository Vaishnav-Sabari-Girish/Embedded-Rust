#![no_std]
#![no_main]

mod bmp280;
mod mpu9250;

use defmt::info;
use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_nrf::{bind_interrupts, peripherals, twim};

use static_cell::StaticCell;

static TWIM_TX_BUFFER: StaticCell<[u8; 128]> = StaticCell::new();

bind_interrupts!(struct Irqs {
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    cortex_m::asm::delay(32_000_000);

    info!("=== GY-91 Sensor Module ===");

    // Configure I2C (TWIM) at 400 kHz
    let mut twim_config = twim::Config::default();
    twim_config.frequency = twim::Frequency::K400;

    let tx_buffer = TWIM_TX_BUFFER.init([0; 128]);

    let mut i2c = twim::Twim::new(
        p.TWISPI0,
        Irqs,
        p.P0_26,
        p.P0_27,
        twim_config,
        tx_buffer,
    );

    // --- BMP280 (I2C address 0x76 with SDO grounded) ---
    info!("Initializing BMP280...");
    let bmp_cfg = bmp280::Config::normal_1hz();
    let mut bmp = bmp280::Bmp280::new(&mut i2c, 0x76, &bmp_cfg)
        .expect("BMP280 init failed");
    info!("BMP280 OK");

    // --- MPU9250 (I2C address 0x68 with AD0 grounded) ---
    info!("Initializing MPU9250...");
    let mpu_cfg = mpu9250::Config::default();
    let mpu = mpu9250::Mpu9250::new(&mut i2c, &mpu_cfg)
        .expect("MPU9250 init failed");
    info!("MPU9250 OK");

    // --- Main Loop ---
    info!("Entering main loop...");
    loop {
        // Read MPU9250: accel, gyro, temp, mag (all from a single I2C handle)
        match mpu.read_all(&mut i2c) {
            Ok(m) => {
                info!(
                    "MPU accel=[{},{},{}]g gyro=[{},{},{}]°/s",
                    m.accel[0], m.accel[1], m.accel[2],
                    m.gyro[0], m.gyro[1], m.gyro[2],
                );
                info!(
                    "MPU mag=[{},{},{}]uT temp={}°C",
                    m.mag[0], m.mag[1], m.mag[2],
                    m.temp,
                );
            }
            Err(_) => {
                info!("MPU read error");
            }
        }

        // Read BMP280 temperature (in hundredths °C)
        match bmp.read_temperature(&mut i2c) {
            Ok(temp) => {
                info!("BMP temp= {} °C", temp as f32 / 100.0);
            }
            Err(_) => info!("BMP temp read error"),
        }

        // Read BMP280 pressure (in Pa)
        match bmp.read_pressure(&mut i2c) {
            Ok(pressure) => {
                info!("BMP pressure= {} hPa", pressure as f32 / 100.0);
            }
            Err(_) => info!("BMP pressure read error"),
        }

        Timer::after(Duration::from_secs(1)).await;
    }
}
