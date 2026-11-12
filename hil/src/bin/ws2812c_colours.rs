//! Human-paced colour check of the status LED D20 (WS2812C-2020) on GPIO43, through the driver's
//! normal API (`Ws2812c` on `PioTransport`) at a moderate brightness.
//!
//! The waveform checks of `ws2812c_smoke` cannot see colour or the GRB byte order, so a person
//! compares the log with the LED: red 4 s, green 4 s, blue 4 s, white 4 s, off 2 s, twice. Before
//! each colour one line names it and the three bytes sent, in wire order (green, red, blue). The
//! program makes no claim of its own about what the LED shows.
#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO1;
use embassy_rp::pio::{self, Pio};
use embassy_time::Timer;
use panic_probe as _;
use ws2812c::pio::PioTransport;
use ws2812c::{Rgb, Ws2812c};

bind_interrupts!(struct Irqs {
    PIO1_IRQ_0 => pio::InterruptHandler<PIO1>;
});

/// Brightness applied by the driver to every colour.
const BRIGHTNESS: u8 = 64;

/// (name, colour, seconds shown).
const SEQUENCE: [(&str, Rgb, u64); 5] = [
    ("RED", Rgb::RED, 4),
    ("GREEN", Rgb::GREEN, 4),
    ("BLUE", Rgb::BLUE, 4),
    ("WHITE", Rgb::WHITE, 4),
    ("OFF", Rgb::OFF, 2),
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_rp::init(Default::default());
    defmt::info!(
        "[INFO] ws2812c_colours: WS2812C-2020 D20 on GPIO43, brightness {=u8}/255, 2 rounds of red 4 s, green 4 s, blue 4 s, white 4 s, off 2 s",
        BRIGHTNESS
    );

    {
        let Pio {
            mut common, sm0, ..
        } = Pio::new(p.PIO1.reborrow(), Irqs);
        let bus = PioTransport::new(&mut common, sm0, p.PIN_43.reborrow()).unwrap();
        let mut led = Ws2812c::new(bus);
        led.set_brightness(BRIGHTNESS);

        for round in 1..=2u32 {
            for (name, color, seconds) in SEQUENCE {
                let wire = color.scaled(BRIGHTNESS).wire();
                defmt::info!(
                    "[INFO] round {=u32}: {=str} for {=u64} s, bytes sent (green, red, blue) = {=u8} {=u8} {=u8}",
                    round,
                    name,
                    seconds,
                    (wire >> 24) as u8,
                    (wire >> 16) as u8,
                    (wire >> 8) as u8
                );
                led.set(color).await.unwrap();
                Timer::after_secs(seconds).await;
            }
        }
        led.off().await.unwrap();
        // Keep the line driven low until the LED has latched the off frame; releasing the PIO
        // lets the pad float.
        Timer::after_millis(1).await;
    }

    defmt::info!(
        "[INFO] [MANUAL] a person must have seen, in each of 2 rounds, the one LED D20 show red, green, blue, white (each 4 s) and then dark (2 s), the same colour as the log line printed just before it"
    );
    hil_common::exit(0)
}
