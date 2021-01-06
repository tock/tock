Cortex-M Architecture
=====================

This crate includes shared low-level code for the Cortex-M family of CPU
architectures. In some cases this crate includes multiple versions of the same
code, but targeted towards different Cortex-M versions. In general, if code is
used by multiple Cortex-M variants it is included here.

Boards and chips should not depend on this crate directly. Instead, all of the
relevant modules and features should be exported through the specific Cortex-M
crates (e.g. Cortex-M4), and chips and boards should depend on the more specific
crate.
