//! SPI

use hal;
pub use hal::spi::{Mode, Phase, Polarity};
use nb;
use bcm2837::spi0::*;
