The Hail Platform
=================

*Note: this is a slightly modified version of the Hail platform,
 used in the SOSP course on Tock. The major modification is that the
 ambient light system call driver has been replaced with a capsule
 that directly samples the sensor within the kernel. The capsule 
 is correctly instantiated in the boot sequence but its functional
 code is empty, to be filled in by the person taking the course.*

Hail is an embedded IoT module for running Tock.
It is programmable over USB, uses BLE for wireless, includes
temperature, humidity, and light sensors, and has an onboard accelerometer.
Further, it conforms to the Particle Photon form-factor.

For Hail schematics or other hardware details,
[visit the Hail repository](https://github.com/lab11/hail).

