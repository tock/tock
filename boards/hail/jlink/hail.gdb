target remote localhost:2331
monitor speed 30
file ../../../target/thumbv7em-none-eabi/release/hail
monitor reset
#
# CPU core initialization (to be done by user)
#
# Set the processor mode
# monitor reg cpsr = 0xd3
# Set auto JTAG speed
monitor speed auto
# Setup GDB FOR FASTER DOWNLOADS
set remote memory-write-packet-size 1024
set remote memory-write-packet-size fixed
# tui enable
# layout split
# layout service_pending_interrupts
b reset_handler
