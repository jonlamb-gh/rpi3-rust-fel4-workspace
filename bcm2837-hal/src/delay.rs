//! Delays

use cortex_a::asm;
use hal::blocking::delay::{DelayMs, DelayUs};

use clocks::Clocks;

/// NOP used as a delay provider
/// NOTE: this is not accurate, for accurate timing and delays consider
/// using one of the timers
pub fn delay_us(us: u32) {
    let cnt = us * (Clocks::read().apbclk().0 / 250_000_000);

    for _ in 0..cnt {
        asm::nop();
    }
}

pub struct Delay {}

impl Delay {
    pub fn new() -> Self {
        Delay {}
    }
}

impl DelayMs<u32> for Delay {
    fn delay_ms(&mut self, ms: u32) {
        self.delay_us(ms * 1_000);
    }
}

impl DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_ms(ms as u32);
    }
}

impl DelayMs<u8> for Delay {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(ms as u32);
    }
}

impl DelayUs<u32> for Delay {
    fn delay_us(&mut self, us: u32) {
        delay_us(us);
    }
}

impl DelayUs<u16> for Delay {
    fn delay_us(&mut self, us: u16) {
        self.delay_us(us as u32)
    }
}

impl DelayUs<u8> for Delay {
    fn delay_us(&mut self, us: u8) {
        self.delay_us(us as u32)
    }
}
