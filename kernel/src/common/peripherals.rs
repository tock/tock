//! Peripheral Management
//!
//! Most peripherals are implemented as memory mapped I/O (MMIO).
//! Intrinsically, this means that accessing a peripheral requires
//! dereferencing a raw pointer that points to the peripheral's memory.
//!
//! Generally, Tock peripherals are modeled by two structures, such as:
//!
//! ```rust
//! # use kernel::common::cells::VolatileCell;
//! # use kernel::common::StaticRef;
//! # struct ChipSpecificPeripheralClock {};
//! /// The MMIO Structure.
//! #[repr(C)]
//! #[allow(dead_code)]
//! pub struct PeripheralRegisters {
//!     control: VolatileCell<u32>,
//!     interrupt_mask: VolatileCell<u32>,
//! }
//!
//! /// The Tock object that holds all information for this peripheral.
//! pub struct PeripheralHardware<'a> {
//!     mmio_address: StaticRef<PeripheralRegisters>,
//!     clock: &'a ChipSpecificPeripheralClock,
//! }
//! ```
//!
//! The first structure mirrors the MMIO specification. The second structure
//! holds a pointer to the actual address in memory. It also holds other
//! information for the peripheral. Kernel traits will be implemented for this
//! peripheral hardware structure. As the peripheral cannot derefence the raw
//! MMIO pointer safely, Tock provides the PeripheralManager interface:
//!
//! ```rust
//! # use kernel::common::cells::VolatileCell;
//! # use kernel::common::peripherals::PeripheralManager;
//! # use kernel::common::StaticRef;
//! # use kernel::hil;
//! # use kernel::ReturnCode;
//! # struct PeripheralRegisters { control: VolatileCell<u32> };
//! # struct PeripheralHardware { mmio_address: StaticRef<PeripheralRegisters> };
//! impl hil::uart::UART for PeripheralHardware {
//!     fn configure(&self, params: hil::uart::UARTParameters) -> ReturnCode {
//!         let peripheral = &PeripheralManager::new(self);
//!         peripheral.registers.control.set(0x0);
//!         //         ^^^^^^^^^-- This is type &PeripheralRegisters
//!         ReturnCode::SUCCESS
//!     }
//!     # fn set_client(&self, _client: &'static hil::uart::Client) {}
//!     # fn transmit(&self, _tx_data: &'static mut [u8], _tx_len: usize) {}
//!     # fn receive(&self, _rx_buffer: &'static mut [u8], _rx_len: usize) {}
//!     # fn abort_receive(&self) {}
//! }
//! # use kernel::common::peripherals::PeripheralManagement;
//! # use kernel::NoClockControl;
//! # impl PeripheralManagement<NoClockControl> for PeripheralHardware {
//! #     type RegisterType = PeripheralRegisters;
//!
//! #     fn get_registers(&self) -> &PeripheralRegisters {
//! #         &*self.mmio_address
//! #     }
//! #     fn get_clock(&self) -> &NoClockControl { unsafe { &kernel::NO_CLOCK_CONTROL } }
//! #     fn before_peripheral_access(&self, _c: &NoClockControl, _r: &Self::RegisterType) {}
//! #     fn after_peripheral_access(&self, _c: &NoClockControl, _r: &Self::RegisterType) {}
//! # }
//! ```
//!
//! Each peripheral must tell the kernel where its registers live in memory:
//!
//! ```rust
//! # use kernel::common::peripherals::PeripheralManagement;
//! # use kernel::common::StaticRef;
//! # pub struct PeripheralRegisters {};
//! # pub struct PeripheralHardware { mmio_address: StaticRef<PeripheralRegisters> };
//! /// Teaching the kernel how to create PeripheralRegisters.
//! use kernel::NoClockControl;
//! impl PeripheralManagement<NoClockControl> for PeripheralHardware {
//!     type RegisterType = PeripheralRegisters;
//!
//!     fn get_registers(&self) -> &PeripheralRegisters {
//!         &*self.mmio_address
//!     }
//!     # fn get_clock(&self) -> &NoClockControl { unsafe { &kernel::NO_CLOCK_CONTROL } }
//!     # fn before_peripheral_access(&self, _c: &NoClockControl, _r: &Self::RegisterType) {}
//!     # fn after_peripheral_access(&self, _c: &NoClockControl, _r: &Self::RegisterType) {}
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
//! To facilitate low-power operation, PeripheralManager captures the peripheral's
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
//! use kernel::common::peripherals::PeripheralManagement;
//! use kernel::common::StaticRef;
//! use kernel::ClockInterface;
//! // A dummy clock for this example.
//! // Real peripherals that do not have clocks should use NoClockControl from this module.
//! struct ExampleClock {};
//! impl ClockInterface for ExampleClock {
//!     fn is_enabled(&self) -> bool { true }
//!     fn enable(&self) { }
//!     fn disable(&self) { }
//! }
//!
//! // Dummy hardware for this example.
//! struct SpiRegisters {};
//! struct SpiHw<'a> {
//!     mmio_address: StaticRef<SpiRegisters>,
//!     clock: &'a ExampleClock,
//!     busy: bool,
//! };
//!
//! /// Teaching the kernel which clock controls SpiHw.
//! impl<'a> PeripheralManagement<ExampleClock> for SpiHw<'a> {
//!     type RegisterType = SpiRegisters;
//!
//!     fn get_registers(&self) -> &SpiRegisters { &*self.mmio_address }
//!
//!     fn get_clock(&self) -> &ExampleClock { self.clock }
//!
//!     fn before_peripheral_access(&self, clock: &ExampleClock, _registers: &SpiRegisters) {
//!         clock.enable();
//!     }
//!
//!     fn after_peripheral_access(&self, clock: &ExampleClock, _registers: &SpiRegisters) {
//!         if !self.busy {
//!             clock.disable();
//!         }
//!     }
//! }
//! ```

