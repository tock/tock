This folder contains original (or close to original) scripts from the AmbiqSuite SDK for configuring and using the (in-silicon?) bootloader of the APollo3.

- configure_silicon_bootloader: generate info0.bin and upload to the Apollo3 with a debugger to enable bootloader
- use_silicon_bootloader: convert your application.bin into an OTA image blob, then convert that to a wired update image blob, and then push that onto an Apollo3 that has the silicon bootloader configured