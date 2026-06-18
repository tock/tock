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

```rust
impl MyPeripheral<'_> {
    pub fn handle_interrupt(&self) {
        // 1. Clear physical interrupt (Write-1-to-Clear as defined in RM)
        self.registers.ccr.modify(CCR::IE::Disabled);
        
        // 2. Notify client
        self.client.map(|client| {
            client.transaction_completed();
        });
    }
}
```

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
```

### Approved Patterns (ALWAYS DO THESE)
1. **Define Field Enums**: As shown in Section 1, represent all possible bit values as enums in the bitfield definition.
2. **Use Compile-time Constants (`const`)**: For physical parameters (like clock frequencies, division factors, maximum retry attempts, or hardware limits), define named constants:
   ```rust
   const CLOCK_FREQ_HZ: u32 = 520_833;
   const HW_TIMEOUT_MAX_POLLS: u32 = 200_000;
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

## 5. Miscellaneous Best Practices

- **Silent Failures**: Avoid silent failures. If an operation fails (e.g. invalid configuration), return an appropriate `ErrorCode` instead of silently ignoring it.
  panic! instead of swallowing unhandled interrupts.

- **Empty functions**: Do not leave empty function bodies. If a trait method is not applicable, return `Err(ErrorCode::NOSUPPORT)` or unimplemented!(), at least in debug mode.

---

## Reference Implementations in Codebase
Refer to these production-tested files when implementing new S32G3 drivers:
- **`chips/nxp_s32g3/src/linflexd.rs`**: Full implementation of LINFlexD UART console driver.
- **`chips/nxp_s32g3/src/stm.rs`**: System Timer Module driver implementing `Time`, `Counter`, and `Alarm` traits.
