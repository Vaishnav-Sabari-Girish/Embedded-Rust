---
title: "**Embedded Rust Programming**"
sub_title: With MicroBit V2
author: Vaishnav Sabari Girish
---

# Getting started 

## Hardware Requirements 

1. MicroBit V2 Board
2. Computer with Rust installed (preferably Linux-based OS)
3. USB Cable

<!--end_slide-->

## Setting up environment - 1

### Adding the architecture

```bash
rustup target add thumbv7em-none-eabihf
```

Check it with 

```bash
rustup show
```

After you have added the architecture, install `probe-rs` and `cargo-binutils` using the following commands

```bash
cargo install cargo-binutils

curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh
```

<!--end_slide-->

## Output of `rustup show`

```bash +exec 
rustup show
```