Cortex-M v7m Architecture
=========================

This crate includes shared low-level code for the Cortex-M v7m family of CPU
architectures.

Boards and chips should not depend on this crate directly. Instead, all of the
relevant modules and features should be exported through the specific Cortex-M
crates (e.g. Cortex-M4), and chips and boards should depend on the more specific
crate.
