---
name: tock-driver
description: Guidelines and procedures for writing a Tock device driver in Rust. Covers register and bitfield translation from the RM, selecting HIL traits, implementing driver logic via typesafe register operations, and enforcing compile-time safety without magic numbers.
user-invokable: true
---

# Writing Tock Device Drivers

This skill guides the agent in writing robust, standard-compliant, and compile-time safe Tock device drivers.

---

## 1. Translating Reference Manual to Register & Bitfield Descriptions

Tock interface drivers use `register_structs!` and `register_bitfields!` from the `tock-registers` crate to define hardware MMIO maps.

### Critical Rules for Register Definitions
1. **Source Veracity**: Translate offsets, names, and widths exactly as defined in the **SoC Reference Manual (RM)**. Reference the source Paragraphs.
2. **Docstring Coverage**: Every register in the `register_structs!` macro and every bit/field in the `register_bitfields!` macro **MUST** have an inline docstring explaining its role/function from the RM.
3. **No Raw Pointer Arithmetic**: Address mapping is defined using `StaticRef<T>` with physical base addresses.
4. **Reserved Fields**: Explicitly mark unused memory regions between registers as `_reserved` to maintain correct alignment.

### Structure Example
```rust
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

pub const MY_PERIPHERAL_BASE: StaticRef<MyPeripheralRegisters> =
    unsafe { StaticRef::new(0x4011_C000 as *const MyPeripheralRegisters) };

// RM §49.5.2
register_structs! {
    pub MyPeripheralRegisters {
        /// Control Register: configures mode and state
        (0x000 => pub cr: ReadWrite<u32, CR::Register>),      
        /// Count Register: contains the running counter
        (0x004 => pub cnt: ReadWrite<u32, CNT::Register>),    
        (0x008 => _reserved0),
        /// Channel Control Register: configures interrupts/DMA
        (0x00C => pub ccr: ReadWrite<u32, CCR::Register>),    
        (0x010 => @END),
    }
}
```

### Bitfields Example (With Compile-Time Enums)

Registers are hard to understand.
We must make sure every bit field is well documented, and that all values are represented as enums to prevent magic numbers and ensure compile-time safety.
To enforce compile-time safety, never use raw numbers for field values. Instead, declare enums inside the bitfield brackets `[]`:

```rust
register_bitfields![u32,
    /// Control Register (CR)
    CR [
        /// Prescaler Division Factor
        PRESCALER OFFSET(16) NUMBITS(8) [],
        /// Freeze Mode Configuration
        FRZ       OFFSET(1)  NUMBITS(1) [
            Normal = 0,
            Frozen = 1
        ],
        /// Peripheral Enable
        ENABLE    OFFSET(0)  NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ],

    /// Count Register (CNT)
    CNT [
        /// Current Count Value
        VAL OFFSET(0) NUMBITS(32) []
    ],

    /// Channel Control Register (CCR)
    CCR [
        /// Interrupt Enable
        IE  OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ]
];
```

---

## 2. Selecting the Correct Tock HIL Trait Abstraction

Tock decouples hardware-specific logic from generic OS capsules using Hardware Interface Layer (**HIL**) traits.

### Key HIL Locations in `kernel/src/hil/`
* **Console / Serial**: `kernel::hil::uart::Configure`, `kernel::hil::uart::Transmit`, `kernel::hil::uart::Receive`
* **Timers / Alarms**: `kernel::hil::time::Time`, `kernel::hil::time::Counter`, `kernel::hil::time::Alarm`
* **GPIO**: `kernel::hil::gpio::Pin`, `kernel::hil::gpio::InterruptPin`
* **SPI**: `kernel::hil::spi::SpiMaster`, `kernel::hil::spi::SpiSlave`

### Finding the Trait
1. Always search the `kernel/src/hil/` folder first to find the relevant trait.
2. Maintain exact trait signatures. Do not wrap custom errors; use Tock's native `kernel::ErrorCode` or `Result<(), ErrorCode>`.
3. If the driver is asynchronous, it **MUST** take a reference to a client trait (e.g. `TransmitClient`, `AlarmClient`) to schedule callbacks.

---

## 3. Implementing HIL Traits with Register Fields

When implementing the traits, utilize typesafe registers instead of manual bitwise operations (`&`, `|`, `<<`).

### Peripheral Struct Definition
```rust
use kernel::utilities::cells::OptionalCell;

pub struct MyPeripheral<'a> {
    registers: StaticRef<MyPeripheralRegisters>,
    client: OptionalCell<&'a dyn MyClientTrait>,
}

impl MyPeripheral<'_> {
    pub const fn new(base: StaticRef<MyPeripheralRegisters>) -> Self {
        Self {
            registers: base,
            client: OptionalCell::empty(),
        }
    }
}
```

### Typesafe Register Access Patterns
* **Write specific fields**:
  ```rust
  // Disables and freezes the peripheral
  self.registers.cr.write(CR::ENABLE::Disabled + CR::FRZ::Frozen);
  ```
* **Modify fields (preserving others)**:
  ```rust
  // Enables the peripheral without altering the prescaler/freeze bits
  self.registers.cr.modify(CR::ENABLE::Enabled);
  ```
