Cortex-M4F Architecture
======================

Architecture support for Cortex-M4F devices. This largely only re-exports the
correct functions from the Cortex-M crate.

_Note:_ Mainline Tock does not currently have any support for hard FPUs.
This is currently a direct clone of the cortexm4. However, chips wil FPUs
should point to this arch crate to pick up hard float support when it is
integrated.
