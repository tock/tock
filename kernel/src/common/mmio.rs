//! Memory Mapped I/O Interfaces
//!
//! Most peripherals are implemented as memory mapped I/O. Intrinsically, this
//! means that accessing a peripheral requires dereferencing a raw pointer that
//! points to the peripheral's memory.
//!
//! Generally, Tock peripherals are modeled by two structures, such as:
//!
//! ```rust
//! /// The MMIO Structure.
//! #[repr(C)]
//! #[allow(dead_code)]
//! pub struct PeripheralRegisters {
//!     control: VolatileCell<u32>,
//!     interrupt_mask: VolatileCell<u32>,
//! }
//!
//! /// The Tock object that holds all information for this peripheral.
//! pub struct PeripheralHardware {
//!     mmio_address: StaticRef<PeripheralRegisters>,
//!     clock: &ChipSpecificPeripheralClock,
//! }
//! ```
//!
//! The first structure mirrors the MMIO specification. The second structure
//! holds a pointer to the actual address in memory. It also holds other
//! information for the peripheral. Kernel traits will be implemented for this
//! peripheral hardware structure. As the peripheral cannot derefence the raw
//! MMIO pointer safely, Tock provides the MMIOManager interface:
//!
//! ```rust
//! impl hil::uart::UART for PeripheralHardware {
//!    fn init(&self, params: hil::uart::UARTParams) {
//!        let regs_manager = &MMIOManager::new(self);
//!        regs_manager.registers.control.set(0x0);
//!        //           ^^^^^^^^^-- This is type &PeripheralRegisters
//! ```
//!
//! Each peripheral must tell the kernel where its registers live in memory:
//!
//! ```rust
//! /// Teaching the kernel how to create PeripheralRegisters.
//! impl MMIOInterface<pm::Clock> for PeripheralHardware {
//!     type MMIORegisterType = PeripheralRegisters;
//!
//!     fn get_registers(&self) -> &PeripheralRegisters {
//!         &*self.mmio_address
//!     }
//! }
//! ```
//!
//! Note, this example kept the `mmio_address` in the `PeripheralHardware`
//! structure, which is useful when there are multiple copies of the same
//! peripheral (e.g. multiple UARTs). For single-instance peripherals, it's
//! fine to simply return the address directly from `get_registers`.
//!
//! Peripheral Clocks
//! -----------------
//!
//! To facilitate low-power operation, MMIOManager captures the peripheral's
//! clock upon instantiation. The intention is to exploit
//! [Ownership Based Resource Management](https://doc.rust-lang.org/beta/nomicon/obrm.html)
//! to capture peripheral power state.
//!
//! To enable this, peripherals must inform the kernel which clock they use,
//! and when the clock should be enabled and disabled. Implementations of the
//! `before/after_mmio_access` methods must take care to not access hardware
//! without enabling clocks if needed if they use hardware for bookkeeping.
//!
//! ```rust
//! /// Teaching the kernel which clock controls SpiHw.
//! impl MMIOClockInterface<pm::Clock> for SpiHw {
//!     fn get_clock(&self) -> &pm::Clock {
//!         &pm::Clock::PBA(pm::PBAClock::SPI)
//!     }
//! }
//!
//! /// Logic for when the SpiHw clock should be enabled and disabled.
//! impl MMIOClockGuard<SpiHw, pm::Clock> for SpiHw {
//!     fn before_mmio_access(&self, clock: &pm::Clock, _registers: &SpiRegisters) {
//!         clock.enable();
//!     }
//!
//!     fn after_mmio_access(&self, clock: &pm::Clock, _registers: &SpiRegisters) {
//!         if !self.is_busy() {
//!             clock.disable();
//!         }
//!     }
//! }
//! ```

use ClockInterface;

/// A structure encapsulating a peripheral should implement this trait.
pub trait MMIOInterface<C>
where
    C: ClockInterface,
{
    type MMIORegisterType;

    fn get_registers(&self) -> &Self::MMIORegisterType;
}

/// A structure encapsulating a clocked peripheral should implement this trait.
pub trait MMIOClockInterface<C>
where
    C: ClockInterface,
{
    fn get_clock(&self) -> &C;
}

/// Hooks for peripherals to enable and disable clocks as appropriate.
pub trait MMIOClockGuard<H, C>
where
    H: MMIOInterface<C>,
    C: ClockInterface,
{
    fn before_mmio_access(&self, &C, &H::MMIORegisterType);
    fn after_mmio_access(&self, &C, &H::MMIORegisterType);
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
    H: 'a + MMIOInterface<C> + MMIOClockGuard<H, C>,
    C: 'a + ClockInterface,
{
    pub registers: &'a H::MMIORegisterType,
    peripheral_hardware: &'a H,
    clock: &'a C,
}

impl<'a, H, C> MMIOManager<'a, H, C>
where
    H: 'a + MMIOInterface<C> + MMIOClockInterface<C> + MMIOClockGuard<H, C>,
    C: 'a + ClockInterface,
{
    pub fn new(peripheral_hardware: &'a H) -> MMIOManager<'a, H, C> {
        let registers = peripheral_hardware.get_registers();
        let clock = peripheral_hardware.get_clock();
        peripheral_hardware.before_mmio_access(clock, registers);
        MMIOManager {
            registers,
            peripheral_hardware,
            clock,
        }
    }
}

impl<'a, H, C> Drop for MMIOManager<'a, H, C>
where
    H: 'a + MMIOInterface<C> + MMIOClockGuard<H, C>,
    C: 'a + ClockInterface,
{
    fn drop(&mut self) {
        self.peripheral_hardware
            .after_mmio_access(self.clock, self.registers);
    }
}
