Nordic Family of Boards
=======================

There are several closely related boards from Nordic supported, or previously
supported, by Tock. Unfortunately, the Nordic naming scheme is a bit confusing,
so two notions get conflated:

  - The `nrf52xxx` family of chips, specifically for Tock the nrf52832 and nrf52840, is often referred to as the "nrf52" family of chips.
  - The development kit for the nrf52840 is called the nrf52840dk; the development kit for the nrf52832 is called nrf52dk.

The result is:

  - "nrf52" => nrf52832 chip or nrf52840 chip, which are on the nrf52dk and nrf52840dk respectively
  - "nrf52dk" => development board with the nrf52832 chip
  - "nrf52840dk" => development board with the nrf52840 chip
  - "nrf52840_dongle" => minimalist board with the nrf52840 chip

This naming matches the products released by Nordic, but users should be careful.

Additionally, the acd52832 board is a platform developed by Aconno that
features a nrf52832 chip.


Tock Hierarchy
--------------

As much is shared across these platforms, Tock uses an "nrf52dk_base" crate.
This is a library crate that contains code shared by boards that include any
chips from the nrf52 family.


Legacy Boards
-------------

Tock 1.3 was the last release with support for the nrf51, a old chip in this
family that has a Cortex M0 with no MPU.

Code for the nrf51dk platform is available in the
[tock archive](https://github.com/tock/tock-archive/tree/master/nrf51dk).
