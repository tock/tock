# Tock Core Notes 2023-02-24

Attendees:
- Leon Schuermann
- Brad Campbell
- Alexandru Radovici
- Viswajith Govinda Rajan
- Johnathan Van Why
- Hudson Ayers

# Updates

- JVW: New design for Tock registers PR. Idea came up to only support MMIO (not
  RISC-V CSRs). Need to work on a design to see how the MMIO only approach
  compares to the more general.
- Hudson: We would lose support for in memory registers and RISC-V CSRs?
- Yes, if we want it to be separate.
- Extensibility for tock-registers is challenging. Could integrate.
- Do other tock-registers users use the extensibility options?
- Do litex registers use extensibility?
- Leon: Maybe, but should just clean that up to avoid the issue if needed.
- We do use in memory registers.
- Brad: part of this is to understand

- Leon: PR#3396. Still open. Still hard to rebase.

- Alex: presented at Rust Nation.
- 40 people. Github classroom failed.
- Will submit PR with tutorial.
  - Integrate in to tock book?
  - Book is a one-stop-shop for tutorials/getting started.
- https://github.com/UPB-RustWorkshop/rust-nation-template
- Updated libtock-rs to export modules correctly
- Difficult to get apps to load, need addresses to match
  - elf2tab --protected-size to buffer changes in TBF header
  - issues with app name size
  - way to customize linker file?
  - no auto layout option to libtock runtime
  - might need to add support outside of libtock-rs to build for slots
  - libtock-c handles this

