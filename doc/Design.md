Tock Design
===========

Most operating systems provide isolation between components using a process-like
abstraction: each component is given it's own slice of the system memory (for
it's stack, heap, data) that is not accessible by other components. Processes
are great because they provide a convenient abstraction for both isolation and
concurrency. However, on resource-limited systems, like microcontrollers with
much less than 1MB of memory, this approach leads to a trade-off between
isolation granularity and resource consumption.

Tock's architecture resolves this trade-off by using a language sandbox to
isolated components and a cooperative scheduling model for concurrency in the
kernel. As a result, isolation is (more or less) free in terms of resource
consumption at the expense of preemptive scheduling (so a malicious component
could block the system by, e.g., spinning in an infinite loop).

To first order, all components in Tock, including those in the kernel, are
mutually distrustful. Inside the kernel, Tock achieves this with a
language-based isolation abstraction called _capsules_ that incurs no memory or
computation overhead. In user-space, Tock uses (more-or-less) a traditional
process model where process are isolated from the kernel and each other using
hardware protection mechanisms.

In addition, Tock is designed with other embedded systems-specific goals in
mind. Tock favors overall reliability of the system and discourages components
(prevents when possible) from preventing system progress when buggy.

## Architecture

![Tock architecture](architecture.png)

Tock includes three architectural components. A small trusted kernel, written in
Rust, implements a hardware abstraction layer (HAL), scheduler and
platform-specific configuration. Other system components are implemented in one
of two protection mechanisms: capsules, which are compiled with the kernel and
use Rust’s type and module systems for safety, and processes, which use the MPU
for protection at runtime.

System components (an application, driver, virtualization layer, etc.) can be
implemented in either a capsule or process, but each mechanism trades off
concurrency and safety with memory consumption, performance, and granularity.

| Category               | Capsule     | Process        |
| ---------------------- | ----------- | -------------- |
| Protection             | Language    | Hardware       |
| Memory Overhead        | None        | Separate stack |
| Protection Granularity | Fine        | Coarse         |
| Concurrency            | Cooperative | Preemptive     |
| Update at Runtime      | No          | Yes            |

As a result, each is more appropriate for implementing different components. In
general, drivers and virtualization layers are implemented as capsules, while
applications and complex drivers using existing code/libraries, such as
networking stacks, are implemented as processes.

### Capsules

A capsule is a Rust struct and associated functions. Capsules interact with each
other directly, accessing exposed fields and calling functions in other
capsules. Trusted platform configuration code initializes them, giving them
access to any other capsules or kernel resources they need. Capsules can protect
internal state by not exporting certain functions or fields.

Capsules run inside the kernel in privileged hardware mode, but Rust’s type and
module systems protect the core kernel from buggy or malicious capsules. Because
type and memory safety are enforced at compile-time, there is no overhead
associated with safety, and capsules require minimal error checking. For
example, a capsule never has to check the validity of a reference. If the
reference exists, it points to valid memory of the right type. This allows
extremely fine-grained isolation since there is virtually no overhead to
splitting up components.

Rust’s language protection offers strong safety guarantees. Unless a capsule is
able to subvert the Rust type system, it can only access resources explicitly
granted to it, and only in ways permitted by the interfaces those resources
expose. However, because capsules are cooperatively scheduled in the same
single-threaded event loop as the kernel, they must be trusted for system
liveness. If a capsule panics, or does not yield back to the event handler, the
system can only recover by restarting.

### Processes

Processes are independent applications that are isolated from the kernel and run
with reduced privileges in separate execution threads from the kernel. The
kernel schedules processes preemptively, so processes have stronger system
liveness guarantees than capsules. Moreover, uses hardware protection to enforce
process isolation at runtime. This allows processes to be written in any
language and to be safely loaded at runtime.

#### Memory Layout

Processes are isolated from each other, the kernel, and the underlying hardware
explicitly by the hardware Memory Protection Unit (MPU). The MPU limits which
memory addresses a process can access. Accesses outside of a process’s permitted
region result in a fault and trap to the kernel.

Code, stored in flash, is made
accessible with a read-only memory protection region. Each process is allocated
a contiguous region of RAM. One novel aspect of a process is the presence of a
“grant” region at the top of the address space. This is memory allocated to the
process covered by a memory protection region that the process can neither read
nor write. The grant region, discussed below, is needed for the kernel to be able
to borrow memory from a process in order to ensure liveness and safety in
response to system calls.

### Grants

Capsules are not allowed to allocate memory dynamically since dynamic
allocation in the kernel makes it hard to predict if memory will be exhausted.
A single capsule with poor memory management could cause the rest of the kernel
to fail. Moreover, since it uses a single stack, the kernel cannot easily
recover from capsule failures.

However, capsules often need to dynamically allocate memory in response to
process requests. For example, a virtual timer driver must allocate a structure
to hold metadata for each new timer any process creates. Therefore, Tock allows
capsules to dynamically allocate from the memory of a process making a request.

It is unsafe, though, for a capsule to directly hold a reference to process
memory. Processes crash and can be dynamically loaded, so, without explicit
checks throughout the kernel code, it would not be possible to ensure that a
reference to process memory is still valid.

For a capsule to safely allocate memory from a process, the kernel must enforce
three properties:

  1. Allocated memory does not allow capsules to break the type system.

  2. Capsules can only access pointers to process memory while the process is
     alive.

  3. The kernel must be able to reclaim memory from a terminated process.

Tock provides a safe memory allocation mechanism that meets these three
requirements through memory grants. Capsules can allocate data of arbitrary
type from the memory of processes that interact with them. This memory is
allocated from the grant segment.

Just as with buffers passed through allow, references to granted memory are
wrapped in a type-safe struct that ensures the process is still alive before
dereferencing. Unlike shared buffers, which can only be a buffer type in a
capsule, granted memory can be defined as any type. Therefore, processes cannot
access this memory since doing so might violate type-safety.

