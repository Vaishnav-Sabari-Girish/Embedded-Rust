//! MPU9250 + AK8963 bare-metal driver
//!
//! I2C register-level driver for the TDK InvenSense MPU9250 nine-axis sensor
//! (3-axis gyro + 3-axis accel in the MPU9250, 3-axis magnetometer in the AK8963).
//!
//! Datasheets:
//!   - MPU-9250 Register Map (PS-MPU-9250A-00)
//!   - AK8963 datasheet

use embedded_hal::blocking::i2c::{Write, WriteRead};

// ======== I2C addresses ========
const MPU_ADDR: u8 = 0x68;
const AK8963_ADDR: u8 = 0x0C;

// ======== MPU9250 registers ========
const REG_WHO_AM_I: u8 = 0x75;
const REG_PWR_MGMT_1: u8 = 0x6B;
const REG_PWR_MGMT_2: u8 = 0x6C;
const REG_SMPLRT_DIV: u8 = 0x19;
const REG_CONFIG: u8 = 0x1A;
const REG_GYRO_CONFIG: u8 = 0x1B;
const REG_ACCEL_CONFIG: u8 = 0x1C;
const REG_ACCEL_CONFIG2: u8 = 0x1D;
const REG_INT_PIN_CFG: u8 = 0x37;
const REG_ACCEL_XOUT_H: u8 = 0x3B;
const REG_TEMP_OUT_H: u8 = 0x41;
const REG_GYRO_XOUT_H: u8 = 0x43;

const MPU_WHO_AM_I_VAL: u8 = 0x71;

// ======== AK8963 registers ========
const AK_WIA: u8 = 0x00;
const AK_ST1: u8 = 0x02;
const AK_HXL: u8 = 0x03;
const AK_ST2: u8 = 0x09;
const AK_CNTL1: u8 = 0x0A;
const AK_CNTL2: u8 = 0x0B;
const AK_ASAX: u8 = 0x10;

const AK_WHO_AM_I_VAL: u8 = 0x48;

/// Gyroscope full-scale range.
#[derive(Copy, Clone)]
pub enum GyroScale {
    /// ±250 °/s
    Dps250 = 0,
    /// ±500 °/s
    Dps500 = 1,
    /// ±1000 °/s
    Dps1000 = 2,
    /// ±2000 °/s
    Dps2000 = 3,
}

impl GyroScale {
    /// LSB per °/s (conversion factor).
    fn lsb_per_dps(&self) -> f32 {
        match self {
            GyroScale::Dps250 => 131.0,
            GyroScale::Dps500 => 65.5,
            GyroScale::Dps1000 => 32.8,
            GyroScale::Dps2000 => 16.4,
        }
    }
}

/// Accelerometer full-scale range.
#[derive(Copy, Clone)]
pub enum AccelScale {
    /// ±2 g
    G2 = 0,
    /// ±4 g
    G4 = 1,
    /// ±8 g
    G8 = 2,
    /// ±16 g
    G16 = 3,
}

impl AccelScale {
    /// LSB per g (conversion factor).
    fn lsb_per_g(&self) -> f32 {
        match self {
            AccelScale::G2 => 16384.0,
            AccelScale::G4 => 8192.0,
            AccelScale::G8 => 4096.0,
            AccelScale::G16 => 2048.0,
        }
    }
}

/// DLPF bandwidth config for gyroscope and temperature sensor.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum DlpfConfig {
    /// 250 Hz bandwidth, 0.97 ms delay (Fs=8kHz)
    Bw250 = 0,
    /// 184 Hz
    Bw184 = 1,
    /// 92 Hz
    Bw92 = 2,
    /// 41 Hz
    Bw41 = 3,
    /// 20 Hz
    Bw20 = 4,
    /// 10 Hz
    Bw10 = 5,
    /// 5 Hz
    Bw5 = 6,
    /// 3600 Hz bandwidth, 0.17 ms delay (Fs=8kHz)
    Bw3600 = 7,
}

/// MPU9250 configuration struct.
pub struct Config {
    pub gyro_scale: GyroScale,
    pub accel_scale: AccelScale,
    pub dlpf: DlpfConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            gyro_scale: GyroScale::Dps250,
            accel_scale: AccelScale::G2,
            dlpf: DlpfConfig::Bw41,
        }
    }
}

/// Raw sensor readings.
#[derive(Debug, Clone, Default)]
pub struct RawReadings {
    pub accel: [i16; 3],
    pub temp: i16,
    pub gyro: [i16; 3],
}

