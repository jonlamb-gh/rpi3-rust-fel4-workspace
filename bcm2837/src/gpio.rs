use super::MMIO_BASE;

use core::ops::Deref;
use register::mmio::ReadWrite;

register_bitfields! {
    u32,

    /// GPIO Function Select 0
    GPFSEL0 [
        /// Pin 9
        FSEL9 OFFSET(27) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100, // SPI0 MISO - Alternate function 0
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 8
        FSEL8 OFFSET(24) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100, // SPI0 chip select 0 - Alternate function 0
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 7
        FSEL7 OFFSET(21) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100, // SPI0 chip select 1 - Alternate function 0
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 6
        FSEL6 OFFSET(18) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100,
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 5
        FSEL5 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100,
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ]
    ],

    /// GPIO Function Select 1
    GPFSEL1 [
        /// Pin 15
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100, // UART0 - Alternate function 0
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010 // Mini UART - Alternate function 5
        ],

        /// Pin 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100, // UART0 - Alternate function 0
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010 // Mini UART - Alternate function 5
        ],

        /// Pin 11
        FSEL11 OFFSET(3) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100, // SPI0 clock - Alternate function 0
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 10
        FSEL10 OFFSET(0) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100, // SPI0 MOSI - Alternate function 0
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ]
    ],

    /// GPIO Pull-up/down Register
    GPPUD [
        /// GPIO Pin Pull-up/down
        PUD OFFSET(0) NUMBITS(2) [
            Off = 0b00,
            PullDown = 0b01,
            PullUp = 0b10
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
        ],

        /// Pin 13
        PUDCLK13 OFFSET(13) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 12
        PUDCLK12 OFFSET(12) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 11
        PUDCLK11 OFFSET(11) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 10
        PUDCLK10 OFFSET(10) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 9
        PUDCLK9 OFFSET(9) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 8
        PUDCLK8 OFFSET(8) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 7
        PUDCLK7 OFFSET(7) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 6
        PUDCLK6 OFFSET(6) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 5
        PUDCLK5 OFFSET(5) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ]
    ]
}

pub const PADDR: u64 = MMIO_BASE + 0x0020_0000;

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    pub GPFSEL0: ReadWrite<u32, GPFSEL0::Register>, // 0x00
    pub GPFSEL1: ReadWrite<u32, GPFSEL1::Register>, // 0x04
    __reserved_0: [u32; 5],                         // 0x08
    pub GPSET0: ReadWrite<u32>,                     // 0x1C
    pub GPSET1: ReadWrite<u32>,                     // 0x20
    __reserved_1: u32,                              // 0x24
    pub GPCLR0: ReadWrite<u32>,                     // 0x28
    pub GPCLR1: ReadWrite<u32>,                     // 0x2C
    __reserved_2: u32,                              // 0x30
    pub GPLEV0: ReadWrite<u32>,                     // 0x34
    pub GPLEV1: ReadWrite<u32>,                     // 0x38
    __reserved_3: [u32; 22],                        // 0x3C
    pub GPPUD: ReadWrite<u32, GPPUD::Register>,     // 0x94
    pub GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>, //0x98
}

#[derive(Debug, Copy, Clone)]
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
    pub fn as_ptr(&self) -> *const RegisterBlock {
        self.addr as *const _
    }
}

impl Deref for GPIO {
    type Target = RegisterBlock;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.as_ptr() }
    }
}
