# Tock Core Notes 05/08/2020

Attending:
 - Leon Schuermann
 - Philip Levis
 - Samuel Jero
 - Vadim Sukhomlinov
 - Alistair
 - Hudson Ayers
 - Brad Campbell
 - Amit Levy
 - Branden Ghena
 - Andrey Pronin
 - Johnathan Van Why
 - Pat Pannuto

## Updates

 * Brad: Would like to update nightly, but blocked on an open bug upstream
 * Brad: We should also try to make some progress on our backlog of PRs
     - amit: we should assign and use that to get maintainers to move forward
     - hudson: many PRs are partially complete, or open Q's (i.e. stable rust?)
     - phil: Need to balance how to do long-term goals; all things as PRs results in hard to get big changes made
     - amit: Yes, but in the short term we do have a backlog

## ChromeOS Presentation (by Vadim)

 * Vadim:
    - Part of ChromeOS hardware security team
    - Ti50 is startup chip; root-of-trust
    - Ti50 boots first, central point that starts main application processor (i.e. the i5), interfaces to fingerprint sensor, etc
    - Ti50 is new security IC, built on RV32IMC+; mask ROM firmware; ideally running Tock
    --- Optional dedicated IC software (flexible on top?) Extended root of trust, Certified Crypto Libraries, Initialization
    - ChromeOS Use Cases:
        - System Manger / Platform Root of Trust
           -- enhanced, security hardened RISC-V chip
           -- detection, mitigation, and recovery of security issues
           -- always ON, even when main apps processors is off --> system power management
        - Multiple Applications
           -- execution from flash (due to code size)
           -- max code reuse between several chip variants
        - Secure & Robust firmware upgrades
           -- 2 flash banks - active (golden) copy and updatable + active data
        - Management of platform secrets (TPM, U2F, OS login, etc)
           -- hardened crypto API
           -- confidentiality & integrity of system & apps persistent data
        - Closed-case debug support
           -- low latency UART multiplexing
        - Shared interface to apps (SPI or I2C; USB also in the mix)
           -- Dispatching of commands from app processor among Ti50 apps based on command codes
        - High availability - reboot cause platform reboot
           -- Support for watchdog timer, sleep, & deep sleep modes
        - Certifications (FIPS crypto, Common Criteria - TPM, U2F, etc)
           -- Need to prove isolation of applications is good enough
           -- Testing, fuzzing, 100% branch-coverage, no dead code at source & binary
           -- Traceability of security threats, security objectives, security functional requirements, and functional tests -> tracking of requirements, artifacts
           -- Reproducible builds
           -- Independent testing, including vulnerability reward program
   - "Extremely multi-tenant case"
      - Expect to be actively processing multiple needs at the same time
      - Unlike isolated secure enclaves, this controls whole system, more responsibilities
   - Multiple Applications
      - extended thread model for our application (WIP)
         - confidentiality & integrity of data, residual data leakage, covert channels, vulnerabilities, ....
         - defense in depth (lite-ASLR, check for stack pivoting, no data execution, etc)
      - code size & performance
         - shared libs among apps
         - static applications -> possible changes in Process::Create
         - efficient IPC, ideally close to zero-copy
         - syscall performance (next slide)
      - isolation of resources
         - ACLs on syscalls / devices / capsules
         - encrypted file system with ACLs
         - crypto key management
         - application reset on panic
  - Performance optimizations
     - low latency requirement for interrupt processing
        - transfer data from one UART to another while monitoring for control sequences
    - syscall penalty reduction
       - expect many syscalls for crypto & I/O
       - home-grown OS has 50 cycles penalty, today Tock master is ~5172 [OT down to ~450] cycles
       - synchronous syscalls (don't subscribe just to always yield) -> remove 2 syscalls
       - enable direct use of constants from .rodata -> remove some allow() syscalls
       - different syscall conventions (a0 -> t6 to minimize register shuffling), pass more regs, stay with just 'command' syscall, make 'allow' part of 'command' syscall
    - IPC using shared-memory
       - dispatch commands/responses up to 4K among apps
       - AppID as u32 or u8
    - 64-bit timers, avoid long division in timer by changing timer frequency 
    > - amit: we have no love for our current IPC interface, but can you be more precise about what is different?
    > - vadim: looks more like a series of IPC buffers that are extant, then syscalls change access to the buffers; a "transfer" of region from one to another
    > - amit: similar to an exchange heap? like move semantics?
    > - vadim: yes
    > - amit: regarding syscalls: where are all these additional cycles coming from? in old ARM measurements was ~300 cycles, mostly configuring MPU
    > - vadim: not my data, from Titan team, so not sure
    > - [editor's note]: looks like it's apples/oranges, those cycle #'s might be for semantic operations, which may be several syscalls, given suggested optimizations
    > - amit: in general, these sycall optimizations seem good; in the cases of back-to-back syscalls, how much of this is syscall design versus inefficiencies in userspace (e.g. redundant subscribe)
    > - vadim: probably some of both; subscribe likely userspace, but command+yield likely often useful
 - Enhanceds RISC-V core (Google internal)
    - Integrated Root of Trust
       - code signing required
    - Certified crypto libraries
       - Use APi to perform operations vs. direct HW access
    - 16 PMP regions
    - Power management, deep sleep support
    - security alerts
    - additional protection mechanism extending PMPs
    - New CSRs and instructions (subset of bitmanip)
       - Modified toolchain to support
 - Crypto libraries
    - API for key generation and management
       - use key handles for apps
       - export keys only using key-wrapping, blob for apps
       - access control to keys on per application basis
       - side-loading of keys for hardware-bound keys
       - zeroization of keys
       -board-specific flash region with restricted access
    - symmetric ciphers, diff modes (OFB, GMAC, KWP, CTR, etc)
    - public key crypto (RSA 4K, ECDSA P-256/P-384, ECDAA(?), etc)
    - parallel context support, sharing hardware resources
    - FIPS 140-3 compliance (health checks, etc)
    - post quantum, firmware signature verification
    - HW-accelerators (AES, HMAC, DRBG, Big num)
       - via certified crypto lib primarily for ChromeOS
 - Filesystems
   - efficient use of shared flash space
   - device-bound, app-bound encryption
   - integrity protection (AEAD, etc)
   - flash brown-out resistance (incomplete writes/erase due to power-off)
   - ACLs for objects
   transaction support (detect incomplete)
   - flash wear minimization
   - performance considerations
     - minimize erase count
     - flash bank aware
 - Host emulation
    - multiple targets for same code
       - target security IC
       - verilator
       - QEMU (with device emulation at register level)
       - host (device emulation at register level)
    - device emulation at register level
       - hooks to tock::register
       - use mostly same driver code as on target for coverage
    - host execution model
       - maximize code reuse from target, not just emulated syscalls
       - emulate context switching
       - interrupts from devices
       - syscall handling
    - Addressing...
       - unit testing for drivers
       - [...]
 - Testing
    - Automated unit tests
    - Automated integration tests (single & multi-app)
       - reuse same test framework among apps and core
       - all levels and targets
    - branch coverage (on target & host emu)
    - fuzzing
    - HWASAN (software memory tagging)
        - apps can be in C, unsafe code in crypto libs, etc
        - need toolchain enabling for RISC-V support
        - OS-specific libs for Tock
 - Toolchain enhancements
    - support new instructions (wip)
    - build / link multiple Tock apps
    - code size optimization
      - support for linker relaxation
      - support for tp- relative addressing for apps vs gp- used for kernel (?)
      - -Oz general improvements
   - on-target code coverage for embedded
       - replace 64-bit counters with 32-bit or 8-bit flags to save data & code
       - download coverage data from target
   - HWASAN for RISC-V
   - toolchain stabilization 
 > [end presentation]
 > - amit: for the certified crypto library, reasonable to think of it as basically a hardware interface that happens to be implemented in software?
 > - amit: probably written in C, exposes limited symbols?
 > - vadim: yeah, basically it's a vtable with 4 function pointers as APIs
 > - andrey: it actually has more than a single layer; there's those interfaces, but also some key management to be done in Tock / in Rust; more than just closed crypto lib
 > - andrey: e.g. key handles from flash, from applications, or from hardware; thus need additional abstraction on top of just primitives
 > - amit: specifically thinking about cases like Bluetooth, where external libraries also expected to own the privileged mode of the CPU; sounds like this is more reasonable to think of as a HW wrapper++, but more amenable to integration
 > - vadim: up to this point, need to be part of the core; have another idea in mind where only certain tasks have access to certain modules (and certain hardware); maybe with PMP support, only user mode task with access can access HW
 > - amit: that seems reasonable
 > - vadim: design choice here: generic API in Tock; or as an isolated app communicating back via IPC; unclear at this point
 > - amit: pitfall in other systems: some register in HW that can expose channels (e.g. DMA registers; or pinmux registers) that can be unexpected leakage mechanisms
 > - andrey: the bigger question: can this be made part of TockOS? [should we upstream]
 > - amit: this is on OT which is open source, and the crypto is open source, so.. yes
 > - leon: with the FIPS certification, what obstacles would the Rust layer above the API possibly present?
 > - vadim: FIPS basically asserts correct implementation of algorithms, correct random, correct testing; don't think a stability of APIs or Rust layer affect this
 > - vadim: big this is stability of the library; that would trigger re-cert. That's what motivates library as blob.
 > - leon: so no limitation on layers above, so long as there is a clean interface for HW or library backend
 > - vadim: yeah
 > - vadim: maybe extra capsules somewhere for key management or other policies / needs, but generally yeah; isolated blobs will work
 > - amit: curious to hear more about what the filesystem would look like? all the way to directories? KV Store? what's the use case?
 > - vadim: something like KV store partitioned to each app likely enough; or flat directory, one per app
 > - vadim: use case, app uses some kind of handle for state or configuration
 > - leon: would like to point to tock-dev persistent storage discussion; would like to see some of these needs captured there
 > - andrey: arch still under development; have a working impl in another OS for file system; more journal-like approach there; will propose similar here
 > - andrey: still under active dev, don't expect to see anything in next week, but something on the sooner side
 > - amit: and ACL needs?
 > - vadim: basically just each app has access to its own objects
 > - andrey: not necessarily physical partition, just data projection
 > - vadim: each flash page could have data from multiple apps; OS in charge of controlling data access
 > - amit: is this primarily for on-chip or external?
 > - andrey: this is for on-chip; have external, but different application
 > - vadim: looking for flash that could allow two writes to the same place; some flash allow only 1 write, some allow many, ours allows 2 before requiring an erase (get one 1->0 reset before an erase); allows some optimizations, but not all flashes support this
 > - andrey: that shouldn't necessarily limit how much we upstream here; the flash write layer should hopefully be below a filesystem
 > - vadim: yes, but the interface then needs to expose things like how many writes to given addresses


