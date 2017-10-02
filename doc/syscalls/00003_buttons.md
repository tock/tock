---
driver number: 0x00003
---

# Buttons

## Overview

The buttons driver allows userspace to receive callbacks when buttons on the
board are pressed (and depressed). This driver can support multiple buttons.

Buttons are indexed in the array starting at 0. The order of the buttons and the
mapping between indexes and actual buttons is set by the kernel in the board's
main file.

## Command

  * ### Command number: `0`

    **Description**: How many buttons are supported on this board.

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: The number of buttons on the board, or `ENODEVICE` if this
    driver is not present on the board.

  * ### Command number: `1`

    **Description**: Enable interrupts for a button. The interrupts will occur
    both when the button is pressed and depressed. The callback will indicate
    which event occurred. This command will succeed even if a callback is
    not registered yet.

    **Argument 1**: The index of the button to enable interrupts for, starting at
    0.

    **Argument 2**: unused

    **Returns**: SUCCESS if the command was successful, ENOMEM if the driver
    cannot support another app, and `EINVAL` if the app is somehow invalid.

  * ### Command number: `2`

    **Description**: Disable the interrupt for a button. This will not remove
    the callback (if one is set).

    **Argument 1**: The index of the button to disable interrupts for, starting at
    0.

    **Argument 2**: unused

    **Returns**: SUCCESS if the command was successful, ENOMEM if the driver
    cannot support another app, and `EINVAL` if the app is somehow invalid.

  * ### Command number: `3`

    **Description**: Read the current state of the button.

    **Argument 1**: The index of the button to read, starting at 0.

    **Argument 2**: unused

    **Returns**: 0 if the button is not currently pressed, and 1 button is
    currently being pressed.

## Subscribe

  * ### Subscribe number: `0`

    **Description**: Subscribe a callback that will fire when any button is
    pressed or depressed. Registering the callback does not have an effect on
    whether any button interrupts are enabled.

    **Callback signature**: The callback receives two arguments. The first is
    the index of the button that was pressed or depressed, and the second is
    whether the button was pressed or depressed. If the button was pressed,
    the second value will be a 1, if the button was released the value will be
    a 0.

    **Returns**: SUCCESS if the subscribe was successful, ENOMEM if the driver
    cannot support another app, and `EINVAL` if the app is somehow invalid.

## Allow

Unused for the LED driver. Will always return `ENOSUPPORT`.

