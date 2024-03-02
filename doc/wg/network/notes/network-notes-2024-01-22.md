# Tock Network WG Meeting Notes

- **Date:** January 22, 2024
- **Participants:**
    - Alex Radovici
    - Tyler Potyondy
    - Felix Mada
    - Leon Schuermann
- **Agenda**:
    - 1. Updates

## OpenThread Update

- Tyler: Been working on getting the OpenThread port working. Leon
  helped with using the CMake build system to generate the required
  static libraries. That is working now. Matched the libtock-c
  compiler flags in their build system. Leon has also been working on
  the Encapsulated Functions integration. In libtock-c we simply link
  in the static library.

  Now encountering an error: OpenThread creates a quite large globally
  defined array. During the initialization phase, when it's allocating
  that memory, the program faults. Alex mentioned that this may be an
  issue with the size allocated to the app. Gave it all the memory
  that the nRF board has. Anyone has an idea?
- Leon: Had same issue with LwIP. Did pretty much the same debugging
  steps, happy to do some rubber-duck debugging.
- Alex: Process loading debugging.
- Alex: Other solution: wait for button press and use the procss
  console to get process placement information.
- Leon: Panic message also shows MPU cofiguration.
- Tyler: Been looking at that message. Tried to increase the
  application heap (40kB), should be amply big for this.
- Leon: May also want to look at OpenThread's configuration around
  heap allocation. Ability to wire up an external allocator. It may
  try to allocate memory at an address that isn't backed by any active
  MPU region.
- Alex: You said something about BSS? That should be unrelated.
- Tyler: My understanding -- data section is for initialized global
  variables, BSS is for initialized to zero. When I get the panic
  message, can I infer the location of the BSS section from that?
- Leon: An app in Tock always has two memory regions allowed in the
  MPU: an execute-in-place (XIP) flash, and a section in main
  memory. When the application is first started, it runs an
  initialization routine defined in `crt0.S`. It's passed information
  by the kernel that tells it where its various memory regions are
  located, and where to find its binary. Uses this information to, for
  instance, zero out the BSS section.
- Leon: Another interesting debugging utility could be the "Low Level
  Debug" driver. Provides some primitive syscalls to convey
  information to the kernel and then print on the console.
- Leon: The panic message itself only shows us the regions that are
  accessible, not where the application places the various memory
  sections.
- Alex: Does it fail within the `crt0.S` file?
- Tyler: Not sure.
- Alex: You can add system call tracing to see whether the app does
  memops.
- Tyler: Can see in the panic message that there are memops occurring.
- Alex: The first few memops are occuring immediately on application
  startup. Those should work. Then comes re-location, does it fail in
  that step? Can also try to build a non-relocatable binary.
- Leon: Most productive way forward seems to do a debugging
  session. Let's sync up asynchronously.

## Console Multiplexer

- Alex: Amalia has started on the console multiplexer and got some
  initial results. Leon, did you push the buffer management code
  somewhere?
- Leon: Still have some local, work in progress changes. Looking
  forward to sync up synchronously.
- Alex: Some time next week?
- Leon: This or next week.
