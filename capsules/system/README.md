System Tock Capsules
==================

System capsules are largely the same as other (i.e., core and extra) capsules,
in that they are logical software modules that contain untrusted code that
cannot use `unsafe`. The difference is that system capsules implement non-HIL
interfaces defined in the core kernel crate and extend the functionality of the
core kernel.

These capsules are used the same way as other capsules in that they are
instantiated in board main.rs files (often using components). However, these
capsule objects are passed to the core kernel.
