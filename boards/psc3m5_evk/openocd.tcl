
source [find interface/kitprog3.cfg]

transport select swd
source [find target/infineon/psc3.cfg]
psc3.cm33 configure -rtos auto -rtos-wipe-on-reset-halt 1
gdb_breakpoint_override hard
CDLiveWatchSetup
if {$::ENABLE_ACQUIRE} {
    init
    reset init
}

proc CDLiveWatchSetup {} {
}
