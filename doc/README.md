Tock Documentation
==================

Here you can find guides on how Tock works, as well as short [tutorials](tutorials)
and longer [workshop-style courses](courses) on how to use Tock, and
[reference documents](reference) that detail internal interfaces.

Tock Guides
-----------

### Overview and Design of Tock
- **[Overview](Overview.md)** - Overview of the OS and this repository.
- **[Design](Design.md)** - Design of the Tock primitives that make safety and security possible.
- **[Networking Stack](Networking_Stack.md)** - Design of the networking stack in Tock.

### Tock Implementation
- **[Lifetimes](Lifetimes.md)** - How Rust lifetimes are used in Tock.
- **[Mutable References](Mutable_References.md)** - How Tock safely shares resources between components.
- **[Compilation](Compilation.md)** - How the kernel and applications are compiled.
- **[Tock Binary Format](TockBinaryFormat.md)** - How Tock application binaries
are specified.
- **[Memory Layout](Memory_Layout.md)** - How the chip memory is divided for Tock.
- **[Startup](Startup.md)** - What happens when Tock boots.
- **[Syscalls](Syscalls.md)** - Kernel/Userland abstraction.
- **[Userland](Userland.md)** - Description of userland applications.

### Interface Details
- **[Syscall Interfaces](syscalls)** - API between userland and the kernel.
- **[Internal Kernel Interfaces](reference)** - Hardware Interface Layers (HILs) for kernel components.

### Tock Setup and Usage
- **[Getting Started](Getting_Started.md)** - Installing the Tock toolchain and programming hardware.

### Tutorials and Courses
- **[Quick Tutorials](tutorials)** - Specific tutorials that walk through features of Tock.
- **[Longer Courses](courses)** - Longer workshops on multiple aspects of Tock.
