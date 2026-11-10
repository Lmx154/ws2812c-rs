# WS2812C-2020 device facts (MARV V2 Rev 1)

Copied from the MARV V2 firmware repository's `docs/devices.md` and `docs/pinout.md`. Page numbers are PDF pages of the datasheet listed at the end. Items marked UNKNOWN or "unverified" have not been checked against the datasheet or on hardware. "Measured" means measured on this board.

## WS2812C-2020 status LED (D20)

Pins: see "Pins" below.

- One LED, data on GPIO43 through a 100 ohm series resistor, supplied from V5_SYS, driven by a PIO program in
  the GPIO16-47 window.
- Fitted part: WS2812C-2020-V6, same pinout and pad drawing as the WS2812C-2020-V1 datasheet.
- Datasheet p4 limits: T0H 220-380 ns; T1H, T0L, T1L 580-1000 ns; reset low over 280 us; GRB, MSB first.
- SPI encoding of the driver: 0 = `1000`, 1 = `1100`, so the SPI clock must be 3.0 to 3.448 MHz (derived from
  the limits; nominal 3.2 MHz, which embassy-rp rounds to 3.125 MHz).
- Measured on the board, PIO on GPIO43 at 150 MHz (divider 47): T0H 306-320, T1H 626, T0L 940, T1L 620-633 ns;
  the 24 decoded bits equal the sent word.
- Measured on the board, SPI1 (TX GPIO43, SCK on the unconnected GPIO26) at 3.125 MHz: mode 1 or 3 is gapless
  (T0H 320, T1H 640, T0L 953-966, T1L 633-646 ns); modes 0 and 2 leave about 0.5 us between bytes and break the
  limits. embassy-rp's blocking SPI write waits for each byte and is not gapless either.
- Measured on the board: low time between two frames 318-356 us (PIO) and 345 us (SPI).
- The pad input reads the driven waveform from another PIO block, so the line is checked without extra hardware.
- Not verified in the driver's runs: the colours as seen by a person (the bring-up run was watched: red, green,
  blue, white).

## Pins (from pinout.md)

All GPIO are 3.3 V logic and are not 5 V tolerant.

| GPIO | Net | Function used | Status |
|---|---|---|---|
| 43 | LED_DATA | PIO (WS2812C) | status LED D20 through 100 ohm R55 |

| Peripheral | GPIOs | Device |
|---|---|---|
| Status LED | 43 | PIO, WS2812C |

| Item | Value |
|---|---|
| Status LED | WS2812C-2020 D20, data on GPIO43 through 100 ohm R55, powered from V5_SYS |

GPIO43 is in the GPIO16-47 PIO window. The LED and ESC PIO signals need the same window, so every PIO user shares GPIOBASE = 16.

## Datasheet

| Device | URL |
|---|---|
| WS2812C-2020 | https://www.lcsc.com/datasheet/lcsc_datasheet_2202221130_Worldsemi-WS2812C-2020-V1_C2976072.pdf |
