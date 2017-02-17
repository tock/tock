//! Cortex-M3 NVIC

use kernel::common::volatile_cell::VolatileCell;

#[repr(C, packed)]
// Registers for the NVIC. Each
struct Registers {
    // Interrupt set-enable
    iser: [VolatileCell<u32>; 8],
    _reserved1: [u32; 24],
    // Interrupt clear-enable
    icer: [VolatileCell<u32>; 8],
    _reserved2: [u32; 24],
    // Interrupt set-pending (and read pending state)
    ispr: [VolatileCell<u32>; 8],
    _reserved3: [VolatileCell<u32>; 24],
    // Interrupt clear-pending (and read pending state)
    icpr: [VolatileCell<u32>; 8],
}

// NVIC base address
const BASE_ADDRESS: *mut Registers = 0xe000e100 as *mut Registers;

/// Clear all pending interrupts
pub unsafe fn clear_all_pending() {
    let nvic: &Registers = &*BASE_ADDRESS;
    for icpr in nvic.icpr.iter() {
        icpr.set(!0)
    }
}

/// Enable all interrupts
pub unsafe fn enable_all() {
    let nvic: &Registers = &*BASE_ADDRESS;
    for icer in nvic.iser.iter() {
        icer.set(!0)
    }
}

/// Disable all interrupts
pub unsafe fn disable_all() {
    let nvic: &Registers = &*BASE_ADDRESS;
    for icer in nvic.icer.iter() {
        icer.set(!0)
    }
}

/// Get the index (0-240) the lowest number pending interrupt, or `None` if none
/// are pending.
pub unsafe fn next_pending() -> Option<u32> {
    let nvic: &Registers = &*BASE_ADDRESS;

    for (block, ispr) in nvic.ispr.iter().enumerate() {
        let ispr = ispr.get();

        // If there are any high bits there is a pending interrupt
        if ispr != 0 {
            // trailing_zeros == index of first high bit
            let bit = ispr.trailing_zeros();
            return Some(block as u32 * 32 + bit);
        }
    }
    None
}

pub unsafe fn has_pending() -> bool {
    let nvic: &Registers = &*BASE_ADDRESS;

    nvic.ispr.iter().fold(0, |i, ispr| ispr.get() | i) != 0
}

/// An opaque wrapper for a single NVIC interrupt.
///
/// Hand these out to low-level driver to let them control their own interrupts
/// but not others.
pub struct Nvic(u32);

impl Nvic {
    /// Creates a new `Nvic`
    ///
    /// Marked unsafe because only chip/platform configuration code should be
    /// able to create these.
    pub const unsafe fn new(idx: u32) -> Nvic {
        Nvic(idx)
    }

    /// Enable the interrupt
    pub fn enable(&self) {
        let nvic: &Registers = unsafe { &*BASE_ADDRESS };
        let idx = self.0 as usize;

        nvic.iser[idx / 32].set(1 << (self.0 & 31));
    }

    /// Disable the interrupt
    pub fn disable(&self) {
        let nvic: &Registers = unsafe { &*BASE_ADDRESS };
        let idx = self.0 as usize;

        nvic.icer[idx / 32].set(1 << (self.0 & 31));
    }

    /// Clear pending state
    pub fn clear_pending(&self) {
        let nvic: &Registers = unsafe { &*BASE_ADDRESS };
        let idx = self.0 as usize;

        nvic.icpr[idx / 32].set(1 << (self.0 & 31));
    }
}
