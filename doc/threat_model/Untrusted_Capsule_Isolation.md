Untrusted Capsule Isolation
===========================

## Isolation Mechanism

Untrusted capsules are limited to what they should be able to access within
Rust's type system without using `unsafe`. That isolation is implemented by
banning `unsafe` from use in untrusted capsule code. This isolation is
vulnerable to code that exploits compiler bugs or bugs in `unsafe` code in
libraries. When a board integrator chooses to use an untrusted capsule, they are
responsible for auditing the code of the untrusted capsule to detect potentially
malicious behavior. This relies in part on Rust's resistance to underhanded
programming techniques (stealthy obfuscation), and is a weaker form of isolation
than the hardware-backed isolation used to isolate the kernel (and other
applications) from applications.

## Impact on Kernel API Design

Kernel APIs should be designed to limit the data that untrusted capsules have
access to. Trusted kernel code should use capabilities as necessary in its API
to limit the access that untrusted capsule code has. For example, an API that
would allow an untrusted capsule to access data external to it should require a
"trusted" capability.
