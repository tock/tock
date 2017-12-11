//! Memory Mapped I/O Interfaces
//!
//! Most peripherals are implemented as memory mapped I/O. Intrinsically, this
//! means that accessing a peripheral requires dereferencing a raw pointer that
//! points to the peripheral's memory.
//!
//! The Tock kernel provides an MMIOManager that encapsulates this unsafety.
//! Critically, it trusts that:
//!
//!  - `get_hardware_address` returns the correct peripheral memory address
//!  - `MMIORegisterType` correctly maps to the hardware peripheral
//!
//!
//! Peripheral Clocks
//! -----------------
//!
//! To facilitate low-power operation, MMIOManager captures the peripheral's
//! clock upon instantiation. The intention is to exploit
//! [Ownership Based Resource Management](https://doc.rust-lang.org/beta/nomicon/obrm.html)
//! to capture peripheral power state. Upon creation, MMIOManager ensures that
//! the clock is powered on. Upon `Drop` (destruction), MMIOManager queries
//! the peripheral-specific `can_disable_clock` method. For peripherals with
//! long-running transactions (e.g. DMA operations) or those that require the
//! clock to be enabled to listen (e.g. some buses), this method should check
//! whether the peripheral can be powered off. In many cases, it is sufficient
//! to check whether the interrupt mask for the peripheral is active.
//!
//! Peripherals whose clock cannot be disabled should use `NoClockControl` from
//! this module. Work-in-progress implementaitons should use `AlwaysOnClock`,
//! which will never power off the peripheral clock.

use ::ClockInterface;


pub trait MMIOClockGuard<C> where
    C: ClockInterface,
{
    fn before_mmio_access(&self, &C);
    fn after_mmio_access(&self, &C);
}


/// The structure encapsulating a peripheral should implement this trait.
pub trait MMIOInterface<C> where
    C: ClockInterface,
{
    type MMIORegisterType : MMIOClockGuard<C>;
    type MMIOClockType : ClockInterface;

    fn get_hardware_address(&self) -> *mut Self::MMIORegisterType;
    fn get_clock(&self) -> &C;
}

/// Structures encapsulating periphal hardware (those implementing the
/// MMIOInterface trait) should instantiate an instance of this method to
/// accesss memory mapped registers.
///
/// ```rust
/// let mmio = &MMIOManager::new(self);
/// mmio.registers.control.set(0x1);
/// ```
pub struct MMIOManager<'a, H, C> where
    H: 'a + MMIOInterface<C>,
    C: 'a + ClockInterface,
{
    pub registers: &'a H::MMIORegisterType,
    periphal_hardware: &'a H,
}

impl<'a, H, C> MMIOManager<'a, H, C> where
    H: 'a + MMIOInterface<C>,
    C: 'a + ClockInterface,
{
    pub fn new(periphal_hardware: &'a H) -> MMIOManager<'a, H, C> {
        let registers = unsafe { &* periphal_hardware.get_hardware_address() };
        let clock = periphal_hardware.get_clock();
        registers.before_mmio_access(clock);
        MMIOManager { registers, periphal_hardware }
    }
}
impl<'a, H, C> Drop for MMIOManager<'a, H, C> where
    H: 'a + MMIOInterface<C>,
    C: 'a + ClockInterface,
{
    fn drop(&mut self) {
        let clock = self.periphal_hardware.get_clock();
        self.registers.after_mmio_access(clock);
    }
}


pub struct NoClockControl {}
impl ClockInterface for NoClockControl {
    type PlatformClockType = NoClockControl;
    fn is_enabled(&self) -> bool { true }
    fn enable(&self) {}
    fn disable(&self) {}
}

pub struct AlwaysOnClock {}
impl ClockInterface for AlwaysOnClock {
    type PlatformClockType = AlwaysOnClock;
    fn is_enabled(&self) -> bool { true }
    fn enable(&self) {}
    fn disable(&self) {}
}
