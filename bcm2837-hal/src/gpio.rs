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
// - GPPUDCLK1 support

use bcm2837::gpio::*;
use core::marker::PhantomData;
use core::ops::Deref;
use cortex_a::asm;
use hal::digital::{InputPin, OutputPin, StatefulOutputPin};

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
    pub p7: Pin7<Input<Floating>>,
    pub p8: Pin8<Input<Floating>>,
    pub p9: Pin9<Input<Floating>>,
    pub p10: Pin10<Input<Floating>>,
    pub p11: Pin11<Input<Floating>>,
    pub p14: Pin14<Input<Floating>>,
    pub p15: Pin15<Input<Floating>>,
}

impl GpioExt for GPIO {
    type Parts = Parts;

    fn split(self) -> Parts {
        // Each pin gets a copy of the GPIO vaddr
        Parts {
            p7: Pin7 {
                pin: 0,
                addr: self.as_ptr() as _,
                _mode: PhantomData,
            },
            p8: Pin8 {
                pin: 0,
                addr: self.as_ptr() as _,
                _mode: PhantomData,
            },
            p9: Pin9 {
                pin: 0,
                addr: self.as_ptr() as _,
                _mode: PhantomData,
            },
            p10: Pin10 {
                pin: 0,
                addr: self.as_ptr() as _,
                _mode: PhantomData,
            },
            p11: Pin11 {
                pin: 0,
                addr: self.as_ptr() as _,
                _mode: PhantomData,
            },
            p14: Pin14 {
                pin: 0,
                addr: self.as_ptr() as _,
                _mode: PhantomData,
            },
            p15: Pin15 {
                pin: 0,
                addr: self.as_ptr() as _,
                _mode: PhantomData,
            },
        }
    }
}

/// GPIO pull-up/down clock sequence wait cycles
const WAIT_CYCLES: usize = 150;

macro_rules! gpio {
    ($GPFSELn:ident, $GPPUDCLKx:ident, $GPLEVx:ident, $GPSETx:ident, $GPCLRx:ident, [
        $($PXi:ident: ($pxi:ident, $FSELi:ident, $PUDCLKi:ident, $MODE:ty),)+
    ]) => {
$(
pub struct $PXi<MODE> {
    pin: u32,
    addr: *const u64,
    _mode: PhantomData<MODE>,
}

impl<MODE> Deref for $PXi<MODE> {
    type Target = RegisterBlock;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.addr as *const RegisterBlock) }
    }
}

