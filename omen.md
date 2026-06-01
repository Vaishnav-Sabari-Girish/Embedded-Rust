# Omen Project Notes — Embedded Rust Workspace

## Workspace Root
`~/Desktop/My_Projects/Embedded-Rust`

## Directory Structure

```
microcontrollers/
├── esp32/                  # ESP32 (Xtensa) — esp-rs toolchain
├── glyph_c6/               # ESP32-C6 (RISC-V) — esp-rs toolchain
├── microBit/               # nRF52833 (BBC micro:bit V2) — bare metal / PAC / HAL / BSP
├── nrf52840_dk/            # nRF52840-DK (PCA10056) — Embassy async
└── rp2040/                 # RP2040-Zero — elf2uf2-rs / flip-link
templates/
└── microBit/               # Cargo templates + linker script for micro:bit projects
```

## Toolchain (per session, resolved at start)

- **arm-none-eabi-gdb:** `~/.omen/tools/arm-none-eabi-gcc/xpack-arm-none-eabi-gcc-15.2.1-1.1-linux-x64/bin/arm-none-eabi-gdb-py3`
- **OpenOCD:** `~/.omen/tools/openocd/xpack-openocd-0.12.0-7-linux-x64/bin/openocd`

---

## Microcontroller Families

### 1. nRF52840-DK (PCA10056) — `microcontrollers/nrf52840_dk/`

**Chip:** nRF52840 (Cortex-M4F)

**Target:** `thumbv7em-none-eabi`

**Runner:** `probe-rs run --chip nRF52840_xxAA`

**GDB:** OpenOCD with `jlink` interface, `nrf52` target → `localhost:3333`

**Projects:**

| Project | Description |
|---|---|
| `blinky/` | LED blink |
| `nrf_button_press/` | All 4 buttons + LEDs |
| `die_temperature_sensor/` | Internal temp sensor |
| `pwm_led/` | LED PWM |
| `oled_display/` | SH1106 OLED via I2C |
| `oled_ratatui/` | OLED with ratatui UI framework |
| `gy_91_sensor/` | GY-91 module (BMP280 + MPU9250) on I2C |

**Build:**
```bash
cd microcontrollers/nrf52840_dk/<project>
cargo build --release
cargo run --release              # build + flash via probe-rs
./build-size.sh .                # size analysis (from project dir)
```

**Key dependencies (Embassy):**
- `embassy-nrf` (time-driver-rtc1, gpiote, defmt)
- `embassy-executor` (platform-cortex-m, executor-thread)
- `embassy-time` (tick-hz-32_768)
- `defmt` / `defmt-rtt` for logging
- `cortex-m` / `cortex-m-rt`
- `static_cell` for static buffers

**Logging:** `DEFMT_LOG = "trace"` in `.cargo/config.toml`. defmt-rtt transport, probe-rs handles RTT.

**Known issues:**
- embedded-hal 0.2.x vs 1.0.x trait mismatch between driver crates (eh0.2) and embassy-nrf (eh1.0)
- BMP280 driver borrows I2C per-call; MPU9250 takes ownership

---

### 2. RP2040-Zero — `microcontrollers/rp2040/`

**Chip:** RP2040 (Cortex-M0+)

**Target:** `thumbv6m-none-eabi`

**Runner:** `elf2uf2-rs deploy --family rp2040`

**Linker:** `flip-link` with `--nmagic`, `-Tlink.x`, `no-vectorize-loops`

**Flashing (manual UF2):**
```bash
# 1. Enter BOOT mode (hold BOOT, press RESET, release BOOT)
# 2. Build
cargo build --release
# 3. Convert to UF2
elf2uf2-rs convert ../../target/thumbv6m-none-eabi/release/<binary> flash.uf2
# 4. Mount & copy (device may vary — use lsblk)
sudo mount -t vfat -o sync /dev/sda1 /mnt/rp2
sudo cp flash.uf2 /mnt/rp2/
sudo umount /mnt/rp2/
```

**Note:** The `cargo run` runner (`elf2uf2-rs deploy`) automates the UF2 flash if the board is in BOOT mode and mounted.

**Projects:**