* **Read and compare fields**:
  ```rust
  // Check if peripheral is currently enabled using compile-time safe enum
  if self.registers.cr.is_set(CR::ENABLE) {
      // is_set is shorthand for matching on '1' for 1-bit fields
  }
  
  if self.registers.cr.read(CR::FRZ) == CR::FRZ::Frozen {
      // typesafe comparison of multi-bit or enum values
  }
  ```

### Interrupt Service Routine (ISR) Implementation
Drivers must handle physical hardware interrupts and route them to their corresponding clients:
1. Clear the hardware interrupt flag inside `handle_interrupt(&self)` immediately to prevent interrupt storm/lockup.
2. Wrap the callback dispatch in `self.client.map(...)` blocks.
3. Configure the NVIC routing in the board's `main.rs`/`lib.rs`.
4. Callbacks from `handle_interrupt` may be fired **directly** — the call stack is bounded:
   `service_pending_interrupts` → `handle_interrupt` → `client.callback`.

```rust
impl MyPeripheral<'_> {
    pub fn handle_interrupt(&self) {
        // 1. Clear physical interrupt (Write-1-to-Clear as defined in RM)
        self.registers.ccr.modify(CCR::IE::Disabled);
        // 2. Notify client directly — safe from ISR context
        self.client.map(|client| {
            client.transaction_completed();
        });
    }
}
```

### Deferred Callbacks — Breaking the Call Stack

Some HIL methods are called *by* the capsule (e.g. `transmit_abort`, `receive_abort`) and their
contract promises a future callback to the same capsule. Firing that callback synchronously inside
the method creates a direct call cycle:

```
capsule::fn → driver::transmit_abort → client::transmitted_buffer → capsule::fn → …
```

This will overflow the stack. The rule is: **if a client callback would be delivered from within
a method that the client itself invoked, use `DeferredCall` to break the cycle.**

#### Pattern

1. Add a `DeferredCall` field and a state cell to the driver struct:
   ```rust
   use kernel::deferred_call::{DeferredCall, DeferredCallClient};

   pub struct MyPeripheral<'a> {
       // ...
       tx_state: Cell<TxState>,
       deferred_call: DeferredCall,
   }
   ```
2. In the HIL method, record the outcome and schedule the deferred call — **do not call the
   client directly**:
   ```rust
   fn transmit_abort(&self) -> Result<(), ErrorCode> {
       // Cancel in-flight TX, record reason
       self.tx_state.set(TxState::Aborted(Err(ErrorCode::CANCEL)));
       self.deferred_call.set(); // schedule callback delivery
       Err(ErrorCode::BUSY)     // signals: callback coming later
   }
   ```
3. Implement `DeferredCallClient` and deliver the callback there:
   ```rust
   impl DeferredCallClient for MyPeripheral<'_> {
       fn register(&'static self) {
           self.deferred_call.register(self);
       }

       fn handle_deferred_call(&self) {
           if let TxState::Aborted(rcode) = self.tx_state.get() {
               self.tx_state.set(TxState::Idle);
               self.tx_client.map(|client| {
                   self.tx_buffer.take().map(|buf| {
                       client.transmitted_buffer(buf, 0, rcode);
                   });
               });
           }
       }
   }
   ```
4. Call `peripheral.register()` during board initialisation, before `kernel_loop()`.

#### When to use DeferredCall vs direct callback

| Delivery site | Client called the method? | Use DeferredCall? |
|---|---|---|
| `handle_interrupt()` | No (hardware fires it) | No — call directly |
| `transmit_abort()` / `receive_abort()` | Yes | **Yes** |
| Any synchronous HIL method that promises a future callback | Yes | **Yes** |

> **Check the HIL trait's documentation.** Some traits explicitly state that a callback
> will (or will not) be delivered synchronously. Always honour that contract.

---

## 4. Enforcing Compile-Time Safety & Eliminating Magic Numbers

We enforce **zero-tolerance** for magic numbers in driver logic. All bit manipulations and values must be represented through safe abstractions.

### Prohibited Anti-Patterns (NEVER DO THESE)
```rust
// WRONG: Magic hexadecimal values and manual shifts
self.registers.cr.set(0x00010003); // BRICKS/SCRAMBLES easily

// WRONG: Raw bit-wise operations
let val = self.registers.cr.get() & !(1 << 5);
self.registers.cr.set(val | (1 << 0));

// WRONG: raw pointer / volatile for MMIO — bypasses type safety and read/write semantics
let addr = (BASE + OFFSET) as *mut u32;
core::ptr::write_volatile(addr, core::ptr::read_volatile(addr) | 0x0002);

// WRONG: separate module holding raw constants for a bitfield's possible values
mod my_state {
    pub const IDLE: u32 = 0b0010;  // no type safety, bypasses tock-registers entirely
    pub const INIT: u32 = 0b0001;
}
// These values will never be type-checked against the register field they belong to.
// Put them as variants INSIDE the register_bitfields! definition instead (see §1).
```

