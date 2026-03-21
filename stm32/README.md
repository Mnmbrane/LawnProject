# STM32F411 Firmware

Single-app Embassy firmware project for STM32F411CEU6.

## Run with probe-rs (SWD)

```bash
cargo run --release --bin app
```

Alias:

```bash
cargo run-firmware
```

## Build binary for DFU

```bash
cargo build-firmware
```

This produces `app.bin` in the project root.

## Flash with dfu-util

Put the board in DFU mode first (BOOT0 high, then reset), then:

```bash
dfu-util -a 0 -s 0x08000000:leave -D app.bin
```

After flashing, set BOOT0 low and reset again.
