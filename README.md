# ws2812c

Driver for the Worldsemi WS2812C-2020 RGB LED (one wire, 24 bits per LED, GRB order), the status LED D20 of the MARV V2 Rev 1 flight controller (RP2354B). It is built on the `embedded-hal` 1.0 traits only: it works with any executor or none. The crate root is the async driver. The `blocking` module is the same driver for blocking buses. The driver does colour handling only; the line goes through a `Bus`, with two transports in the crate. All LED knowledge (colours, the wire word, the datasheet timing limits, the SPI encoding and clock window, the PIO timing calculation) is pure code tested on the host. Patterns, blinking and fading are left to the application.

## Usage

Async, PIO (feature `embassy-rp`):

```rust
let Pio { mut common, sm0, .. } = Pio::new(p.PIO1, Irqs);
let bus = ws2812c::pio::PioTransport::new(&mut common, sm0, p.PIN_43)?;
let mut led = ws2812c::Ws2812c::new(bus);
led.set_brightness(32);
led.set(ws2812c::Rgb::GREEN).await?;
```

Async, SPI:

```rust
let mut spi_cfg = embassy_rp::spi::Config::default();
spi_cfg.frequency = ws2812c::spi::SPI_NOMINAL_HZ;
let spi = Spi::new_txonly(p.SPI1, p.PIN_26, p.PIN_43, p.DMA_CH0, Irqs, spi_cfg);
let mut led = ws2812c::Ws2812c::new(ws2812c::spi::SpiTransport::new(spi));
led.set_brightness(32);
led.set(ws2812c::Rgb::GREEN).await?;
```

Blocking:

```rust
let mut led = ws2812c::blocking::Ws2812c::new(ws2812c::spi::SpiTransport::new(spi_bus));
led.set(ws2812c::Rgb::WHITE)?;
```

An LED at full level is very bright and draws about 15 mA. `set_brightness` scales every colour of the following writes; the default is 255 (no scaling).

## What the application provides

- A `Bus` for the data line. Two are included:
  - `pio::PioTransport` (feature `embassy-rp`): one PIO state machine, any GPIO, no DMA needed for one LED.
  - `spi::SpiTransport`: any `embedded-hal` SPI bus at 3.0 to 3.45 MHz with a gapless stream. On the RP2350 that means SPI mode 1 or 3 and the async (DMA) write; modes 0 and 2 break the LED's timing.
- The pause the LED needs between frames (over 280 us) is kept by the transport; the caller never waits.
- For the PIO transport: GPIO43 is reachable only with the PIO block set to GPIO16-47, so every other state machine in that block must use pins in that range too (the SD bus does).

Dependencies: `embedded-hal` 1.0, `embedded-hal-async` 1.0 and, optionally, `defmt`, `embassy-rp` and `embassy-time`.

## Cargo features

All features are off by default. Without them the crate is hardware-independent `no_std` code.

| Feature | What it adds |
|---|---|
| `defmt` | `defmt::Format` for the public data and error types |
| `embassy-rp` | the PIO transport for embassy-rp; the application picks the chip feature (`rp235xa` or `rp235xb`) |

## Measured on the MARV V2 Rev 1 board

- Every bit time is inside the datasheet limits on both transports.
- PIO on GPIO43 at 150 MHz (divider 47): T0H 306-320, T1H 626, T0L 940, T1L 620-633 ns; the 24 decoded bits equal the sent word.
- SPI1 (TX GPIO43, SCK on the unconnected GPIO26) at 3.125 MHz: mode 1 or 3 is gapless (T0H 320, T1H 640, T0L 953-966, T1L 633-646 ns); modes 0 and 2 leave about 0.5 us between bytes and break the limits. embassy-rp's blocking SPI write waits for each byte and is not gapless either.
- Low time between two frames: 318-356 us (PIO) and 345 us (SPI).

## Tests

On the host:

    cargo test

On the board, through a debug probe (the script takes a lock, so runs never overlap):

    tools/run.sh alive
    tools/run.sh ws2812c_smoke
    tools/run.sh ws2812c_colours 600    # for a person watching the LED, about 36 s

Used here: Rust 1.99.1 with the `thumbv8m.main-none-eabihf` target, probe-rs 0.31.0, and a debug probe on the SWD header J10.

| Binary | What it checks |
|---|---|
| `alive` | tool chain, probe and board |
| `ws2812c_smoke` | the waveform on the LED pin for both transports against the datasheet limits, the pause between frames, the blocking front-end; red, green, blue, white for a person to watch; then a PIO send dropped at each of its await points (the next frame must be right and come after a pause above 280 us), and the CPU time of every poll of `send` on both transports against a 50 us budget |
| `ws2812c_colours` | nothing by itself: it shows red, green, blue and white for 4 s each, then dark for 2 s, twice, and prints the colour and the bytes sent before each, so a person can compare the LED with the log |

The exit status is 0 when every check passed. Full logs are kept under `hil/logs/`.

Pins the test uses: data on GPIO43. The pad input reads the driven waveform from another PIO block, so the line is checked without extra hardware. The SPI part of the test uses SPI1 with TX on GPIO43 and SCK on the unconnected GPIO26.

## Status

Version 0.1.0, as tested on the Rev 1 board.

- Host: `cargo test` passes. Clippy, rustdoc and rustfmt are clean for the driver crate, and every public item is documented.
- Board: `alive` and `ws2812c_smoke` exited 0 when last run. Measured by it (2026-11-11): after a dropped PIO send the next frame is correct, with a pause of 316 to 674 us before it; the longest single poll of `send` over 128 sends is 20 to 25 us on both transports (median 2.4 us on PIO, 20 us on SPI); the very first send after start-up takes longer in one poll, 37 to 39 us on PIO and 65 to 68 us on SPI, which is reported and not held to the budget.
- Status LED: nobody has confirmed the colours by eye yet. The waveform on the pin is measured against the datasheet limits, which cannot show colour or the order of the colour bytes. `ws2812c_colours` is the check for a person: run it and compare the LED with the lines it prints.

Not done for this device:

- Patterns, long chains on SPI in one gapless write, DMA for long chains on PIO.

## Layout

```
Cargo.toml       the driver crate (its own workspace root)
src/             the driver
hil/             the on-board test: one package, its own workspace; the only place that uses embassy-rp
  src/lib.rs     reports the verdict to the host
  src/bin/       alive.rs, ws2812c_smoke.rs and ws2812c_colours.rs
  build.rs       links memory.x
tools/run.sh     builds, flashes and runs one on-board test
docs/device.md   timing and measured facts, pins, datasheet
LICENSE          MIT
```

## License

MIT, see `LICENSE`. The driver is written from the datasheet listed in `docs/device.md`.
