//! BMP280 bare-metal driver
//!
//! I2C register-level driver for the Bosch BMP280 pressure/temperature sensor.
//! Datasheet: BST-BMP280-DS001

use embedded_hal::blocking::i2c::{Write, WriteRead};

// --- Register addresses ---
const REG_ID: u8 = 0xD0;
const REG_RESET: u8 = 0xE0;
const REG_STATUS: u8 = 0xF3;
const REG_CTRL_MEAS: u8 = 0xF4;
const REG_CONFIG: u8 = 0xF5;
const REG_PRESS_MSB: u8 = 0xF7;
const REG_TEMP_MSB: u8 = 0xFA;
const REG_CALIB_START: u8 = 0x88;

const CHIP_ID: u8 = 0x58;
const RESET_CMD: u8 = 0xB6;

/// Calibration data stored in the BMP280 NVM.
#[derive(Debug, Clone)]
struct Calib {
    dig_t1: u16,
    dig_t2: i16,
    dig_t3: i16,
    dig_p1: u16,
    dig_p2: i16,
    dig_p3: i16,
    dig_p4: i16,
    dig_p5: i16,
    dig_p6: i16,
    dig_p7: i16,
    dig_p8: i16,
    dig_p9: i16,
}

/// Oversampling setting for temperature measurements.
#[derive(Copy, Clone)]
pub enum TempOversampling {
    Skip = 0b000,
    X1 = 0b001,
    X2 = 0b010,
    X4 = 0b011,
    X8 = 0b100,
    X16 = 0b101,
}

/// Oversampling setting for pressure measurements.
#[derive(Copy, Clone)]
pub enum PressOversampling {
    Skip = 0b000,
    X1 = 0b001,
    X2 = 0b010,
    X4 = 0b011,
    X8 = 0b100,
    X16 = 0b101,
}

/// Sensor power mode.
#[derive(Copy, Clone)]
pub enum PowerMode {
    Sleep = 0b00,
    Forced = 0b01,
    Normal = 0b11,
}

/// Standby time between measurements in normal mode (t_sb).
#[derive(Copy, Clone)]
pub enum StandbyTime {
    Ms0_5 = 0b000,
    Ms62_5 = 0b001,
    Ms125 = 0b010,
    Ms250 = 0b011,
    Ms500 = 0b100,
    Ms1000 = 0b101,
    Ms2000 = 0b110,
    Ms4000 = 0b111,
}

/// IIR filter coefficient.
#[derive(Copy, Clone)]
pub enum Filter {
    Off = 0b000,
    X2 = 0b001,
    X4 = 0b010,
    X8 = 0b011,
    X16 = 0b100,
}

/// BMP280 configuration struct (passed to `new`).
pub struct Config {
    pub temp_oversampling: TempOversampling,
    pub press_oversampling: PressOversampling,
    pub power_mode: PowerMode,
    pub standby_time: StandbyTime,
    pub filter: Filter,
}

impl Default for Config {
    /// Normal mode, 1x oversampling, 0.5ms standby, filter off.
    fn default() -> Self {
        Config {
            temp_oversampling: TempOversampling::X1,
            press_oversampling: PressOversampling::X1,
            power_mode: PowerMode::Normal,
            standby_time: StandbyTime::Ms0_5,
            filter: Filter::Off,
        }
    }
}

/// Pre-built config for weather monitoring (forced mode, 1x oversampling).
impl Config {
    pub fn weather_monitoring() -> Self {
        Config {
            temp_oversampling: TempOversampling::X1,
            press_oversampling: PressOversampling::X1,
            power_mode: PowerMode::Forced,
            standby_time: StandbyTime::Ms0_5,
            filter: Filter::Off,
        }
    }

    pub fn normal_1hz() -> Self {
        Config {
            temp_oversampling: TempOversampling::X1,
            press_oversampling: PressOversampling::X1,
            power_mode: PowerMode::Normal,
            standby_time: StandbyTime::Ms62_5,
            filter: Filter::Off,
        }
    }
}

