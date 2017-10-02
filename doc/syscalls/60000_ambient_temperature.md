---
driver number: 0x60000
---

# Ambient Temperature

## Overview

The ambient temperature driver allows a process to read the ambient temperature
from a sensor. Temperature is reported in degrees centigrate at a precision of
hundredths of degrees

## Command

  * ### Command number: `0`

    **Description**: Does the driver exist?

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: SUCCESS if it exists, otherwise ENODEVICE

  * ### Command number: `1`

    **Description**: Initiate a sensor reading.  When a reading is ready, a
    callback will be delivered if the process has `subscribed`.

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: `EBUSY` if a reading is already pending, `ENOMEM` if there
    isn't sufficient grant memory available, or `SUCCESS` if the sensor reading
    was initiated successfully.

## Subscribe

  * ### Subscribe number: `0`

    **Description**: Subscribe to temperature readings.

    **Callback signature**: The callback receives a single argument, the
    temperature in hundredths of degrees centigrate.

    **Returns**: SUCCESS if the subscribe was successful or ENOMEM if the
    driver failed to allocate memory to store the callback.

