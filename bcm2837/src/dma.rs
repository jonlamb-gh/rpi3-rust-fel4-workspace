use super::MMIO_BASE;

use core::ops::Deref;
use register::mmio::{ReadOnly, ReadWrite};

register_bitfields! {
    u32,

    /// Control and status
    CS [
        ACTIVE OFFSET(0) NUMBITS(1) [],
        END OFFSET(1) NUMBITS(1) [],
        INT OFFSET(2) NUMBITS(1) [],
        DREQ OFFSET(3) NUMBITS(1) [],
        PAUSED OFFSET(4) NUMBITS(1) [],
        DREQ_STOPS_DMA OFFSET(5) NUMBITS(1) [],
        WAITING_FOR_OUTSTANDING_WRITES OFFSET(6) NUMBITS(1) [],
        ERROR OFFSET(8) NUMBITS(1) [],
        PRIORITY OFFSET(16) NUMBITS(4) [],
        PANIC_PRIORITY OFFSET(20) NUMBITS(4) [],
        WAIT_FOR_OUTSTANDING_WRITES OFFSET(28) NUMBITS(1) [],
        DISDEBUG OFFSET(29) NUMBITS(1) [],
        ABORT OFFSET(30) NUMBITS(1) [],
        RESET OFFSET(31) NUMBITS(1) []
    ],

    /// Transfer information
    TI [
        INTEN OFFSET(0) NUMBITS(1) [],
        TDMODE OFFSET(1) NUMBITS(1) [],
        WAIT_RESP OFFSET(3) NUMBITS(1) [],
        DEST_INC OFFSET(4) NUMBITS(1) [],
        DEST_WIDTH OFFSET(5) NUMBITS(1) [],
        DEST_DREQ OFFSET(6) NUMBITS(1) [],
        DEST_IGNORE OFFSET(7) NUMBITS(1) [],
        SRC_INC OFFSET(8) NUMBITS(1) [],
        SRC_WIDTH OFFSET(9) NUMBITS(1) [],
        SRC_DREQ OFFSET(10) NUMBITS(1) [],
        SRC_IGNORE OFFSET(11) NUMBITS(1) [],
        BURST_LENGTH OFFSET(12) NUMBITS(4) [],
        PERMAP OFFSET(16) NUMBITS(5) [],
        WAITS OFFSET(21) NUMBITS(5) [],
        NO_WIDE_BURSTS OFFSET(26) NUMBITS(1) []
    ],

    /// Transfer length
    TXFR_LEN [
        XLENGTH OFFSET(0) NUMBITS(16) [],
        YLENGTH OFFSET(16) NUMBITS(14) []
    ],

    /// 2D stride
    STRIDE [
        S_STRIDE OFFSET(0) NUMBITS(16) [],
        D_STRIDE OFFSET(16) NUMBITS(16) []
    ],

    /// Debug
    DEBUG [
        READ_LAST_NOT_SET_ERROR OFFSET(0) NUMBITS(1) [],
        FIFO_ERROR OFFSET(1) NUMBITS(1) [],
        READ_ERROR OFFSET(2) NUMBITS(1) [],
        OUTSTANDING_WRITES OFFSET(4) NUMBITS(4) [],
        DMA_ID OFFSET(8) NUMBITS(8) [],
        DMA_STATE OFFSET(16) NUMBITS(8) [],
        VERSION OFFSET(25) NUMBITS(3) [],
        LITE OFFSET(28) NUMBITS(1) []
    ]
}

/// Base address, each channel is offset by 0x100
pub const PADDR: u64 = MMIO_BASE + 0x0000_7000;

pub const CHANNEL0_OFFSET: u64 = 0x000;
pub const CHANNEL1_OFFSET: u64 = 0x100;
pub const CHANNEL2_OFFSET: u64 = 0x200;
pub const CHANNEL3_OFFSET: u64 = 0x300;
pub const CHANNEL4_OFFSET: u64 = 0x400;
pub const CHANNEL5_OFFSET: u64 = 0x500;
pub const CHANNEL6_OFFSET: u64 = 0x600;
pub const CHANNEL7_OFFSET: u64 = 0x700;
pub const CHANNEL8_OFFSET: u64 = 0x800;
pub const CHANNEL9_OFFSET: u64 = 0x900;
pub const CHANNEL10_OFFSET: u64 = 0xa00;
pub const CHANNEL11_OFFSET: u64 = 0xb00;
pub const CHANNEL12_OFFSET: u64 = 0xc00;
pub const CHANNEL13_OFFSET: u64 = 0xd00;
pub const CHANNEL14_OFFSET: u64 = 0xe00;

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    pub CS: ReadWrite<u32, CS::Register>,            // 0x00
    pub CONBLK_AD: ReadWrite<u32>,                   // 0x04
    pub TI: ReadOnly<u32, TI::Register>,             // 0x08
    pub SOURCE_AD: ReadOnly<u32>,                    // 0x0C
    pub DEST_AD: ReadOnly<u32>,                      // 0x10
    pub TXFR_LEN: ReadOnly<u32, TXFR_LEN::Register>, // 0x14
    pub STRIDE: ReadOnly<u32, STRIDE::Register>,     // 0x18
    pub NEXTCONBK: ReadOnly<u32>,                    // 0x1C
    pub DEBUG: ReadOnly<u32, DEBUG::Register>,       // 0x20
}

pub struct DMA {
    addr: *const u64,
}

impl From<u64> for DMA {
    fn from(vaddr: u64) -> DMA {
        assert_ne!(vaddr, 0);
        DMA {
            addr: vaddr as *const u64,
        }
    }
}

unsafe impl Send for DMA {}

impl DMA {
    pub fn ptr(&self) -> *const RegisterBlock {
        self.addr as *const _
    }
}

impl Deref for DMA {
    type Target = RegisterBlock;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}
