//! Peripheral Manager
//!
//! This construct facilitates the management of different hardware peripherals
//! on a chip or a board in order to easier invoke and notify peripherals
//! during transitions between sleep modes - and to determine which sleep mode
//! it is safe to transition into.
//!
//! For a peripheral to get these notifications or prevent specific sleep modes, it has to be
//! registered at a central PeripheralManager, which in turn is used when the chip is put
//! to sleep.
//!
//! ```rust
//! pub static mut M: PeripheralManager = PeripheralManager::new();
//! static mut UART_PERIPHERAL: Peripheral<'static> = unsafe { Peripheral::new(&uart::UART0) };
//!
//!  ...
//!
//!  M.register_peripheral(&UART_PERIPHERAL);
//! ```
//!
//! Following is an example from the cc26xx family of MCUs that shows how a peripheral
//! can implement the `PowerClient` trait to get these notifications.
//!
//! ```rust
//! impl peripheral_manager::PowerClient for UART {
//!     fn before_sleep(&self, _sleep_mode: u32) {
//!         // Wait for all transmissions to occur
//!         while self.busy() {}
//!
//!         unsafe {
//!             // Disable the TX & RX pins in order to avoid current leakage
//!             self.tx_pin.get().map(|pin| {
//!                 gpio::PORT[pin as usize].disable();
//!             });
//!             self.rx_pin.get().map(|pin| {
//!                 gpio::PORT[pin as usize].disable();
//!             });
//!
//!             PowerManager.release_resource(prcm::PowerDomain::Serial as u32);
//!         }
//!
//!         prcm::Clock::disable_uart_run();
//!     }
//!
//!     fn after_wakeup(&self, _sleep_mode: u32) {
//!         unsafe {
//!             PM.request_resource(prcm::PowerDomain::Serial as u32);
//!         }
//!         prcm::Clock::enable_uart_run();
//!         self.configure();
//!     }
//!
//!     fn lowest_sleep_mode(&self) -> u32 {
//!         chip::SleepMode::DeepSleep as u32
//!     }
//! }
//! ```
//!
//! The peripheral manager should then be used in conjunction with platform specific
//! sleep code, in a manner that would differ between platforms. Following is an example of how
//! the cc26x0 transitions into the lowest possible sleep mode using the peripheral manager.
//!
//! ```rust
//! let sleep_mode: SleepMode = SleepMode::from(unsafe { peripherals::M.lowest_sleep_mode() });
//!
//! match sleep_mode {
//!     SleepMode::DeepSleep => unsafe {
//!         peripherals::M.before_sleep(sleep_mode as u32);
//!         power::prepare_deep_sleep();
//!     },
//!     _ => (),
//! }
//!
//! unsafe { support::wfi() }
//!
//! match sleep_mode {
//!     SleepMode::DeepSleep => unsafe {
//!         power::prepare_wakeup();
//!         peripherals::M.after_wakeup(sleep_mode as u32);
//!     },
//!     _ => (),
//! }
//! ```
//!

use core::cell::Cell;
use kernel::common::{List, ListLink, ListNode};

/// A PowerClient implements ways to get notified when the chip changes its power mode.
pub trait PowerClient {
    fn before_sleep(&self, sleep_mode: u32);
    fn after_wakeup(&self, sleep_mode: u32);
    fn lowest_sleep_mode(&self) -> u32;
}

pub trait HWIntClient {
    fn enable(&self, interrupt_no: u32);
    fn disable(&self, interrupt_no: u32);
}

pub struct HWIntPeriph<'a> {
    args: Cell<Option<&'a [u8]>>,
    interrupt_no: Cell<u8>,
    enabled: bool,
}

/// Wrapper around PowerClient to be used in a linked list.
pub struct Peripheral<'a> {
    client: Cell<Option<&'a PowerClient>>,
    next: ListLink<'a, Peripheral<'a>>,
}

impl<'a> Peripheral<'a> {
    pub const fn new(client: &'a PowerClient) -> Peripheral {
        Peripheral {
            client: Cell::new(Some(client)),
            next: ListLink::empty(),
        }
    }

    /// Returns the lowest possible power mode this peripheral can enter at the moment.
    pub fn lowest_sleep_mode(&self) -> u32 {
        self.client
            .get()
            .map(|c| c.lowest_sleep_mode())
            .expect("No power client for a peripheral is set.")
    }

    /// Prepares the peripheral before going into sleep mode.
    pub fn before_sleep(&self, sleep_mode: u32) {
        self.client.get().map(|c| c.before_sleep(sleep_mode));
    }

    /// Initializes the peripheral after waking up from sleep mode.
    pub fn after_wakeup(&self, sleep_mode: u32) {
        self.client.get().map(|c| c.after_wakeup(sleep_mode));
    }
}

impl<'a> ListNode<'a, Peripheral<'a>> for Peripheral<'a> {
    fn next(&self) -> &'a ListLink<Peripheral<'a>> {
        &self.next
    }
}

/// Manages peripherals wanting to get notified when changing power modes.
pub struct PeripheralManager<'a> {
    peripherals: List<'a, Peripheral<'a>>,
}

impl<'a> PeripheralManager<'a> {
    pub const fn new() -> PeripheralManager<'a> {
        PeripheralManager {
            peripherals: List::new(),
        }
    }

    /// Registers a new peripheral to be managed by the PeripheralManager.
    pub fn register_peripheral(&self, peripheral: &'a Peripheral<'a>) {
        self.peripherals.push_head(peripheral);
    }

    /// Prepares all registered clients for entering sleep mode.
    pub fn before_sleep(&self, sleep_mode: u32) {
        for peripheral in self.peripherals.iter() {
            peripheral.before_sleep(sleep_mode);
        }
    }

    /// Starts all registered clients after waking up from sleep mode.
    pub fn after_wakeup(&self, sleep_mode: u32) {
        for peripheral in self.peripherals.iter() {
            peripheral.after_wakeup(sleep_mode);
        }
    }

    /// Returns the lowest possible power mode allowed by the registered clients.
    pub fn lowest_sleep_mode(&self) -> u32 {
        self.peripherals.iter().fold(0, |prev, peripheral| {
            prev.max(peripheral.lowest_sleep_mode())
        })
    }
}
