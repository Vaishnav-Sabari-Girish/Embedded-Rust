# RP2040 Embedded Rust Programs

A collection of Embedded Rust programs using the RP2040-Zero microcontroller.

## How to Build & Flash

### 1. One-Time Setup

Install the UF2 converter tool:

```bash
cargo install elf2uf2-rs
```

Create a mount point for the RP2040:

```bash
sudo mkdir -p /mnt/rp2
```

### 2. Put the Board into BOOT Mode

The RP2040-Zero has dedicated BOOT and RESET buttons that allow you to enter
bootloader mode without unplugging the board:

1. Press and hold the **BOOT** button.
2. While holding BOOT, press and release the **RESET** button.
3. Keep holding BOOT for one more second, then release it.

The board should now appear as a USB mass storage device named **RPI-RP2**.

### 3. Build the Firmware

Navigate to the project directory and compile the code:

```bash
cargo build --release
```

The compiled binary will be located at:
`../../target/thumbv6m-none-eabi/release/rp2040-1in54-epd-example`

### 4. Convert to UF2 Format

Convert the ELF binary to UF2 format, which the RP2040 bootloader understands:

```bash
# The RP2040 bootloader only accepts UF2 files for flashing
elf2uf2-rs convert ../../target/thumbv6m-none-eabi/release/rp2040-1in54-epd-example flash.uf2
```

### 5. Flash the Firmware

Mount the RP2040 drive:

```bash
# Mount the RP2040 as a VFAT filesystem with synchronous writes
# The 'sync' option ensures all write operations complete immediately,
# preventing data corruption if the board reboots before buffered writes finish
sudo mount -t vfat -o sync /dev/sda1 /mnt/rp2
```

> **Note:** The device path may vary depending on your system. If you have
> multiple USB drives connected, use `lsblk` to identify the correct device
> (it may be `/dev/sdb1`, `/dev/sdc1`, etc.).

Copy the UF2 file to the mounted drive:

```bash
# The bootloader detects the file and flashes it automatically
sudo cp flash.uf2 /mnt/rp2/
```

Unmount the drive:

```bash
# Safely unmount the drive to ensure all writes are flushed
sudo umount /mnt/rp2/
```

The board will automatically reboot and start running your program.

## Programs

1. [Led Blink](./led_flash/src/main.rs)
2. [UART Serial Monitor](./uart_serial_monitor/src/main.rs)
3. [internal Temperature Sensor](./internal_temp_sensor/src/main.rs)
4. [E-Paper Display](./e-paper-display/src/main.rs)
5. [E-Paper Display `ratatui`](./e-paper-display-ratatui/src/main.rs)
6. [E-Paper Display `tui-big-text`](./e-paper-big-text/src/main.rs)
7. [E-Paper Display ASCII text](./e-paper-display-ascii/src/main.rs)
8. [Conway's Game of Life](./conway_game_of_life/src/main.rs)
