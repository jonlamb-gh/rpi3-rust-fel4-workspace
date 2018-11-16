use super::MMIO_BASE;

use core::ops::Deref;
use register::mmio::ReadWrite;

register_bitfields! {
    u32,

    /// GPIO Function Select 1
    GPFSEL1 [
        /// Pin 15
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            RXD0 = 0b100, // UART0     - Alternate function 0
            RXD1 = 0b010  // Mini UART - Alternate function 5
        ],

        /// Pin 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            TXD0 = 0b100, // UART0     - Alternate function 0
            TXD1 = 0b010  // Mini UART - Alternate function 5
        ]
    ],

    /// GPIO Pull-up/down Clock Register 0
    GPPUDCLK0 [
        /// Pin 15
        PUDCLK15 OFFSET(15) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 14
        PUDCLK14 OFFSET(14) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ]
    ]
}

pub const PADDR: u64 = MMIO_BASE + 0x0020_0000;

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    __reserved_0: u32,                                  // 0x00
    pub GPFSEL1: ReadWrite<u32, GPFSEL1::Register>,     // 0x04
    __reserved_1: [u32; 35],                            // 0x08
    pub GPPUD: ReadWrite<u32>,                          // 0x94
    pub GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>, // 0x98
}

pub struct GPIO {
    addr: *const u64,
}

impl From<u64> for GPIO {
    fn from(vaddr: u64) -> GPIO {
        assert_ne!(vaddr, 0);
        GPIO {
            addr: vaddr as *const u64,
        }
    }
}

unsafe impl Send for GPIO {}

impl GPIO {
    pub fn ptr(&self) -> *const RegisterBlock {
        self.addr as *const _
    }

    /*
    pub fn ptr() -> *const RegisterBlock {
        PADDR as *const _
    }

    // TODO
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
    */
}

impl Deref for GPIO {
    type Target = RegisterBlock;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}