use ClockInterface;

/// A structure encapsulating a peripheral should implement this trait.
pub trait PeripheralManagement<C>
where
    C: ClockInterface,
{
    type RegisterType;

    /// How to get a reference to the physical hardware registers (the MMIO struct).
    fn get_registers(&self) -> &Self::RegisterType;

    /// Which clock feeds this peripheral.
    ///
    /// For peripherals with no clock, use `&::kernel::platform::NO_CLOCK_CONTROL`.
    fn get_clock(&self) -> &C;

    /// Called before peripheral access.
    ///
    /// Responsible for ensure the periphal can be safely accessed, e.g. that
    /// its clock is powered on.
    fn before_peripheral_access(&self, &C, &Self::RegisterType);

    /// Called after periphal access.
    ///
    /// Currently used primarily for power management to check whether the
    /// peripheral can be powered off.
    fn after_peripheral_access(&self, &C, &Self::RegisterType);
}

/// Structures encapsulating periphal hardware (those implementing the
/// PeripheralManagement trait) should instantiate an instance of this
/// method to accesss memory mapped registers.
///
/// ```
/// # use kernel::common::cells::VolatileCell;
/// # use kernel::common::peripherals::PeripheralManager;
/// # use kernel::common::StaticRef;
/// # pub struct PeripheralRegisters { control: VolatileCell<u32> };
/// # pub struct PeripheralHardware { mmio_address: StaticRef<PeripheralRegisters> };
/// impl PeripheralHardware {
///     fn example(&self) {
///         let peripheral = &PeripheralManager::new(self);
///         peripheral.registers.control.set(0x1);
///     }
/// }
/// # use kernel::common::peripherals::PeripheralManagement;
/// # use kernel::NoClockControl;
/// # impl PeripheralManagement<NoClockControl> for PeripheralHardware {
/// #     type RegisterType = PeripheralRegisters;
///
/// #     fn get_registers(&self) -> &PeripheralRegisters {
/// #         &*self.mmio_address
/// #     }
/// #     fn get_clock(&self) -> &NoClockControl { unsafe { &kernel::NO_CLOCK_CONTROL } }
/// #     fn before_peripheral_access(&self, _c: &NoClockControl, _r: &Self::RegisterType) {}
/// #     fn after_peripheral_access(&self, _c: &NoClockControl, _r: &Self::RegisterType) {}
/// # }
/// ```
pub struct PeripheralManager<'a, H, C>
where
    H: 'a + PeripheralManagement<C>,
    C: 'a + ClockInterface,
{
    pub registers: &'a H::RegisterType,
    peripheral_hardware: &'a H,
    clock: &'a C,
}

impl<H, C> PeripheralManager<'a, H, C>
where
    H: 'a + PeripheralManagement<C>,
    C: 'a + ClockInterface,
{
    pub fn new(peripheral_hardware: &'a H) -> PeripheralManager<'a, H, C> {
        let registers = peripheral_hardware.get_registers();
        let clock = peripheral_hardware.get_clock();
        peripheral_hardware.before_peripheral_access(clock, registers);
        PeripheralManager {
            registers,
            peripheral_hardware,
            clock,
        }
    }
}

impl<H, C> Drop for PeripheralManager<'a, H, C>
where
    H: 'a + PeripheralManagement<C>,
    C: 'a + ClockInterface,
{
    fn drop(&mut self) {
        self.peripheral_hardware
            .after_peripheral_access(self.clock, self.registers);
    }
}
