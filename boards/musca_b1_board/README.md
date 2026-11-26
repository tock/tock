# Musca B1 board

> Musca-B1 is a test chip and Platform Security Architecture (PSA) development platform for IoT subsystems. In addition to the Arm Cortex-M33 based subsystem, and reference architecture for Arm TrustZone-based systems, it has added eFlash features for increased security assurance.
> 
> [src](https://developer.arm.com/Tools%20and%20Software/Musca-B1%20Test%20Chip%20Board)

> :warning: This board is still experimental as timer is not reliable in qemu.

## Getting started

Install `qemu-system-arm` package.

Running qemu

```
APP=/path/to/eg-console.tbf make qemu
```
