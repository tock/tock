Imix Kernel Tests
==============

This folder contains optional in-kernel tests that can be run by being called from main.rs.
Only tests that require hardware to run belong in this folder.
These tests may conflict with normal operation in cases where hardware peripherals
are not virtualized, and should not be enabled in normal use.