impl<MODE> $PXi<MODE> {
    /// Configures the pin to operate in AF0 mode
    pub fn into_alternate_af0(self) -> $PXi<Alternate<AF0>> {
        // Select function
        self.$GPFSELn.modify($GPFSELn::$FSELi::AF0);
        self.GPPUD.write(GPPUD::PUD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.write($GPPUDCLKx::$PUDCLKi::AssertClock);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.set(0);

        $PXi { pin: self.pin, addr: self.addr, _mode: PhantomData }
    }

    /// Configures the pin to operate in AF1 mode
    pub fn into_alternate_af1(self) -> $PXi<Alternate<AF1>> {
        // Select function
        self.$GPFSELn.modify($GPFSELn::$FSELi::AF1);
        self.GPPUD.write(GPPUD::PUD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.write($GPPUDCLKx::$PUDCLKi::AssertClock);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.set(0);

        $PXi { pin: self.pin, addr: self.addr, _mode: PhantomData }
    }

    /// Configures the pin to operate in AF2 mode
    pub fn into_alternate_af2(self) -> $PXi<Alternate<AF2>> {
        // Select function
        self.$GPFSELn.modify($GPFSELn::$FSELi::AF2);
        self.GPPUD.write(GPPUD::PUD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.write($GPPUDCLKx::$PUDCLKi::AssertClock);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.set(0);

        $PXi { pin: self.pin, addr: self.addr, _mode: PhantomData }
    }

    /// Configures the pin to operate in AF3 mode
    pub fn into_alternate_af3(self) -> $PXi<Alternate<AF3>> {
        // Select function
        self.$GPFSELn.modify($GPFSELn::$FSELi::AF3);
        self.GPPUD.write(GPPUD::PUD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.write($GPPUDCLKx::$PUDCLKi::AssertClock);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.set(0);

        $PXi { pin: self.pin, addr: self.addr, _mode: PhantomData }
    }

    /// Configures the pin to operate in AF4 mode
    pub fn into_alternate_af4(self) -> $PXi<Alternate<AF4>> {
        // Select function
        self.$GPFSELn.modify($GPFSELn::$FSELi::AF4);
        self.GPPUD.write(GPPUD::PUD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.write($GPPUDCLKx::$PUDCLKi::AssertClock);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.set(0);

        $PXi { pin: self.pin, addr: self.addr, _mode: PhantomData }
    }

    /// Configures the pin to operate in AF5 mode
    pub fn into_alternate_af5(self) -> $PXi<Alternate<AF5>> {
        // Select function
        self.$GPFSELn.modify($GPFSELn::$FSELi::AF5);
        self.GPPUD.write(GPPUD::PUD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.write($GPPUDCLKx::$PUDCLKi::AssertClock);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.set(0);

        $PXi { pin: self.pin, addr: self.addr, _mode: PhantomData }
    }

    /// Configures the pin to operate as a floating input pin
    pub fn into_floating_input(self) -> $PXi<Input<Floating>> {
        // Select function
        self.$GPFSELn.modify($GPFSELn::$FSELi::Input);
        self.GPPUD.write(GPPUD::PUD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.write($GPPUDCLKx::$PUDCLKi::AssertClock);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.set(0);

        $PXi { pin: self.pin, addr: self.addr, _mode: PhantomData }
    }

    /// Configures the pin to operate as a pulled down input pin
    pub fn into_pull_down_input(self) -> $PXi<Input<PullDown>> {
        // Select function
        self.$GPFSELn.modify($GPFSELn::$FSELi::Input);
        self.GPPUD.write(GPPUD::PUD::PullDown);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.write($GPPUDCLKx::$PUDCLKi::AssertClock);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.set(0);

        $PXi { pin: self.pin, addr: self.addr, _mode: PhantomData }
    }

    /// Configures the pin to operate as a pulled up input pin
    pub fn into_pull_up_input(self) -> $PXi<Input<PullUp>> {
        // Select function
        self.$GPFSELn.modify($GPFSELn::$FSELi::Input);
        self.GPPUD.write(GPPUD::PUD::PullUp);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.write($GPPUDCLKx::$PUDCLKi::AssertClock);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.set(0);

        $PXi { pin: self.pin, addr: self.addr, _mode: PhantomData }
    }

    /// Configures the pin to operate as an push pull output pin
    pub fn into_push_pull_output(self) -> $PXi<Output<PushPull>> {
        // Select function
        self.$GPFSELn.modify($GPFSELn::$FSELi::Output);
        self.GPPUD.write(GPPUD::PUD::Off);

        // Enable pin
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.write($GPPUDCLKx::$PUDCLKi::AssertClock);
        for _ in 0..WAIT_CYCLES { asm::nop(); }
        self.$GPPUDCLKx.set(0);

        $PXi { pin: self.pin, addr: self.addr, _mode: PhantomData }
    }
}

impl<MODE> OutputPin for $PXi<Output<MODE>> {
    fn set_high(&mut self) {
        self.$GPSETx.set(1 << self.pin);
    }

    fn set_low(&mut self) {
        self.$GPCLRx.set(1 << self.pin);
    }
}

impl<MODE> StatefulOutputPin for $PXi<Output<MODE>> {
    fn is_set_high(&self) -> bool {
        !self.is_set_low()
    }

    fn is_set_low(&self) -> bool {
        self.$GPLEVx.get() & (1 << self.pin) == 0
    }
}

impl<MODE> InputPin for $PXi<Input<MODE>> {
    fn is_high(&self) -> bool {
        !self.is_low()
    }

    fn is_low(&self) -> bool {
        self.$GPLEVx.get() & (1 << self.pin) == 0
    }
}
)+
    }
}

gpio!(
    GPFSEL0,
    GPPUDCLK0,
    GPLEV0,
    GPSET0,
    GPCLR0,
    [
        Pin7: (p7, FSEL7, PUDCLK7, Input<Floating>),
        Pin8: (p8, FSEL8, PUDCLK8, Input<Floating>),
        Pin9: (p9, FSEL9, PUDCLK9, Input<Floating>),
    ]
);

gpio!(
    GPFSEL1,
    GPPUDCLK0,
    GPLEV0,
    GPSET0,
    GPCLR0,
    [
        Pin10: (p10, FSEL10, PUDCLK10, Input<Floating>),
        Pin11: (p11, FSEL11, PUDCLK11, Input<Floating>),
        Pin14: (p14, FSEL14, PUDCLK14, Input<Floating>),
        Pin15: (p15, FSEL15, PUDCLK15, Input<Floating>),
    ]
);
