---
driver number: 0x00007
---

# Analog Comparator

## Overview

The Analog Comparator driver allows userspace to compare voltages and
gives an output depending on this comparison. 

Analog Comparators (ACs) can be first of all be configured in the 'normal
mode', in which each AC performs a single comparison of two voltages. They can
also be configured to send an interrupt as soon as a voltage is higher than another voltage, i.e. when a voltage exceeds a certain threshold. 

A specific AC is referred to as ACx, where x is any number from 0 to n, and n is
the index of the last AC module.

## Command

  * ### Command number: `0`

    **Description**: Does the driver exist?

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: `SUCCESS` if it exists, otherwise `ENODEVICE`

  * ### Command number: `1`

    **Description**: Do a comparison of two inputs, referred to as the positive
    input voltage and the negative input voltage (Vp and Vn).

    **Argument 1**: The index of the Analog Comparator for which the comparison
    needs to be made, starting at 0.

    **Argument 2**: unused

    **Returns**: The output of this function is `True` when Vp > Vn, and 
    `False` if Vp < Vn.

* ### Command number: `2`

    **Description**: Start interrupts on an analog comparator. This analog
    comparator will then listen, and the callback set in subscribe will be
    called when the positive input voltage is higher than the negative input 
    voltage (Vp > Vn).

    **Argument 1**: The index of the Analog Comparator for which the comparison
    needs to be made, starting at 0.

    **Argument 2**: unused

    **Returns**: `SUCCESS` if starting interrupts was succesful.

* ### Command number: `4`

    **Description**: Stop interrupts on an analog comparator. 

    **Argument 1**: The index of the Analog Comparator for which the comparison
    needs to be made, starting at 0.

    **Argument 2**: unused

    **Returns**: `SUCCESS` if stopping interrupts was succesful.