/// Scaled sensor readings (floating-point).
#[derive(Debug, Clone, Default)]
pub struct ScaledReadings {
    /// Acceleration in g.
    pub accel: [f32; 3],
    /// Gyroscope rate in °/s.
    pub gyro: [f32; 3],
    /// Temperature in °C.
    pub temp: f32,
    /// Magnetometer in µT.
    pub mag: [f32; 3],
}

/// AK8963 magnetometer sensitivity adjustment factors (from on-chip fuse ROM).
#[derive(Debug, Clone, Default)]
pub struct Asa {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// MPU9250 driver (includes AK8963 magnetometer).
pub struct Mpu9250 {
    gyro_scale: GyroScale,
    accel_scale: AccelScale,
    asa: Asa,
}

impl Mpu9250 {
    /// Initialise the MPU9250 and the on-board AK8963.
    ///
    /// Takes `&mut I2C` — does not own the bus, so it can be shared
    /// with BMP280 or other I2C devices.
    pub fn new<I2C, E>(i2c: &mut I2C, config: &Config) -> Result<Self, E>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>,
        E: core::fmt::Debug,
    {
        // 1. Verify MPU9250 WHO_AM_I
        let mut whoami = [0u8];
        i2c.write_read(MPU_ADDR, &[REG_WHO_AM_I], &mut whoami)?;
        if whoami[0] != MPU_WHO_AM_I_VAL {
            panic!(
                "MPU9250 WHO_AM_I mismatch: expected 0x{:02X}, got 0x{:02X}",
                MPU_WHO_AM_I_VAL, whoami[0]
            );
        }

        // 2. Wake up: clear sleep bit, select PLL with x-axis gyro reference
        //    PWR_MGMT_1: CLKSEL = 0b001 (PLL), SLEEP = 0, TEMP_DIS = 0
        i2c.write(MPU_ADDR, &[REG_PWR_MGMT_1, 0x01])?;

        // 3. Set sample rate divider to 0 => 1 kHz gyro output rate (when DLPF on)
        i2c.write(MPU_ADDR, &[REG_SMPLRT_DIV, 0x00])?;

        // 4. DLPF config
        i2c.write(MPU_ADDR, &[REG_CONFIG, config.dlpf as u8])?;

        // 5. Gyro config (FS_SEL)
        i2c.write(MPU_ADDR, &[REG_GYRO_CONFIG, (config.gyro_scale as u8) << 3])?;

        // 6. Accel config (AFS_SEL)
        i2c.write(MPU_ADDR, &[REG_ACCEL_CONFIG, (config.accel_scale as u8) << 3])?;

        // 7. Accel config 2: set DLPF for accel to match gyro DLPF
        i2c.write(MPU_ADDR, &[REG_ACCEL_CONFIG2, config.dlpf as u8])?;

        // 8. Enable I2C bypass to access AK8963
        i2c.write(MPU_ADDR, &[REG_INT_PIN_CFG, 0x02])?;

        // 9. Init AK8963 and read ASA
        let asa = init_ak8963(i2c)?;

        Ok(Mpu9250 {
            gyro_scale: config.gyro_scale,
            accel_scale: config.accel_scale,
            asa,
        })
    }

    /// Read raw un-scaled sensor values.
    ///
    /// Reads 14 bytes starting at ACCEL_XOUT_H (0x3B):
    ///   accel[0..6], temp[0..2], gyro[0..6]
    pub fn read_raw<I2C, E>(&self, i2c: &mut I2C) -> Result<RawReadings, E>
    where
        I2C: WriteRead<Error = E>,
    {
        let mut buf = [0u8; 14];
        i2c.write_read(MPU_ADDR, &[REG_ACCEL_XOUT_H], &mut buf)?;

        Ok(RawReadings {
            accel: [
                i16::from_be_bytes([buf[0], buf[1]]),
                i16::from_be_bytes([buf[2], buf[3]]),
                i16::from_be_bytes([buf[4], buf[5]]),
            ],
            temp: i16::from_be_bytes([buf[6], buf[7]]),
            gyro: [
                i16::from_be_bytes([buf[8], buf[9]]),
                i16::from_be_bytes([buf[10], buf[11]]),
                i16::from_be_bytes([buf[12], buf[13]]),
            ],
        })
    }

