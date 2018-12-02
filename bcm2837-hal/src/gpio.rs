//! General Purpose Input / Output

// TODO
//
// - GPIO is a shared peripheral with common registers,
// split() just gives pin abstractions but changing AF for example
// still requires access to common GPPUDCLKn
//
// - Not all of the type states are supported
//
// - Would be more efficient to group configs, currently
// each pin config takes 2x150 wait cycles
//
// - Will likely refactor this for better scalability, via macros maybe

use hal::prelude::*;
use hal::digital::{OutputPin, InputPin, StatefulOutputPin};
use bcm2837::gpio::*;
use core::marker::PhantomData;
use core::ops::Deref;
use cortex_a::asm;

/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The parts to split the GPIO into
    type Parts;

    /// Splits the GPIO block into independent pins and registers
    fn split(self) -> Self::Parts;
}

pub struct AF0;
pub struct AF1;
pub struct AF2;
pub struct AF3;
pub struct AF4;
pub struct AF5;

pub struct Alternate<MODE> {
    _mode: PhantomData<MODE>,
}

/// Input mode (type state)
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Floating input (type state)
pub struct Floating;

/// Pulled down input (type state)
pub struct PullDown;

/// Pulled up input (type state)
pub struct PullUp;

/// Open drain input or output (type state)
pub struct OpenDrain;

/// Output mode (type state)
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Push pull output (type state)
pub struct PushPull;

pub struct Parts {
    /// Pins
    //pub p0: Pin7<Input<Floating>>,
    pub p7: Pin7<Input<Floating>>,
    // TODO p1...pN
}

// TODO - store pin number in state?
pub struct Pin7<MODE> {
    pin: u32,
    addr: *const u64,
    _mode: PhantomData<MODE>,
}

impl GpioExt for GPIO {
    type Parts = Parts;

    fn split(self) -> Parts {

        // Each pin gets a copy of the GPIO vaddr
        Parts {
            p7: Pin7 { pin: 0, addr: self.as_ptr() as _, _mode: PhantomData },
        }
    }
}

impl<MODE> Deref for Pin7<MODE> {
    type Target = RegisterBlock;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.addr as *const RegisterBlock) }
    }
}

/// GPIO pull-up/down clock sequence wait cycles
const WAIT_CYCLES: usize = 150;

impl<MODE> Pin7<MODE> {
    /// Configures the pin to operate in AF0 mode
    pub fn into_alternate_af0(self) -> Pin7<Alternate<AF0>> {
        // Select function
        self.GPFSEL0.modify(GPFSEL0::FSEL7::AF0);
        self.GPPUD.write(GPPUD::PUD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.GPPUDCLK0.write(GPPUDCLK0::PUDCLK7::AssertClock);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.GPPUDCLK0.set(0);

        Pin7 { pin: self.pin, addr: self.addr, _mode: PhantomData }
    }

    // TODO - AF1:5

    /// Configures the pin to operate as a floating input pin
    pub fn into_floating_input(self) -> Pin7<Input<Floating>> {
        // Select function
        self.GPFSEL0.modify(GPFSEL0::FSEL7::Input);
        self.GPPUD.write(GPPUD::PUD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.GPPUDCLK0.write(GPPUDCLK0::PUDCLK7::AssertClock);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.GPPUDCLK0.set(0);

        Pin7 { pin: self.pin, addr: self.addr, _mode: PhantomData }
    }

    /// Configures the pin to operate as a pulled down input pin
    pub fn into_pull_down_input(self) -> Pin7<Input<PullDown>> {
        // Select function
        self.GPFSEL0.modify(GPFSEL0::FSEL7::Input);
        self.GPPUD.write(GPPUD::PUD::PullDown);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.GPPUDCLK0.write(GPPUDCLK0::PUDCLK7::AssertClock);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.GPPUDCLK0.set(0);

        Pin7 { pin: self.pin, addr: self.addr, _mode: PhantomData }
    }

    /// Configures the pin to operate as a pulled up input pin
    pub fn into_pull_up_input(self) -> Pin7<Input<PullUp>> {
        // Select function
        self.GPFSEL0.modify(GPFSEL0::FSEL7::Input);
        self.GPPUD.write(GPPUD::PUD::PullUp);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.GPPUDCLK0.write(GPPUDCLK0::PUDCLK7::AssertClock);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.GPPUDCLK0.set(0);

        Pin7 { pin: self.pin, addr: self.addr, _mode: PhantomData }
    }

    /// Configures the pin to operate as an push pull output pin
    pub fn into_push_pull_output(self) -> Pin7<Output<PushPull>> {
        // Select function
        self.GPFSEL0.modify(GPFSEL0::FSEL7::Output);
        self.GPPUD.write(GPPUD::PUD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.GPPUDCLK0.write(GPPUDCLK0::PUDCLK7::AssertClock);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.GPPUDCLK0.set(0);

        Pin7 { pin: self.pin, addr: self.addr, _mode: PhantomData }
    }
}

impl<MODE> OutputPin for Pin7<Output<MODE>> {
    fn set_high(&mut self) {
        self.GPSET0.set(1 << self.pin);
    }

    fn set_low(&mut self) {
        self.GPCLR0.set(1 << self.pin);
    }
}

impl<MODE> StatefulOutputPin for Pin7<Output<MODE>> {
    fn is_set_high(&self) -> bool {
        !self.is_set_low()
    }

    fn is_set_low(&self) -> bool {
        self.GPLEV0.get() & (1 << self.pin) == 0
    }
}

impl<MODE> InputPin for Pin7<Input<MODE>> {
    fn is_high(&self) -> bool {
        !self.is_low()
    }

    fn is_low(&self) -> bool {
        self.GPLEV0.get() & (1 << self.pin) == 0
    }
}

// TODO
// macro
// n = fn block id
// i = pin num
// x = level, set, clear block id
//
// GPFSELn
// FSELi
// PUDCLKi
// GPLEVx
// GPSETx
// GPCLRx