### Approved Patterns (ALWAYS DO THESE)
1. **Define Field Enums**: As shown in Section 1, represent all possible bit values as variants
   **inline** inside the `register_bitfields!` definition — never in a separate `mod` or as bare `const`:
   ```rust
   MY_STATE OFFSET(12) NUMBITS(4) [
       Idle = 0b0010,
       Init = 0b0001,
   ]
   ```
2. **Use Compile-time Constants (`const`)**: For physical parameters (clock frequencies, division
   factors, spin-wait ceilings), define named constants. **Every spin-wait constant must carry a
   comment** stating its units, worst-case duration at the operating clock, and what the caller
   must do on expiry — never silently continue:
   ```rust
   const CLOCK_FREQ_HZ: u32 = 520_833;

   // Units: bare loop iterations (register read + compare + branch).
   // At 48 MHz FIRC (~10 cycles/MMIO read) this caps the wait at ≈40 ms,
   // well above the hardware's sub-microsecond transition.
   // Callers MUST propagate an error on expiry, not silently continue.
   const HW_POLL_MAX: u32 = 200_000;
   ```
3. **Compound Write Operations**: Combine operations using the `+` operator, ensuring a single MMIO write transaction:
   ```rust
   self.registers.cr.write(
       CR::PRESCALER.val(12) + // Using .val() for numeric fields without dedicated enums
       CR::FRZ::Normal +       // Using typesafe enum values
       CR::ENABLE::Enabled
   );
   ```

---

## 5. Busy-Wait Discipline

Tock is a cooperative, interrupt-driven kernel. Spinning inside a running kernel
stalls every other task for an unbounded time and invalidates all scheduling
latency arguments. The rules below are **non-negotiable** for safety certification.

### Absolute prohibitions — NEVER spin in:
- `handle_interrupt()` or any function reachable from it
- `DeferredCall` handlers
- HIL client callbacks (e.g. `TransmitClient::transmitted_buffer`)
- Any code path that can execute after `kernel.kernel_loop()` has been called

Spinning in any of the above is a latency violation that cannot be argued away at
any ASIL level. Use the interrupt-driven HIL callbacks instead.

### When a busy wait is unavoidable
Some synchronous HIL methods (e.g. `Configure::configure`, `Transmit::transmit_sync`)
have no async counterpart, yet the hardware protocol requires polling a status bit
before the operation can complete (e.g. LINFlexD must reach INIT mode before
registers can be written). In these cases:

1. **Name and document the ceiling constant** (see §4 rule 2 — units, WCET at
   operating clock, error-propagation contract on expiry).
2. **Tag the containing function as INIT-ONLY** with a doc comment:
   ```rust
   /// Configure the peripheral.
   ///
   /// # INIT-ONLY
   /// Contains a hardware spin-wait (bounded by `HW_POLL_MAX`; WCET ≈ 40 ms at
   /// 48 MHz FIRC).  **Must only be called during board initialisation, before
   /// `kernel_loop()`.  Runtime reconfiguration is prohibited — see safety manual
   /// §UART-INIT.**
   fn configure(&self, params: uart::Parameters) -> Result<(), ErrorCode> {
   ```
3. **Record the constraint in the safety manual.** The driver cannot enforce that
   it is never called at runtime; the safety argument must live outside the code.
   For each INIT-ONLY function, the safety manual entry must state:
   - The function name and crate
   - The maximum spin duration (WCET)
   - The requirement: called only from the board-init path, before `kernel_loop()`

### Summary table

| Call site | Busy wait allowed? |
|---|---|
| Board init (`reset_handler` → `main`, before `kernel_loop()`) | Yes, with WCET comment + INIT-ONLY tag |
| Sync HIL method (e.g. `configure`, `transmit_sync`) called from board init | Yes, same requirements |
| Panic handler (`IoWrite::write`) | Yes — kernel is already halted |
| `handle_interrupt` / DeferredCall / client callback | **Never** |
| Any path after `kernel_loop()` | **Never** |

---

## 6. Miscellaneous Best Practices

- **Silent Failures**: Avoid silent failures. If an operation fails (e.g. invalid configuration), return an appropriate `ErrorCode` instead of silently ignoring it.
  panic! instead of swallowing unhandled interrupts.

- **Empty functions**: Do not leave empty function bodies. If a trait method is not applicable, return `Err(ErrorCode::NOSUPPORT)` or unimplemented!(), at least in debug mode.

---

## Reference Implementations in Codebase
These drivers in `chips/stm32f4xx/src/` are the canonical examples of
correct Tock driver structure. Read them before writing a new driver:
- **`chips/stm32f4xx/src/usart.rs`**: UART with DMA, `SubSliceMut` buffer management,
  full `Configure`/`Transmit`/`Receive` HIL impl, deferred-call TX abort.
- **`chips/stm32f4xx/src/tim2.rs`**: General-purpose timer implementing `Time`, `Counter`,
  and `Alarm` traits with clock-gating via `ClockInterface`.
- **`chips/stm32f4xx/src/gpio.rs`**: GPIO pin with alternate-function muxing, interrupt
  routing via EXTI, and the `Pin`/`InterruptPin` HIL.
