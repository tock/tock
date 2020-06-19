Code Review
===========

## Kernel Code Review

Changes to the Tock OS kernel (in the kernel/ directory of the repository) are
reviewed by the Tock core working group. However, not all ports of Tock (which
include chip crates, board crates, and hardware-specific capsules) are
maintained by the Tock core working group.

The Tock repository must document which working group (if any) is responsible
for each hardware-specific crate or capsule.

## Third-Party Dependencies

Tock OS repositories permit third party dependencies for critical components
that are impractical to author directly. Each repository containing embedded
code (including [tock](https://www.github.com/tock/tock),
[libtock-c](https://www.github.com/tock/libtock-c), and
[libtock-rs](https://www.github.com/tock/libtock-rs)) must have a written policy
documenting:

1. All unaudited required dependencies. For example, Tock depends on Rust's
   [libcore](https://doc.rust-lang.org/core/index.html), and does not audit
   `libcore`'s source.

1. How to avoid pulling in unaudited optional dependencies.

A dependency may be audited by vendoring it into the repository and putting it
through code review. This policy does not currently apply to host-side tools,
such as elf2tab and tockloader, but may be extended in the future.
