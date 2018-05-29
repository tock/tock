---
driver number: 0x00001
---

# Console

## Overview

The console driver allows the process to write buffers to serial device. To
write a buffer, a process must share the buffer using `allow` then initiate the
write using a `command` call. It may also using `subscribe` to receive a
callback when the write has completed.

Once the write has completed, the buffer shared with the driver is released, so
can be deallocated by the process. This also means that it is necessary to
share a buffer for every write transaction, even if it's the same buffer.

## Command

  * ### Command number: `0`

    **Description**: Does the driver exist?

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS if it exists, otherwise ENODEVICE

  * ### Command number: `1`

    **Description**: Initiate a write transaction of a buffer shared using `allow`.
    At the end of the transaction, a callback will be delivered if the process
    has `subscribed`.

    **Argument 1**: The maximum number of bytes to write.

    **Argument 2**: unused

    **Returns**: SUCCESS if the command was successful, EBUSY if no buffer was
    shared, or ENOMEM if the driver failed to allocate memory for the
    transaction.

  * ### Command number: `2`

    **Description**: Initiate a read transaction into a buffer shared using `allow`.
    At the end of the transaction, a callback will be delivered if the process
    has `subscribed` to read events using `subscribe number` 2.

    **Argument 1**: The maximum number of bytes to write.

    **Argument 2**: unused

    **Returns**: SUCCESS if the command was successful, EBUSY if no buffer was
    shared, or ENOMEM if the driver failed to allocate memory for the
    transaction.

  * ### Command number: `3`

    **Description**: Abort any ongoing read transactions.
    Any received bytes will be delivered via callback if the process
    has `subscribed` to read events using `subscribe number` 2.

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS if the command was successful, EBUSY if no buffer was
    shared, or ENOMEM if the driver failed to allocate memory for the
    transaction.

## Subscribe

  * ### Subscribe number: `1`

    **Description**: Subscribe to write transaction completion event. The
    callback will be called whenever a write transaction completes.

    **Callback signature**: The callback receives a single argument, the number
    of bytes written in the transaction. The value of the remaining arguments
    is undefined.

    **Returns**: SUCCESS if the subscribe was successful or ENOMEM if the
    driver failed to allocate memory for the transaction.

  * ### Subscribe number: `2`

    **Description**: Subscribe to read transaction completion event. The
    callback will be called whenever a read transaction completes.

    **Callback signature**: The callback receives a single argument, the number
    of bytes read in the transaction. The value of the remaining arguments
    is undefined.

    **Returns**: SUCCESS if the subscribe was successful or ENOMEM if the
    driver failed to allocate memory for the transaction.

## Allow

  * ### Allow number: `1`

    **Description**: Sets a shared buffer to be used as a source of data for
    the next write transaction. A shared buffer is released if it is replaced
    by a subsequent call and after a write transaction is completed. Replacing
    the buffer after beginning a write transaction but before receiving a
    completion callback is undefined (most likely either the original buffer or
    new buffer will be written in its entirety but not both).

    **Returns**: SUCCESS if the subscribe was successful or ENOMEM if the
    driver failed to allocate memory for the transaction.

  * ### Allow number: `2`

    **Description**: Sets a shared buffer to be read into by the next read
    transaction. A shared buffer is released in two cases: if it is replaced by
    a subsequent call or after a read transaction is completed. Replacing the
    buffer after beginning a read transaction but before receiving a completion
    callback is undefined (most likely either the original buffer or new buffer
    will be sent in its entirety but not both).

    **Returns**: SUCCESS if the subscribe was successful or ENOMEM if the
    driver failed to allocate memory for the transaction.

