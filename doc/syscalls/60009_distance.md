---
driver number: 0x60009
---

# Distance

## Overview

The distance sensor driver allows a process to read the distance measured by a distance sensor. Distance is reported in millimeters.

## Command

  * ### Command number: `0`

    **Description**: Does the driver exist?

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: Success if it exists, otherwise NODEVICE

  * ### Command number: `1`

    **Description**: Initiate a sensor reading. When a reading is ready, a callback will be delivered if the process has `subscribed`.

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: `BUSY` if a reading is already pending, `NOMEM` if there isn't sufficient grant memory available, or `Ok(())` if the sensor reading was initiated successfully.

  * ### Command number: `2`

      **Description**: Get the minimum measurable distance.

      **Argument 1**: unused

      **Argument 2**: unused

      **Returns**: The minimum measurable distance in millimeters.

  * ### Command number: `3`

      **Description**: Get the maximum measurable distance.

      **Argument 1**: unused

      **Argument 2**: unused

      **Returns**: The maximum measurable distance in millimeters.

## Subscribe

  * ### Subscribe number: `0`

    **Description**: Subscribe to distance readings.

    **Callback signature**: The callback receives a single argument, the distance in millimeters.

    **Returns**: Ok(()) if the subscribe was successful or NOMEM if the driver failed to allocate memory to store the callback.
