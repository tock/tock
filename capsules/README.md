Tock Capsules
=============

Capsules are drivers that live in the kernel and are written in Rust. They are required
to conform to Rust's type system (i.e. no `unsafe`). Capules are platform agnostic and
provide a range of features:
- Drivers for sensors or other ICs
- Virtualization of hardware resources
- Syscall interfaces for userland applications

When using hardware resources, capsules must only use features provided by the HIL (hardware
interface layer). This ensures they can be used on multiple microcontrollers and hardware
platforms.

Capsules have some flexibility in how they present access to a sensor or virtualized hardware
resource. Some capsules directly implement the `Driver` trait and can be used by userland
applications. Others provide an internal interface that can be used by other in-kernel
capsules as well as a `Driver` interface for applications.
