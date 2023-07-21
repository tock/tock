# Tock Core Notes 2023-03-10

Attendees:
 - Branden Ghena
 - Hudson Ayers
 - Alyssa Haroldsen
 - Johnathan Van Why
 - Amit Levy
 - Pat Pannuto

## Updates
* Alyssa: Someone at Google published an open source registers library,
  it creates registers from an svd file.
  Seems to be the only registers library that does not use VolatileCell, so it is
  inherently more sound. Johnathan could probably base his tock-registers
  updates off of it. Link is https://github.com/chipsalliance/caliptra-sw/tree/main/registers
* Hudson: I am planning to merge DeferredCall today if no one has complaints
* Amit: I am going to work on updating the PR to add a default license notice
  to all Rust files, it just uses a sed script with a little manual checking so
  I will run it again on current master
* Johnathan: Make sure to update the .lcignore file so it checks all the files
* Amit: Yeah that should make things much easier to verify which is nice
* Alyssa: DeferredCall is a breaking change, will there be a version bump?
* Hudson: We only have versioning for the userspace <--> kernel interface now,
  not for any changes to crates in the Tock kernel repository that only have interfaces
  to other stuff that will run in the kernel. We might want to do that eventually.
* Alyssa: Yeah it certainly seems like that would be a nice to have so we could know what changes need
  to be made before we do a merge and everything breaks.
* Alyssa: Why doesn't Tock generate `macro\_rules!()` definitions from svd files?
* Hudson: Tock svd2regs.py which does this, but I have not used it
* Johnathan: Opentitan also has an internal tool that does this for its own internal format of svd files.
* Alyssa: I would like to talk about the userspace ordered prints PR, but Phil
  isn't here so maybe that is best left for next week.