| Project | Description |
|---|---|
| `led_flash/` | LED blink |
| `uart_serial_monitor/` | UART serial output |
| `internal_temp_sensor/` | Internal temperature sensor |
| `e-paper-display/` | 1.54" E-Paper display |
| `e-paper-display-ratatui/` | E-Paper with ratatui |
| `e-paper-big-text/` | E-Paper with tui-big-text |
| `e-paper-display-ascii/` | E-Paper ASCII text |
| `conway_game_of_life/` | Conway's Game of Life on E-Paper |

**Utility scripts:**
- `build-size.sh` — size analysis
- `clean_cargo.sh` — clean build artifacts

---

### 3. ESP32 (Xtensa) — `microcontrollers/esp32/`

**Chip:** ESP32 (Xtensa LX6)

**Target:** `xtensa-esp32-none-elf`

**Runner:** `espflash flash --monitor --chip esp32`

**Build-std:** `build-std = ["core"]` (unstable)

**Rustflags:** `-C link-arg=-nostartfiles`

**Projects:**

| Project | Description |
|---|---|
| `hello_world/` | Hello world via UART |
| `led_blink/` | LED blink |
| `led_blink_type_2/` | LED blink (variant) |
| `button_press/` | Button input |
| `led_pwm/` | LED PWM |
| `sensor_reading/` | Multiple sensors: BME280, DHT22, touch, ultrasonic |
| `tft_display_hello_world/` | TFT text |
| `tft_display_image/` | TFT image |
| `tft_display_text_new/` | TFT text (variant) |
| `tft_display_image_new/` | TFT image (variant) |
| `hello-rat/` | ratatui on ESP32 |

**Note:** ESP32 projects use `src/bin/main.rs` entry points (not `src/main.rs`).

---

### 4. ESP32-C6 (RISC-V) — `microcontrollers/glyph_c6/`

**Chip:** ESP32-C6-MINI (RISC-V IMAC)

**Target:** `riscv32imac-unknown-none-elf`

**Runner:** `espflash flash --monitor --chip esp32c6`

**Build-std:** `build-std = ["core"]` (unstable)

**Rustflags:** `-C force-frame-pointers`

**Projects:**

| Project | Description |
|---|---|
| `blinky/` | LED blink |

---

### 5. micro:bit V2 (nRF52833) — `microcontrollers/microBit/`

**Chip:** nRF52833 (Cortex-M4F)

**Target:** `thumbv7em-none-eabihf` (hard-float)

**Linker:** `-Tlink.x`

**No runner** configured — flash manually via OpenOCD / pyOCD / DAPLink.

**Projects:**

| Project | Description |
|---|---|
| `hello_world/` | Hello world via RTT/semihosting |
| `led_blink/blinky_bare_metal/` | Bare metal register-level blink |
| `led_blink/blinky_pac/` | Peripheral Access Crate blink |
| `led_blink/blinky_hal/` | HAL blink |
| `led_blink/blinky_bsp/` | Board Support Package blink |
| `button_programs/button_press/` | Button input |

**Templates** in `templates/microBit/`:
- `cargo_template_microbit.toml` — Cargo.toml template
- `embed_template_microbit.toml` — embed.toml (probe config)
- `memory_microbit.x` — linker script

**Schematic:** `microBit_V2.0.0_S_schematic.pdf` (in microBit project root)

---

## Conventions (workspace-wide)

- Edition 2024 where supported
- `no_std` + `no_main` for bare-metal / Embassy projects
- `defmt` for logging on nRF52/RP2040
- Embassy async executor (thread mode) for nRF52840-DK
- ESP projects use `esp-println` / standard logging
- Release builds: `opt-level = 'z'`, LTO, debug symbols (`debug = 2`)

## Datasheets

No datasheets directory set up yet. Use `/datasheet <path-to-pdf> [part-number]` to import.

## GDB Debugging

- **nRF52840-DK:** OpenOCD server with `interface: jlink`, `target: nrf52`, port `3333`
- Use `gdb_openocd_start` to start server, then `gdb_session_open` with `kind: "remote"` and `remote: "localhost:3333"`
- Use `gdb_stack_inspect` first for any crash/debug session
