# Tock Network WG Meeting Notes

- **Date:** November 18, 2024
- **Participants:**
    - Tyler Potyondy
    - Branden Ghena
    - Leon Schuermann
    - Felix Mada
- **Agenda**
    1. Updates
    2. 15.4 Power Management
- **References:**
    - [15.4 Bug Issue](https://github.com/tock/tock/issues/4227)


## Updates
- Tyler: Have TX test on Treadmill. Installs dependencies and checks results through serial outputs. Wireshark could be integrated for a higher quality test in the future.
- Tyler: We're hoping to include that in the general CI in the mid-term future. Maybe for code with a 15.4 tag or networking tag perhaps? We'll discuss further.
- Branden: And just having something runnable in Treadmill will help testing for Tutorial sessions, right?
- Tyler: For sure. And it turns out OpenThread and the 15.4 drivers both stress different parts of the stack. So automating things will help
- Branden: I have a student team working on an independent study with me this coming Winter quarter to play with WiFi on the RP2040 in Tock. Depending on the progress Alex's group makes, we'll either work on a SPI driver, on a WiFi driver, or play with something that exists. Pretty flexible here. No progress yet, as they're currently being trained on Tock, Rust, SPI, etc.


## 15.4 Power Management
- Tyler: Background here is that OpenThread changes separated the userspace and 15.4 drivers. There's a PHY layer driver that exposes that to userspace, with no packet forming or services. That's the driver OpenThread uses, which Brad iterated on. There's also the 15.4 driver stack which has been in Tock longer with a radio layer and MAC and the userspace driver is a client. It's a little clunky that there are two separate drivers, but that's the current state.
- Tyler: The default for that 15.4 driver previously just leaves the radio on at all times. Just burns power. Brad switched that in updates to make it off by default and you have to switch it on with a call. The problem is that the userspace driver for this standard 15.4 stack sits at the link layer and doesn't have access directly to the radio to turn it on/off. So we need to come up with some way to enable the radio
- Tyler: Background here was that a user had a bug on the Microbit and Amit tried 15.4 on the nRF52833 and nothing worked. There were two issues, one a buffer size in libtock-sync and the other is that the radio is just off
- Branden: Back to OpenThread for a second, does it turn the radio on/off?
- Tyler: Yes, it does. There's a syscall to power the radio that the OpenThread stack uses. In the 15.4 driver though, that functionality doesn't exist
- Branden: Second question, for the 15.4 driver do we want to power cycle it or do we want to just turn it on and leave it on?
- Tyler: For now, just some fix seems good. So turning on would be sufficient
- Tyler: So, options:
    1) in the 15.4 driver we have a finalize method for setting it up and we could as part of this turn on the radio, not the best design but it would fix the issue.
    2) whenever we transmit or receive we turn on the radio on first call, but that could add weird issues maybe
- Tyler: A best option would probably turn off the radio after sending packets, but that's more complicated behavior.
- Tyler: There was once an issue on this, which is closed now but still relevant: https://github.com/tock/tock/issues/4227
- Branden: Third option: Brad changed something so the radio is off by default, but couldn't we change it back to be on by default?
- Tyler: No, I think that's bad. No reason to be on if it's not needed. If you're doing that, do it in the component. Plus the OpenThread lives on top of that, so now that's off by default until OpenThread turns it on
- Tyler: So, maybe a better version would be turning it on on first use. Amit liked that option
- Branden: Are there timing issues there? What if the radio takes a while to turn on? Is there a callback or blocking function for that?
- Tyler: Yeah, there could be a can of worms there. For OpenThread, it's the app itself doing this, which powers it on, then checks if it's on and delays.
- Branden: There's a fourth option. We could forward radio-power-on to the upper layers and out through the syscall layer. Maybe not even a turn off function, just turn on and check-if-on. Then libtock could implement this turn on followed by check-delay-loop mechanism
- Tyler: That's a good possibility. Would be pretty easy to do. Not sure if it would make our layering mixed and bad
- Branden: Let's take a look at the current interface design to see
    - 15.4 syscall interface: https://github.com/tock/tock/blob/master/capsules/extra/src/ieee802154/driver.rs#L617
    - Networking stack documentation: https://book.tockos.org/doc/networking_stack
- Branden: As I see it, we could add an Enable radio syscall here ("Turn the radio on")
- Branden: You'd have to plumb it, so add a function to the MAC that calls down to the radio
- Tyler: There are more hops in there, through a mux I think. But I wouldn't be worried about a timing concern as the openthread 15.4 PHY interface has a similar call. So pushing the check to the user would be fine. We could mirror that naive approach
- Branden: And for libtock-sync you'd add a function that enables the radio and blocks waiting on it
- Tyler: Actually that already exists. The 15.4 stack uses the same driver number and just ignores some unsupported syscalls depending on which you're using. That's the clunkiness I mentioned
- Branden: I think that's a good option then. Much better than on-by-default, and almost as good as automatically turning it on when transmitting/receiving without the complexity

