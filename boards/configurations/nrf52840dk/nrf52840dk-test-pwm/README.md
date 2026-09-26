nRF52840-DK PWM Test Board
===================================

This is a minimal kernel for testing the PWM stack without virtualization.

Each of the four onboard LED pins is driven by its own, dedicated PWM
hardware peripheral (PWM0-PWM3).

Because those pins are being driven as PWM outputs, the LED driver is
intercepted in `with_driver()` and always returns an error so userspace
cannot also try to drive the same pins as plain digital outputs.
