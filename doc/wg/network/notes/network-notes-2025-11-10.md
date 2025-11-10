# Tock Network WG Meeting Notes

- **Date:** November 10, 2025
- **Participants:**
    - Alex Radovici
    - Branden Ghena
    - Leon Schuermann
    - Tyler Potyondy
- **Agenda:**
    1. Updates
    2. WiFi
- **References:**
    - [Thread Hardware CI](https://github.com/tock/tock-hardware-ci/pull/39)
    - [WiFi PR](https://github.com/tock/tock/pull/4529)
    - [WiFi Example C App](https://gist.github.com/irina-nita/66bb825054c69b2e7232cbd3b4f8fc14)


## Updates
### Thread Hardware CI Testing
- https://github.com/tock/tock-hardware-ci/pull/39
- Leon: Tyler's Thread tests have been merged. Known as "tensile"
- Tyler: Thread network testing. I named it tensile.
- Tyler: Thanks Leon for sending that over the finish line.
- Leon: The abstraction for Treadmill is that you specify the board you want and you get a shell that can run stuff. Tyler wrote scripts that initialize a Pi which has four nRF52840s attached. There's one thread router, and three Tock devices that communicate with it. The python script checks that stuff runs and messages get delivered. Now running every night around 3am! And if it fails it should file a github issue tagging Tyler.
- Tyler: It also tests that libtock-c 802.15.4 test applications. Checks that a payload is received.


## WiFi
- https://github.com/tock/tock/pull/4529
- Alex: WiFi PR is updated. It's pretty close now. Works very well.
- Alex: We'll make a libtock-c library for this too. For now there's a short gist that scans networks and attaches to one. https://gist.github.com/irina-nita/66bb825054c69b2e7232cbd3b4f8fc14
- Alex: This works, and we need comments to polish the code
- Alex: After this gets merged, we should review the WiFi and Ethernet and 15.4 infrastructure and figure out what they can use in-common. We're at a place with general networking infrastructure in Tock!
### Pico Boards
- Branden: There are two different boards here. The Pico has an LED. The PicoW has WiFi (which itself controls an LED). We should have separate board files for the two of them in my opinion.
- Alex: The other option is a configuration that selects which board design to compile. Otherwise there's a lot of duplication.
- Leon: We could have a shared configuration board (like Brad has used for the nRF52840) that the two Pico boards could use. So all the stuff in common wouldn't be duplicated. We do this with nRF52840 boards. There's a set of stuff that configures all the normal stuff, and the main.rs adds the board-specific extensions.
- Branden: That seems like a great choice.
- Alex: Sounds good
### CYW4343 Capsule Organization
- Branden: A question on organization. Right now the CYW4343 crate is in `capsules/`. I think it was there because it originally had an external dependency. That has been moved into the board file, so I think the capsule should move into `capsules/extra`.
- Leon: If there's not a good reason for it to be a separate crate, we should follow precedent of Tock organization right now
- Branden: Right now there are various folders within `capsules/extra/src`, so adding another seems fine
### Bar for PR
- Branden: If you haven't looked yet, I'll warn you that this PR is really large. Tens of new files with dozens-to-hundreds of lines of code each. A good thing though is that it's pretty self-contained. So we should focus on how this connects to the rest of Tock and make sure this doesn't do anything silly, but generally it's okay to not totally understand some parts of this. If they have bugs, they'll only break the WiFi stuff.
- Alex: Sorry it's so large! It just really is a large driver.
- Alex: DMA was a big thing, but really valuable. We'll end up porting other stuff to use this DMA in the future.
### WiFi trait
- https://github.com/OxidosAutomotive/tock/blob/55e97a974bcc991cc19fd486eb129f3863fb2fbc/capsules/extra/src/wifi/device.rs
- Branden: Something to be aware of is that this PR provides a base example for a WiFi trait. It's in the capsule, not an official HIL yet, but it's neat to see a first effort at this.

## IPC
- Branden: Sorry this work didn't get done for this time. I do have a lot more time before our next meeting and I will make progress here for us to all review.
- Leon: Generally looking into IPC research and going to be doing some paper-reading. Seems like an interesting research space generally

