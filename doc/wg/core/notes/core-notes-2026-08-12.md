# Tock Meeting Notes 2026-08-12

## Attendees

- Amit Levy
- Brad Campbell
- Branden Ghena
- Johnathan Van Why
- Leon Schuermann
- Pat Pannuto


## Updates

- Johnathan: Have open and/or closed PRs for remaining tock-registers todos.
  - Getting closer to tock-registers release.
- Branden: Updated IPC work to do round-robin for grants. Inspired by existing
  work for round-robin in virtualizers.

## Tock Registers - MMIO Structs

- Register structs can be implemented on any system (e.g., a 64-bit map on a
  32-bit arch).
- This issue is more apparent on x86 port I/O which panics on non-x86
  systems.
- Can apply the same approach to 32-bit and 64-bit platforms.
- Can't use on Tock because we build board crates on host.
- What can we do?
  - Don't compile chips for host. (Have to re-architect build system)
  - Panic at runtime on non-matching-arch platforms. (Ugly impl, but easier to
    port to tock-reg 2.0)
  - Something else?
- Want outside users to be able to extend this, will add test/examples for
  that.


### `unsafe` in register map

- https://github.com/tock/tock-registers/pull/57
- A register map is just a map, no one says you have to _do_ anything with the
  map. That isn't unsafe.
- The Span trait (internal to tock-registers) requires unsafe for the offset
  and for the constructor.
  - Span only exists for real hardware.
- This would 
- There are two fundamental unsafe operations.
  1. Asserting the layout is correct.
  2. Asserting the base pointer is correct.
- The layout matters if the documentation makes strong assertions.
- The layout could be defined very far from where the base pointer is
  specified.
- Not clear how the additional unsafe advances correctness/due diligence
  concerns.


## AppId in IPC

- App store in threat model.
  - https://book.tockos.org/doc/threat_model/secure_app_offload
- Brad: we've been considering this model. I don't think we should scrap it.
- IPC document does not (currently) consider this case
- IPC model could support (in theory) a new registry model that could support
  this
- What is a "correct" service could be up to the apps, and not something the
  kernel can enforce.
- Issue example: Dev B's app wants to talk to ensure it is talking to Dev A's
  app (via a IPC service). With some cryptographic guarantee.
- We could support this with a new registry capsule (having this be capsules
  is a nice design because it makes it extensible).
- App could be signed by app store organizer as a way to allow certain
  services. This is different from the pure app offload use case.


## PanicWriter Safety 

- We never documented what safety requirements
  `PanicWriter::create_panic_writer` requires. We need to document that to
  move forward on managing the unsafe in the kernel.
- What is in https://github.com/tock/tock/pull/5047 is wrong, it documents the
  implementation, but not the caller.
- It would help us to make that function safe.
- The unsafe requirement could be that we only call this from panic.
- Could we have a type that only exists in panics we could use.
- This is similar to our discussion about unique register managers from last
  week.
- More needs to be done to describe what the safety requirements are.

