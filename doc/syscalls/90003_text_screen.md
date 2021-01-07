---
driver number: 0x90003
---

# Text Screen

## Overview

The screen driver allows the process to write data to a text 
screen like an LCD display.

## Command

  * ### Command number: `0`

    **Description**: Does the driver exist?

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS if it exists, otherwise ENODEVICE

  * ### Command number: `1` 

    **Description**: Get the screen's resolution (in characters)

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS followed by a callback with the rotation value, EBUSY if another command is in progress.

  * ### Command number: `2`

    **Description**: Turn the display on

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS if the command was successful, EBUSY if another command is in progress.

  * ### Command number: `3`

    **Description**: Turn the display off

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS if the command was successful, EBUSY if another command is in progress.

  * ### Command number: `4`

    **Description**: Turn blink mode on

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS if the command was successful, EBUSY if another command is in progress.

  * ### Command number: `5`

    **Description**: Turn blink mode off

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS if the command was successful, EBUSY if another command is in progress.
  
  * ### Command number: `6`

    **Description**: Show cursor

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS if the command was successful, EBUSY if another command is in progress.

  * ### Command number: `7` 

    **Description**: Hide cursor

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS_U32 with a u32 being the number of supported resolutions (minimum 1), EBUSY if another command is in progress.

  * ### Command number: `8` 

    **Description**: Initiate a write transaction of a buffer shared using `allow_readonly`. This will write the characters in the
    buffer to the text screen.
    At the end of the transaction, a callback will be delivered if the process
    has `subscribed`.

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS followed by a callback when it is done, EBUSY if another command is in progress.
  
  * ### Command number: `9` 

    **Description**: Clear screen

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS_U32 with a u32 being the number of supported color depth (minimum 1), EBUSY if another command is in progress.

  * ### Command number: `10` 

    **Description**: Set the cursor position at (0, 0)

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS followed by a callback with the rotation value, EBUSY if another command is in progress.

  * ### Command number: `11` 

    **Description**: Set cursor position

    **Argument 1**: row

    **Argument 2**: column

    **Returns**: SUCCESS followed by a callback with the resolution, EBUSY if another command is in progress.

## Subscribe

  * ### Subscribe number: `0`

    **Description**: Subscribe to to all commands.

    **Callback signature**: The callback receives different arguments 
    depending on the issued command.

    **Returns**: SUCCESS if the subscribe was successful.

## Allow ReadOnly

  * ### Allow number: `0`

    **Description**: Sets a shared buffer to be used as a source of data for
    the next write transaction. A shared buffer is released if it is replaced
    by a subsequent call and after a write transaction is completed. Replacing
    the buffer after beginning a write transaction but before receiving a
    completion callback is undefined (most likely either the original buffer or
    new buffer will be written in its entirety but not both).

    **Returns**: SUCCESS if the subscribe was successful, INVAL if the buffer's length is not a multiple of the color depth length. 

