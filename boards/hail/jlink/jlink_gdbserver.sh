#!/usr/bin/env sh
JLinkGDBServer -device ATSAM4LC8C -speed 1200 -if swd -AutoConnect 1 -port 2331
