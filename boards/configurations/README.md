Configuration Boards
====================

This folder contains Tock kernel configurations for boards designed for very
specific purposes. These are not expected to be general purpose boards (the root
boards/ directory stores those), but instead these boards are configured to
expose specific functionality, likely for testing.

Many functions in Tock are configurable, and each board is free to select the
preferred configuration based on hardware features or intended use cases. This
often means, however, that there are many kernel configurations that _no_ boards
use, making it difficult to test those configurations.

For example, checking process credentials can use different credential policies.
A configuration board can be configured with a specific credential checker, even
if no root-level board wants to use that configuration.

Directory Structure and Naming
------------------------------

Boards in this `boards/configurations` directory should be organized by the root
board type. The name must be `<board>-test-[configuration]`. For example:

```
boards/configurations/
  nrf52840dk/
    nrf52840k-test-[configuration1]
    nrf52840k-test-[configuration2]
  imix/
    imix-test-[configuration1]
    imix-test-[configuration2]
```

Each specific board configuration for each root board should have a descriptive
name. For example, if a configuration of the nrf52840dk is designed for running
kernel tests, the board might be called `nrf52840dk/nrf52840dk-test-kernel`.
