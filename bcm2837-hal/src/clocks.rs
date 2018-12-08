//! Clocks
//!
//! WARNING/TODO: assumes `core_clock` is running at the default 250 MHz
//!
//! TODO: proper determination of APB clock frequency

use time::Hertz;

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no
/// longer be changed
#[derive(Clone, Copy, Debug)]
pub struct Clocks {
    apbclk: Hertz,
}

impl Clocks {
    pub fn read() -> Self {
        Clocks {
            apbclk: Hertz(250_000_000),
        }
    }

    /// Returns the frequency of the APB
    pub fn apbclk(&self) -> Hertz {
        self.apbclk
    }
}
