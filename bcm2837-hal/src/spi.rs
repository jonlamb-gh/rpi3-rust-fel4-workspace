//! SPI

// TODO
// - use the more recent example Pins<> trait
// - support all available pins

use bcm2837::spi0::*;
use hal;
pub use hal::spi::{Mode, Phase, Polarity};
use nb;

use clocks::Clocks;
use gpio::{Pin10, Pin11, Pin7, Pin8, Pin9, AF0};
use time::Hertz;

// FIXME these should be "closed" traits
/// SCK pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait CePin<SPI> {}
/// SCK pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait SckPin<SPI> {}
/// MISO pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait MisoPin<SPI> {}
/// MOSI pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait MosiPin<SPI> {}

// SPI0 SSEL - CE0 Pin 8, CE1 Pin 7
//unsafe impl CePin<SPI0> for Pin7<AF0> {}
// TODO - need to be able to pass in CE_0 or CE_1 bits
unsafe impl CePin<SPI0> for Pin8<AF0> {}
unsafe impl MisoPin<SPI0> for Pin9<AF0> {}
unsafe impl MosiPin<SPI0> for Pin10<AF0> {}
unsafe impl SckPin<SPI0> for Pin11<AF0> {}

pub struct Spi<SPI, PINS> {
    spi: SPI,
    pins: PINS,
}

impl<CE, SCK, MISO, MOSI> Spi<SPI0, (CE, SCK, MISO, MOSI)> {
    pub fn spi0(
        spi: SPI0,
        pins: (CE, SCK, MISO, MOSI),
        mode: Mode,
        freq: Hertz,
        clocks: Clocks,
    ) -> Self
    where
        CE: CePin<SPI0>,
        SCK: SckPin<SPI0>,
        MISO: MisoPin<SPI0>,
        MOSI: MosiPin<SPI0>,
    {
        spi.CS.modify(CS::TA::CLEAR);
        spi.CS.modify(CS::CLEAR::ClearTxRx);

        // TODO

        Spi { spi, pins }
    }
}
