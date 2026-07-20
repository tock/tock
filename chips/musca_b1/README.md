# Musca-B1

> Musca-B1 is a test chip and Platform Security Architecture (PSA) development platform for IoT subsystems. In addition to the Arm Cortex-M33 based subsystem, and reference architecture for Arm TrustZone-based systems, it has added eFlash features for increased security assurance.
> 
> [src](https://developer.arm.com/Tools%20and%20Software/Musca-B1%20Test%20Chip%20Board)

## Links

* [Technical Overview](https://documentation-service.arm.com/static/60af3c8ee022752339b44a49)
* [Technical Reference Manual](https://documentation-service.arm.com/static/60af3cc5e022752339b44a4c)
* [SVD](https://raw.githubusercontent.com/driveraid/muscab1-pac/refs/heads/master/svd/Musca_B1.svd)

## Notes

Some peripherals are unimplemented by qemu. See [here](https://github.com/qemu/qemu/blob/d41b9b44ac9a9c4d82cc74f59bfd1bdd4ac4014c/hw/arm/musca.c#L61-L89).

```c
struct MuscaMachineState {
    MachineState parent;

    ARMSSE sse;
    /* RAM and flash */
    MemoryRegion ram[MUSCA_MPC_MAX];
    SplitIRQ cpu_irq_splitter[MUSCA_NUMIRQ_MAX];
    SplitIRQ sec_resp_splitter;
    TZPPC ppc[MUSCA_PPC_MAX];
    MemoryRegion container;
    UnimplementedDeviceState eflash[2];
    UnimplementedDeviceState qspi;
    TZMPC mpc[MUSCA_MPC_MAX];
    UnimplementedDeviceState mhu[2];
    UnimplementedDeviceState pwm[3];
    UnimplementedDeviceState i2s;
    PL011State uart[2];
    UnimplementedDeviceState i2c[2];
    UnimplementedDeviceState spi;
    UnimplementedDeviceState scc;
    UnimplementedDeviceState timer;
    PL031State rtc;
    UnimplementedDeviceState pvt;
    UnimplementedDeviceState sdio;
    UnimplementedDeviceState gpio;
    UnimplementedDeviceState cryptoisland;
    Clock *sysclk;
    Clock *s32kclk;
};
```
