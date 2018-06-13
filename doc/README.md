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

### Tock Implementation
- **[Lifetimes](Lifetimes.md)** - How Rust lifetimes are used in Tock.
- **[Mutable References](Mutable_References.md)** - How Tock safely shares resources between components.
- **[Soundness](Soundness.md)** - How Tock safely uses unsafe code.
- **[Compilation](Compilation.md)** - How the kernel and applications are compiled.
- **[Tock Binary Format](TockBinaryFormat.md)** - How Tock application binaries are specified.
- **[Memory](Memory.md)** - How memory is divided for Tock, and the corresponding isolation properties.
- **[Startup](Startup.md)** - What happens when Tock boots.
- **[Syscalls](Syscalls.md)** - Kernel/Userland abstraction.
- **[Userland](Userland.md)** - Description of userland applications.
- **[Networking Stack](Networking_Stack.md)** - Design of the networking stack in Tock.

### Interface Details
- **[Syscall Interfaces](syscalls)** - API between userland and the kernel.
- **[Internal Kernel Interfaces](reference)** - Hardware Interface Layers (HILs) for kernel components.

### Tock Setup and Usage
- **[Getting Started](Getting_Started.md)** - Installing the Tock toolchain and programming hardware.
- **[Porting Tock](Porting.md)** - Guide to add new platforms.
- **[Out of Tree Boards](OutOfTree.md)** - Best practices for maintaining boards not in Tock master.
- **[Debugging Help](debugging)** - Guides for various debugging techniques.

### Management of Tock
- **[Code Review Process](CodeReview.md)** - Process for pull request reviews and Tock releases.

### Courses and Tutorials
- **[Courses](courses)** - Workshops on multiple aspects of Tock.
- **[Tutorials](tutorials)** - Tutorials that walk through specific features of
  Tock.
