//! Physical memory wrapper
// TODO - make generic so paddr/etc can be u32/u64

use cache::bus_address_bits;

#[derive(Debug, Copy, Clone)]
pub struct PMem {
    vaddr: u64,
    paddr: u32,
    /// Size in bytes
    size: usize,
}

impl PMem {
    pub fn new(vaddr: u64, paddr: u32, size: usize) -> Self {
        assert_ne!(vaddr, 0);
        assert_ne!(paddr, 0);
        assert_ne!(size, 0);
        Self { vaddr, paddr, size }
    }

    pub fn as_slice<T>(&self, count: usize) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.as_ptr(), count) }
    }

    pub fn as_mut_slice<T>(&self, count: usize) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), count) }
    }

    pub fn as_ptr<T>(&self) -> *const T {
        self.vaddr as *const T
    }

    pub fn as_mut_ptr<T>(&self) -> *mut T {
        self.vaddr as *mut T
    }

    pub fn vaddr(&self) -> u64 {
        self.vaddr
    }

    pub fn paddr(&self) -> u32 {
        self.paddr
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn bus_paddr(&self) -> u32 {
        self.paddr | bus_address_bits::ALIAS_4_L2_COHERENT
    }
}
