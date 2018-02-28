//! Memory Mapped I/O Interfaces
//!
//! Most peripherals are implemented as memory mapped I/O. Intrinsically, this
//! means that accessing a peripheral requires dereferencing a raw pointer that
//! points to the peripheral's memory.
//!
//! Generally, Tock peripherals are modeled by two structures, such as:
//!
//!    #[repr(C)]
//!    #[allow(dead_code)]
//!    pub struct PeripheralRegisters {
//!        control: VolatileCell<u32>,
//!        interrupt_mask: VolatileCell<u32>,
//!    }
//!
//!    pub struct PeripheralHardware {
//!        mmio_address: *mut PeripheralRegisters,
//!        clock: &ChipSpecificPeripheralClock,
//!    }
//!
//! The first structure mirrors the MMIO specification. The second structure
//! holds a pointer to the actual address in memory. It also holds other
//! information for the peripheral. Kernel traits will be implemented for this
//! peripheral hardware structure. As the peripheral cannot derefence the raw
//! MMIO pointer safely, Tock provides the MMIOManager interface:
//!
//!    impl hil::uart::UART for PeripheralHardware {
//!       fn init(&self, params: hil::uart::UARTParams) {
//!           let regs_manager = &MMIOManager::new(self);
//!           regs_manager.registers.control.set(0x0);
//!           //           ^^^^^^^^^-- This is type &PeripheralRegisters
//!
//!
//! Peripheral Clocks
//! -----------------
//!
//! To facilitate low-power operation, MMIOManager captures the peripheral's
//! clock upon instantiation. The intention is to exploit
//! [Ownership Based Resource Management](https://doc.rust-lang.org/beta/nomicon/obrm.html)
//! to capture peripheral power state.
//!
//! Peripherals whose clock cannot be disabled should use `NoClockControl`.

use ClockInterface;

/// Hooks for peripherals to enable and disable clocks as appropriate.
pub trait MMIOClockGuard<C>
where
    C: ClockInterface,
{
    fn before_mmio_access(&self, &C);
    fn after_mmio_access(&self, &C);
}

/// A structure encapsulating a peripheral should implement this trait.
pub trait MMIOInterface<C>
where
    C: ClockInterface,
{
    type MMIORegisterType: MMIOClockGuard<C>;

    fn get_hardware_address(&self) -> *mut Self::MMIORegisterType;
}

/// A structure encapsulating a clocked peripheral should implement this trait.
pub trait MMIOClockInterface<C>
where
    C: ClockInterface,
{
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
pub struct MMIOManager<'a, H, C>
where
    H: 'a + MMIOInterface<C>,
    C: 'a + ClockInterface,
{
    pub registers: &'a H::MMIORegisterType,
    clock: &'a C,
}

impl<'a, H, C> MMIOManager<'a, H, C>
where
    H: 'a + MMIOInterface<C> + MMIOClockInterface<C>,
    C: 'a + ClockInterface,
{
    pub fn new(periphal_hardware: &'a H) -> MMIOManager<'a, H, C> {
        let registers = unsafe { &*periphal_hardware.get_hardware_address() };
        let clock = periphal_hardware.get_clock();
        registers.before_mmio_access(clock);
        MMIOManager { registers, clock }
    }
}
impl<'a, H, C> Drop for MMIOManager<'a, H, C>
where
    H: 'a + MMIOInterface<C>,
    C: 'a + ClockInterface,
{
    fn drop(&mut self) {
        self.registers.after_mmio_access(self.clock);
    }
}
