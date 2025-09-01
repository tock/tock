Screen-Related Capsules
=======================

These capsules provide various features and support for using screens. These are
not drivers for specific screens, but instead apply to any screen.

Userspace Drivers for the Screen System Call
--------------------------------------------

These capsules implement `SyscallDriver`.

- **[Screen](screen.rs)**: Displays and screens.
- **[Screen Shared](screen_shared.rs)**: App-specific screen windows.

Virtual Peripherals Using the Screen
------------------------------------

Boards that have a screen but not a particular user-facing peripheral
(e.g., LEDs) can emulate the peripheral by drawing it on the screen. These
capsules implement this virtual functionality.

- **[Screen On Led](screen_on_led.rs)**: Draw fake LEDs on the screen.

Adapters for Converting Between Screen Formats
----------------------------------------------

These convert between screen formats, such as between pixel formats. For
example, an upper layer may be writing for a monochrome (e.g., one
bit-per-pixel) display, but the actual screen might be an RGB display.

- **[Screen Adapters](src/screen_adapters.rs)**: Adapters to convert pixel
    formats for implementations of the `Screen` HIL, such as
    `ScreenARGB8888ToMono8BitPage`.
