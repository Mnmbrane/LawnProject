# STM32 Firmware Build/Flash Quick Reference

## Build And Run (SWD/probe-rs)

```bash
cargo run --release --bin app
```

Alias:

```bash
cargo run-firmware
```

## USB DFU Flash (no debug probe)

1. Put the board in ROM DFU mode (BOOT0 high, reset/power cycle as required by board wiring).
2. Generate `app.bin`:

```bash
cargo build-firmware
```

1. Flash with `dfu-util`:

```bash
dfu-util -a 0 -s 0x08000000:leave -D app.bin
```
