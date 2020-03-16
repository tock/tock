Untrusted Capsule Isolation
===========================

## Isolation Mechanism

Untrusted capsules are limited to what they can access within
Rust's type system without using `unsafe`. That isolation is implemented by
banning `unsafe` from use in untrusted capsule code and by banning the use of
unaudited libraries (except those that ship with Rust's toolchain) in kernel
code. This isolation is
vulnerable to code that exploits compiler bugs or bugs in `unsafe` code in
toolchain libraries. When a board integrator chooses to use an untrusted
capsule, they are responsible for auditing the code of the untrusted capsule to
confirm the policies are followed and to detect potentially
malicious behavior. The use of Rust's type system as a security isolation
mechanism relies in part on Rust's resistance to underhanded
programming techniques (stealthy obfuscation), and is a weaker form of isolation
than the hardware-backed isolation used to isolate the kernel (and other
applications) from applications.

## Impact on Kernel API Design

Kernel APIs should be designed to limit the data that untrusted capsules have
access to. Trusted kernel code should use capabilities as necessary in its API
to limit the access that untrusted capsule code has. For example, an API that
allows its clients to access data that is not owned by either the API or caller
should require a "trusted" capability.
