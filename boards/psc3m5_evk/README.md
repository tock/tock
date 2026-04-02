# PSOC™ Control C3M5 Evaluation Kit

<img src="https://assets.infineon.com/is/image/infineon/kit-psc3m5-evk-main-picture-kit-psc3m5-evk.png" width="40%">

The
[PSOC™ Control C3M5 Evaluation Kit](https://www.infineon.com/evaluation-board/kit-psc3m5-evk)
is a evaluation board for the PSOC Control C3M5 microcontroller, which is based
on the Arm Cortex-M33 architecture.

## Getting started

Install `probe-rs`.\
OR\
OpenOCD from
[ModusToolbox™ Programming Tools](https://softwaretools.infineon.com/tools/com.ifx.tb.tool.modustoolboxprogtools)

## Flashing the kernel

The kernel can be programmed by going inside the board's directory and running:

```bash
$ make flash # program for OpenOCD
```

## Flashing an app

Apps are built out-of-tree. Once an app is built, you must add the path to the
generated TBF in the Makefile (APP variable), then run:

```bash
$ make flash APP=path/to/app.tbf # program for OpenOCD
```

This will generate a new ELF file that can be deployed on the board via gdb and
probe-rs.

## Protection Contexts

Infineon added a security feature called Protection Contexts (PC) to the PSOC
Control C3. This allows the user to create up to 8 different contexts, each with
its own set of permissions for accessing memory.

These contexts have to be configured with the `edgeprotecttools` CLI tool. From
delivery, the board is configured with all contexts only available in secure
mode. To reset this configuration, follow these steps:

```bash
$ cd edgeprotecttools
# install edgeprotecttools
$ pip install edgeprotecttools
# init configurations
$ edgeprotecttools -t psoc_c3 init
# provision the device with the configuration
$ edgeprotecttools -t psoc_c3 provision-device -p ns_policy/policy_oem_provisioning.json
```

### Troubleshooting

If provisioning does not work because of "ERROR : Unable to read current LCS value", 
you can try to erasing the flash with OpenOCD and then try provisioning again.

```bash
# adapt path to Infineon-OpenOCD if needed
$ /opt/ModusToolboxProgtools-1.7/openocd/bin/openocd -f interface/kitprog3.cfg -c "set ENABLE_ACQUIRE 0" -f target/infineon/psc3.cfg -c "init; reset init; erase_all; shutdown"
$ edgeprotecttools -t psoc_c3 provision-device -p ns_policy/policy_oem_provisioning.json
```
