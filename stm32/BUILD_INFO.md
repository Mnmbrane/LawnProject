# STM32 Firmware Build/Flash Quick Reference

## Build And Run (SWD/probe-rs)

- Build + create `firmware.bin`:

```bash
cargo build-firmware
```

- Build and run via `probe-rs` runner:

```bash
cargo run-firmware
```

## USB DFU Flash (no debug probe)

1. Put the board in ROM DFU mode (BOOT0 high, reset/power cycle as required by board wiring).
2. Generate the `.bin`:

```bash
cargo build-firmware
```

3. Flash with `dfu-util`:

```bash
dfu-util -a 0 -s 0x08000000:leave -D firmware.bin
```

## Notes

- `probe-rs` is for SWD debug/programming. It does not replace STM32 ROM USB DFU mode.
- `firmware.rs` is currently copied from the `blinky` test so the command works now.
