//! General Purpose Input / Output

// TODO
// - GPIO is a shared peripheral with common registers,
// split() just gives pin abstractions but changing AF for example
// still requires access to common GPPUDCLKn
//
// - Not all of the type states are supported

use hal::prelude::*;
use hal::digital::{OutputPin, InputPin};
use bcm2837::gpio::*;
use core::marker::PhantomData;
use core::ops::Deref;

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
    pub p0: Pin<Input<Floating>>,
    // TODO p1...pN
}

// TODO - store pin number in state?
pub struct Pin<MODE> {
    addr: *const u64,
    _mode: PhantomData<MODE>,
}

impl GpioExt for GPIO {
    type Parts = Parts;

    fn split(self) -> Parts {

        // Each pin gets a copy of the GPIO vaddr
        Parts {
            p0: Pin { addr: self.as_ptr() as _, _mode: PhantomData },
        }
    }
}

impl<MODE> Deref for Pin<MODE> {
    type Target = RegisterBlock;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.addr as *const RegisterBlock) }
    }
}

/// GPIO pull-up/down clock sequence wait cycles
const WAIT_CYCLES: usize = 150;

impl<MODE> Pin<MODE> {
    /// Configures the pin to operate in AF0 mode
    pub fn into_alternate_af0(self) -> Pin<Alternate<AF0>> {
        // TODO
        Pin { addr: self.addr, _mode: PhantomData }
    }

    // TODO - AF1:5

    // TODO
    // Configures the pin to operate as a floating input pin
    //pub fn into_floating_input(self) ->

    /// Configures the pin to operate as a pulled down input pin
    pub fn into_pull_down_input(self) -> Pin<Input<PullDown>> {
        // TODO
        Pin { addr: self.addr, _mode: PhantomData }
    }

    /// Configures the pin to operate as a pulled up input pin
    pub fn into_pull_up_input(self) -> Pin<Input<PullUp>> {
        // TODO
        Pin { addr: self.addr, _mode: PhantomData }
    }

    // TODO other input/output
}

impl<MODE> OutputPin for Pin<Output<MODE>> {
    fn set_high(&mut self) {
        // TODO
    }

    fn set_low(&mut self) {
        // TODO
    }
}

impl<MODE> InputPin for Pin<Input<MODE>> {
    fn is_high(&self) -> bool {
        !self.is_low()
    }

    fn is_low(&self) -> bool {
        // TODO
        true
    }
}