/// BMP280 driver struct.
///
/// Holds the I2C address and calibration data.
/// All I/O methods take `&mut I2C` so the bus can be shared with other devices.
pub struct Bmp280 {
    addr: u8,
    calib: Calib,
    t_fine: i32,
}

impl Bmp280 {
    /// Initialise the BMP280: verify ID, load calibration, apply config.
    pub fn new<I2C, E>(i2c: &mut I2C, addr: u8, config: &Config) -> Result<Self, E>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>,
        E: core::fmt::Debug,
    {
        // 1. Verify chip ID
        let mut id = [0u8; 1];
        i2c.write_read(addr, &[REG_ID], &mut id)?;
        if id[0] != CHIP_ID {
            // We can't create a typed error easily, so just return the I2C error
            // but we need a way to signal "wrong chip". For simplicity we panic
            // through the caller — the caller can check.
            // Actually, this is a problem — we can only return E.
            // Let's work around by returning the I2C error from a dummy operation.
            // Better: just panic here since it's a fatal hardware issue.
            panic!("BMP280 ID mismatch: expected 0x{:02X}, got 0x{:02X}", CHIP_ID, id[0]);
        }

        // 2. Read 24 bytes of calibration data starting at 0x88
        let mut calib_bytes = [0u8; 24];
        i2c.write_read(addr, &[REG_CALIB_START], &mut calib_bytes)?;

        let calib = Calib {
            dig_t1: u16::from_le_bytes([calib_bytes[0], calib_bytes[1]]),
            dig_t2: i16::from_le_bytes([calib_bytes[2], calib_bytes[3]]),
            dig_t3: i16::from_le_bytes([calib_bytes[4], calib_bytes[5]]),
            dig_p1: u16::from_le_bytes([calib_bytes[6], calib_bytes[7]]),
            dig_p2: i16::from_le_bytes([calib_bytes[8], calib_bytes[9]]),
            dig_p3: i16::from_le_bytes([calib_bytes[10], calib_bytes[11]]),
            dig_p4: i16::from_le_bytes([calib_bytes[12], calib_bytes[13]]),
            dig_p5: i16::from_le_bytes([calib_bytes[14], calib_bytes[15]]),
            dig_p6: i16::from_le_bytes([calib_bytes[16], calib_bytes[17]]),
            dig_p7: i16::from_le_bytes([calib_bytes[18], calib_bytes[19]]),
            dig_p8: i16::from_le_bytes([calib_bytes[20], calib_bytes[21]]),
            dig_p9: i16::from_le_bytes([calib_bytes[22], calib_bytes[23]]),
        };

        let this = Bmp280 {
            addr,
            calib,
            t_fine: 0,
        };

        // 3. Write config
        this.write_config(i2c, config)?;

        Ok(this)
    }

    fn write_config<I2C, E>(&self, i2c: &mut I2C, config: &Config) -> Result<(), E>
    where
        I2C: Write<Error = E>,
    {
        // ctrl_meas: osrs_t[7:5] | osrs_p[4:2] | mode[1:0]
        let ctrl = ((config.temp_oversampling as u8) << 5)
            | ((config.press_oversampling as u8) << 2)
            | (config.power_mode as u8);
        i2c.write(self.addr, &[REG_CTRL_MEAS, ctrl])?;

        // config: t_sb[7:5] | filter[4:2] | (spi3w_en[0] = 0)
        let cfg = ((config.standby_time as u8) << 5) | ((config.filter as u8) << 2);
        i2c.write(self.addr, &[REG_CONFIG, cfg])?;

        Ok(())
    }

    /// Trigger a single measurement in forced mode.
    /// Must call this before reading temperature/pressure when in forced mode.
    pub fn trigger_measurement<I2C, E>(&mut self, i2c: &mut I2C, config: &Config) -> Result<(), E>
    where
        I2C: Write<Error = E>,
    {
        let ctrl = ((config.temp_oversampling as u8) << 5)
            | ((config.press_oversampling as u8) << 2)
            | (PowerMode::Forced as u8);
        i2c.write(self.addr, &[REG_CTRL_MEAS, ctrl])
    }

