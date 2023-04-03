Tock Capsules
=============

Capsules are drivers that live in the kernel and are written in Rust. They are
required to conform to Rust's type system (i.e. no `unsafe`). Capsules are
platform agnostic and provide a range of features:

- Drivers for sensors or other ICs
- Virtualization of hardware resources
- Syscall interfaces for userland applications

When using hardware resources, capsules must only use features provided by the
HIL (hardware interface layer). This ensures they can be used on multiple
microcontrollers and hardware platforms.

Capsules have some flexibility in how they present access to a sensor or
virtualized hardware resource. Some capsules directly implement the `Driver`
trait and can be used by userland applications. Others provide an internal
interface that can be used by other in-kernel capsules as well as a `Driver`
interface for applications.

Capsule Organization
--------------------

Capsules are sub-divided into multiple crates, which can be imported and used
independently. This enables Tock to enforce different policies on a per-crate
basis, for instance whether a given crate is allowed to use external
(non-vendored) dependencies.

Currently, capsules are divided into the following crates:

- [**`core`**](./core): these capsules implement functionality which are
  required for most (if not all) Tock-based systems to operate. For instance,
  these capsules implement basic infrastructure for interacting with timer or
  alarm hardware, exposing UART hardware as console ports, etc.

  This crate further contains virtualizers, which enable a given single
  peripheral to be used by multiple clients. Virtualizers are agnostic over
  their underlying peripherals; they do not implement logic specific to any
  given peripheral device.

  This crate stricly prohibits use of any external (non-vendored and unvetted)
  dependencies.

- [**`extra`**](./extra): this crate contains all remaining capsules;
  specifically capsules which does not fit into any the above categories and
  which does not require any external dependencies.
