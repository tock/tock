---
driver number: 0x00007
---

# Analog Comparator

## Overview

The Analog Comparator driver allows userspace to compare voltages and
gives an output depending on this comparison. 

Analog Comparators (ACs) can be first of all be configured in the 'normal mode',
in which each AC performs a comparison of two voltages. The other option for
comparison is the 'window mode', in which a voltage can be compared against a
window of two voltages.

A specific AC is referred to as ACx, where x is any number from 0 to n, and n is
the index of the last AC module.

## Command

  * ### Command number: `0`

    **Description**: Does the driver exist?

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: `SUCCESS` if it exists, otherwise `ENODEVICE`

  * ### Command number: `1`

    **Description**: Enabling the ACIFC by activating the clock and the
    ACs (Analog Comparators). Currently always-on mode is
    enabled, allowing a measurement on an AC to be made quickly after a
    measurement is triggered, without waiting for the AC startup time. The
    drawback is that the AC is always on, leading to a higher power dissipation.

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: `SUCCESS` if the initialization was succesful, `FAIL` if it 
    failed.


  * ### Command number: `2`

    **Description**: Do a comparison of two inputs, referred to as the positive
    input voltage and the negative input voltage (Vp and Vn).

    **Argument 1**: The index of the Analog Comparator for which the comparison
    needs to be made, starting at 0.

    **Argument 2**: unused

    **Returns**: The output of this function is `True` when Vp > Vn, and 
    `False` if Vp < Vn.

  * ### Command number: `3`

    **Description**: Do a comparison of three input voltages. Two ACs, ACx and
    ACx+1, are grouped for this comparison depending on the window chosen. They
    each have a positive and negative input: we define these respectively as (Vp_x
    and Vn_x) for ACx and (Vp_x+1 and Vn_x+1) for ACx+1. The sources of the
    negative input of ACx (Vn_x) and the positive input of ACx+1 (Vp_x+1) must be
    connected together externally as a prerequisite to use the windowed mode. These
    then together form the common voltage Vcommon.  The way the windowed mode
    works is then as follows. The two remaining sources, being the positive input
    of ACx (Vp_x) and negative input of ACx+1 (Vn_x+1) define an upper and a lower
    bound of a window. The result of the comparison then depends on Vcommon lying
    inside of outside of this window.

    **Argument 1**: The index of the window for which to do a window comparison,
    starting at 0. For example, window 0 is the combination of ACx and ACx+1,
    window 1 is the combination of ACx+2 and ACx+3 etcetera.

    **Argument 2**: unused

    **Returns**: When the voltage of Vcommon lies inside the window defined by
    the positive input of ACx and the negative input of ACx+1, the output will be
    `True`; it will be `False` if it lies outside of the window.  Specifically, the
    output will be `True` when Vn_x+1 < Vcommon < Vp_x, and `False` if Vcommon <
    Vn_x+1 or Vcommon > Vp_x.

* ### Command number: `4`

    **Description**: Configure interrupts on an analog comparator.
    After enabling interrupts, the callback set in subscribe will be called
    when the positive input voltage is higher than the negative input voltage
    (Vp > Vn).

    **Argument 1**: The index of the Analog Comparator for which the comparison
    needs to be made, starting at 0.

    **Argument 2**: unused

    **Returns**: `SUCCESS` if enabling interrupts was succesful, `EINVAL` if 
    an invalid value of ac was set.