    /// Read the compensated temperature.
    ///
    /// Returns temperature in hundredths of a degree Celsius (e.g., 2508 = 25.08°C).
    /// In normal mode the sensor updates automatically.
    /// In forced mode you must call `trigger_measurement` first and wait for conversion.
    pub fn read_temperature<I2C, E>(&mut self, i2c: &mut I2C) -> Result<i32, E>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>,
    {
        let raw = self.read_raw_temp(i2c)?;
        let (t, t_fine) = compensate_temperature(raw, &self.calib);
        self.t_fine = t_fine;
        Ok(t)
    }

    /// Read the compensated pressure.
    ///
    /// Returns pressure in Pascals.
    /// Must call `read_temperature` first (or at least once) so `t_fine` is valid.
    pub fn read_pressure<I2C, E>(&mut self, i2c: &mut I2C) -> Result<i32, E>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>,
    {
        let raw = self.read_raw_press(i2c)?;
        Ok(compensate_pressure(raw, self.t_fine, &self.calib))
    }

    fn read_raw_temp<I2C, E>(&self, i2c: &mut I2C) -> Result<i32, E>
    where
        I2C: WriteRead<Error = E>,
    {
        let mut buf = [0u8; 3];
        i2c.write_read(self.addr, &[REG_TEMP_MSB], &mut buf)?;
        // 20-bit value: buf[0]<<12 | buf[1]<<4 | buf[2]>>4
        Ok(((buf[0] as i32) << 12) | ((buf[1] as i32) << 4) | ((buf[2] as i32) >> 4))
    }

    fn read_raw_press<I2C, E>(&self, i2c: &mut I2C) -> Result<i32, E>
    where
        I2C: WriteRead<Error = E>,
    {
        let mut buf = [0u8; 3];
        i2c.write_read(self.addr, &[REG_PRESS_MSB], &mut buf)?;
        Ok(((buf[0] as i32) << 12) | ((buf[1] as i32) << 4) | ((buf[2] as i32) >> 4))
    }
}

// --- Compensation formulas from Bosch datasheet section 3.11.3 ---

fn compensate_temperature(raw: i32, calib: &Calib) -> (i32, i32) {
    let var1 = (((raw >> 3) - ((calib.dig_t1 as i32) << 1)) * (calib.dig_t2 as i32)) >> 11;
    let var2 = (((((raw >> 4) - (calib.dig_t1 as i32))
        * ((raw >> 4) - (calib.dig_t1 as i32)))
        >> 12)
        * (calib.dig_t3 as i32))
        >> 14;
    let t_fine = var1 + var2;
    let t = (t_fine * 5 + 128) >> 8;
    (t, t_fine)
}

fn compensate_pressure(raw: i32, t_fine: i32, calib: &Calib) -> i32 {
    let t_fine = t_fine as i64;

    let var1 = t_fine - 128_000;
    let var2 = var1 * var1 * (calib.dig_p6 as i64);
    let var2 = var2 + ((var1 * (calib.dig_p5 as i64)) << 17);
    let var2 = var2 + ((calib.dig_p4 as i64) << 35);

    let var1 = ((var1 * var1 * (calib.dig_p3 as i64)) >> 8)
        + ((var1 * (calib.dig_p2 as i64)) << 12);
    let var1 = (((1i64 << 47) + var1) * (calib.dig_p1 as i64)) >> 33;

    if var1 == 0 {
        return 0;
    }

    let p = 1_048_576i64 - (raw as i64);
    let p = (((p << 31) - var2) * 3125) / var1;
    let var1 = ((calib.dig_p9 as i64) * (p >> 13) * (p >> 13)) >> 25;
    let var2 = ((calib.dig_p8 as i64) * p) >> 19;
    let p = ((p + var1 + var2) >> 8) + ((calib.dig_p7 as i64) << 4);

    // Q24.8 → Pa: divide by 256
    (p >> 8) as i32
}
