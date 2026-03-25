# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Infineon Technologies AG 2026.

if { [info exists env(OPENOCD_ROOT)] } {
    add_script_search_dir $env(OPENOCD_ROOT)/scripts
} else {
    add_script_search_dir /opt/ModusToolboxProgtools-1.7/openocd/scripts
}

if { [info exists env(DEBUG_CERTIFICATE)] } {
    set DEBUG_CERTIFICATE $env(DEBUG_CERTIFICATE)
} else {
    set DEBUG_CERTIFICATE "./packets/debug_token.bin"
}

source [find interface/kitprog3.cfg]
transport select swd

# Select the right CMSIS-DAP probe if multiple are connected.
# For Infineon: start ModusToolboxProgtools and extract the serial number here
# Info : Selected Programmer: KitProg3 CMSIS-DAP BULK-*0E18119E02272400*
if { [info exists env(CMSIS_DAP_SERIAL)] } {
    cmsis_dap_serial $env(CMSIS_DAP_SERIAL)
} 

set TARGET_VARIANT psc3m5
source [find target/infineon/psc3.cfg]

psc3.cm33 configure -rtos auto -rtos-wipe-on-reset-halt 1
gdb_breakpoint_override hard
