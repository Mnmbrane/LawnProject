# LawnProject

## Goals

Build a smart lawnmower controller that accepts compact motion commands from
a base station (in-house) and executes them reliably. The mower already has a
camera and should be able to communicate with the base station, but the
mower-side controls are the focus here.

## Mower Control Scope

The mower should accept discrete motion commands rather than a continuous
stream. Commands are relative to the mower's current heading.

Minimum command set:

- `MOVE(distance)`
- `TURN(angle)`
- `STOP`
- `PAUSE`
- `RESUME`
- `SET_SPEED(tier)`

Execution behavior:

- Commands execute sequentially with ACK/NACK + sequence numbers to prevent
  duplicates.
- Short command queue (1-3 items max) to keep behavior predictable.
- Safety overrides always win (emergency stop, obstacle trigger, lost comms
  timeout).

## Compact Protocol Sketch

Binary framing (compact, non-English, fixed fields):

```
[SYNC][TYPE][LEN][PAYLOAD...][CRC]
```

Field meanings:

- `SYNC`: start-of-frame marker so the receiver can align to a message.
- `TYPE`: command or response ID (MOVE, TURN, STOP, ACK, NACK, DONE, STATUS).
- `LEN`: payload size in bytes.
- `PAYLOAD`: command data (distance, angle, speed tier, or status codes).
- `CRC`: checksum (CRC-8 or CRC-16) to detect corrupted frames.

Optional but recommended:

- `SEQ`: sequence number in header or payload to prevent duplicates.
- `FLAGS`: priority or response-required bits (e.g., emergency stop).

Suggested payload fields:

- `MOVE`: int16 distance in centimeters
- `TURN`: int16 angle in tenths of degrees
- `SET_SPEED`: uint8 tier (0–3)

Reliability:

- Sequence number in header; receiver ACKs each command.
- Timeout on missing ACK; base station can retry.
- Mower stops if no valid command in N seconds.

## Control Pipeline (Mower Side)

Command input -> parser -> motion planner -> motor controller.

Motion planner converts `MOVE`/`TURN` into wheel velocities (PWM) with
feedback control. Safety layer can override any motion.

## Hardware Recommendations (Camera + Base Station Link)

Given you want a camera on the mower and a reliable link back to the base
station at low cost:

- **ESP32-S3 (Recommended)**: best low-cost choice with camera support, strong
  performance, built-in Wi-Fi/BLE. Commonly paired with OV2640/OV5640 modules.
  Good for control + basic vision or low-rate streaming.
- **ESP32-CAM**: very cheap and proven for camera use, but less headroom and
  limited I/O; still fine for basic streaming + control.
- **Raspberry Pi Zero 2 W**: more expensive but far more capable for vision
  workloads. Use this if you plan heavier on-board vision or higher quality
  streaming.

If you want the lowest price while keeping camera + comms reliable, choose
ESP32-S3 and keep image processing light. If you need real-time or heavier
vision, use Pi Zero 2 W.

## Hardware + Language

- Primary controller: ESP32-S3 for camera + base station comms.
- Motor control MCU: STM32 for drive chain, encoders, safety interlocks.
- Mower-side control code will be written in Rust.

Recommended pairing detail:

- STM32F411CEU6 is a good low-cost motor-control MCU (timers for PWM and
  encoders, 100 MHz core, 3.3V logic).
- Link ESP32-S3 <-> STM32 over UART for commands + telemetry (simple and
  reliable at this scale).

## Drive Layout

Chosen layout: 2-wheel drive (differential) with a front caster.

Why:
- Simple, low cost, and predictable control.
- Works well with MOVE/TURN command model.

Tradeoffs:
- Turn-in-place can scuff grass.
- Caster can wobble on uneven ground.

## UART vs SPI (ESP32-S3 <-> STM32)

UART:
- 3 wires (TX, RX, GND), simple wiring and debugging.
- Asynchronous, forgiving timing, reliable for command + telemetry.
- Plenty of bandwidth at typical baud rates (115200-921600).

SPI:
- 4+ wires (SCLK, MOSI, MISO, CS, GND).
- Faster and lower latency, but more sensitive to timing and wiring.
- Requires strict master/slave framing and is harder to debug.

Recommendation: use UART unless you truly need higher throughput.

## Telemetry

Telemetry is the mower's status data sent back to the base station. It is
needed to verify the mower is alive, safe, and executing commands correctly.

Recommended minimal telemetry (0.5-2s interval):
- Battery voltage
- Current state (IDLE/MOVING/TURNING/ERROR)
- Motor fault flag
- Camera fault flag
- GPS device fault flag