    /// Read scaled accel (g), gyro (°/s), temperature (°C).
    pub fn read_scaled<I2C, E>(&self, i2c: &mut I2C) -> Result<ScaledReadings, E>
    where
        I2C: WriteRead<Error = E>,
    {
        let raw = self.read_raw(i2c)?;

        let accel_lsb = self.accel_scale.lsb_per_g();
        let gyro_lsb = self.gyro_scale.lsb_per_dps();

        // Temperature in °C = (TEMP_OUT / 333.87) + 21.0
        // (per datasheet: Temp = ((TEMP_OUT - RoomTemp_Offset) / Temp_Sensitivity) + 21°C
        //  RoomTemp_Offset = 0, Temp_Sensitivity = 333.87)
        let temp = (raw.temp as f32) / 333.87 + 21.0;

        Ok(ScaledReadings {
            accel: [
                raw.accel[0] as f32 / accel_lsb,
                raw.accel[1] as f32 / accel_lsb,
                raw.accel[2] as f32 / accel_lsb,
            ],
            gyro: [
                raw.gyro[0] as f32 / gyro_lsb,
                raw.gyro[1] as f32 / gyro_lsb,
                raw.gyro[2] as f32 / gyro_lsb,
            ],
            temp,
            mag: [0.0; 3],
        })
    }

    /// Read magnetometer data (µT).
    ///
    /// Reads AK8963 via MPU9250 bypass.
    /// Must be called while bypass is enabled (set during init).
    pub fn read_mag<I2C, E>(&self, i2c: &mut I2C) -> Result<[f32; 3], E>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>,
    {
        // Check data ready via ST1
        let mut st1 = [0u8];
        i2c.write_read(AK8963_ADDR, &[AK_ST1], &mut st1)?;
        if st1[0] & 0x01 == 0 {
            // Data not ready; return last or zero
            return Ok([0.0; 3]);
        }

        // Read 6 data bytes + ST2 byte
        let mut buf = [0u8; 7];
        i2c.write_read(AK8963_ADDR, &[AK_HXL], &mut buf)?;

        let st2 = buf[6];
        // If ST2 bit 3 (HOFL) is set, magnetic sensor overflow → data invalid
        if st2 & 0x08 != 0 {
            return Ok([0.0; 3]);
        }

        let raw_x = i16::from_le_bytes([buf[0], buf[1]]);
        let raw_y = i16::from_le_bytes([buf[2], buf[3]]);
        let raw_z = i16::from_le_bytes([buf[4], buf[5]]);

        // 16-bit resolution: 0.15 µT per LSB
        // Apply ASA adjustment: H_adj = H_raw * ((ASA - 128) * 0.5 / 128 + 1)
        let mx = raw_x as f32 * 0.15 * self.asa.x;
        let my = raw_y as f32 * 0.15 * self.asa.y;
        let mz = raw_z as f32 * 0.15 * self.asa.z;

        Ok([mx, my, mz])
    }

    /// Read everything: accel, gyro, temp, mag — scaled.
    pub fn read_all<I2C, E>(&self, i2c: &mut I2C) -> Result<ScaledReadings, E>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>,
    {
        let mut readings = self.read_scaled(i2c)?;
        if let Ok(mag) = self.read_mag(i2c) {
            readings.mag = mag;
        }
        Ok(readings)
    }
}

/// Initialise AK8963 and read ASA factors.
fn init_ak8963<I2C, E>(i2c: &mut I2C) -> Result<Asa, E>
where
    I2C: WriteRead<Error = E> + Write<Error = E>,
    E: core::fmt::Debug,
{
    // 1. Power down AK8963
    i2c.write(AK8963_ADDR, &[AK_CNTL1, 0x00])?;

    // 2. Verify AK8963 WHO_AM_I
    let mut wia = [0u8];
    i2c.write_read(AK8963_ADDR, &[AK_WIA], &mut wia)?;
    if wia[0] != AK_WHO_AM_I_VAL {
        panic!(
            "AK8963 WHO_AM_I mismatch: expected 0x{:02X}, got 0x{:02X}",
            AK_WHO_AM_I_VAL, wia[0]
        );
    }

    // 3. Enter Fuse ROM access mode to read sensitivity adjustment data
    i2c.write(AK8963_ADDR, &[AK_CNTL1, 0x0F])?;

    let mut asa_bytes = [0u8; 3];
    i2c.write_read(AK8963_ADDR, &[AK_ASAX], &mut asa_bytes)?;

    let asa = Asa {
        // formula: H_adj = H_raw * ((ASA - 128) * 0.5 / 128 + 1)
        x: (asa_bytes[0] as f32 - 128.0) * 0.5 / 128.0 + 1.0,
        y: (asa_bytes[1] as f32 - 128.0) * 0.5 / 128.0 + 1.0,
        z: (asa_bytes[2] as f32 - 128.0) * 0.5 / 128.0 + 1.0,
    };

    // 4. Power down
    i2c.write(AK8963_ADDR, &[AK_CNTL1, 0x00])?;

    // 5. Set to Continuous Measurement Mode 2 (100 Hz), 16-bit output
    i2c.write(AK8963_ADDR, &[AK_CNTL1, 0x16])?;

    Ok(asa)
}
