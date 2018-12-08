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
// assuming Pin8/CE_0 for now
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

        // TODO - assuming Pin8/CE_0 for now
        // cannot control CE lines manually it seems,
        // so consume whatever pin CEn is mapped to
        spi.CS.modify(CS::CS::CS_0);

        // TODO
        spi.CS.modify(CS::CSPOL::ActiveLow);
        spi.CS.modify(CS::CSPOL0::ActiveLow);
        spi.CS.modify(CS::CSPOL1::ActiveLow);
        spi.CS.modify(CS::CSPOL2::ActiveLow);

        if mode.polarity == Polarity::IdleHigh {
            spi.CS.modify(CS::CPOL::RestingHigh);
        } else {
            spi.CS.modify(CS::CPOL::RestingLow);
        }

        if mode.phase == Phase::CaptureOnSecondTransition {
            spi.CS.modify(CS::CPHA::Middle);
        } else {
            spi.CS.modify(CS::CPHA::Beginning);
        }

        spi.CS.modify(CS::LEN::CLEAR);

        spi.CS.modify(CS::INTR::CLEAR);
        spi.CS.modify(CS::INTD::CLEAR);

        spi.CS.modify(CS::DMAEN::CLEAR);

        // Bidirectional mode
        spi.CS.modify(CS::REN::SET);

        // TODO - clean this up
        // Clock divider, must be a multiple of 2
        let clk_div = if freq.0 >= clocks.apbclk().0 / 2 {
            // clk/2 is the fastest we can go
            2
        } else if freq.0 > 0 {
            let cd = clocks.apbclk().0 / freq.0;
            if cd >= 65536 {
                0
            } else {
                cd
            }
        } else {
            // 0 is the slowest we can go, divisor gets set to 65536
            0
        };
        spi.CLK.modify(CLK::CDIV.val(clk_div));

        Spi { spi, pins }
    }

    /*
    pub fn free(self) -> (SPI0, (CE, SCK, MISO, MOSI)) {
        (self.spi, self.pins)
    }
    */

    pub fn begin_transfer(&mut self) {
        // Transfer active
        self.spi.CS.modify(CS::TA::SET);
    }

    pub fn end_transfer(&mut self) {
        // Transfer not active
        self.spi.CS.modify(CS::TA::CLEAR);
    }
}
