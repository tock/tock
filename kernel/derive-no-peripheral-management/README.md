No Peripheral Management
========================

Tock generally provides tooling to support managed control of peripherals.
However, some peripherals may not require any support. These peripherals
can use this derive to handle all the boilerplate automatically.

For peripherals that require no runtime management, Tock still prefers the use
of a PeripheralManager, as it reduces the `unsafe` surface and presents an easy
mechanism for possible future enhancements that may wish to change an unmanaged
peripheral into a managed one.

Example / Usage
---------------

 - Update your Cargo.toml:

    derive-no-peripheral-managemenet = { path = "../../kernel/derive-no-peripheral-managemenet" }

 - Add this crate to your local lib.rs:

    #[macro_use]
    extern crate derive_no_clock_control;

 - Then your peripheral should look something like this:

```rust
use kernel::StaticRef;
use kernel::common::VolatileCell;
use kernel::common::peripherals::PeripheralManager;

/// The MMIO Structure.
#[repr(C)]
#[allow(dead_code)]
pub struct TestRegisters {
    control: VolatileCell<u32>,
    interrupt_mask: VolatileCell<u32>,
}

/// The Tock object that holds all information for this peripheral.
#[derive(NoPeripheralManagement)]
#[RegisterType(TestRegisters)]
pub struct TestHw {
    registers: StaticRef<TestRegisters>,
}

/// Mapping to actual hardware instance(s).
const TEST_BASE_ADDR: StaticRef<TestRegisters> =
    unsafe { StaticRef::new(0x40001000 as *const TestRegisters) };
pub static mut TEST0: TestHw = TestHw::new(TEST_BASE_ADDR);

/// Methods this peripheral exports to the rest of the kernel.
impl TestHw {
    const fn new(base_addr: StaticRef<TestRegisters>) -> TestHw {
        TestHw { registers: base_addr }
    }

    pub fn do_thing(&self) {
        let peripheral = &PeripheralManager::new(self);
        peripheral.registers.control.get();
    }
}
```
