# Tock Core Notes 2023-06-09

 - Brad Campbell
 - Branden Ghena
 - Johnathan Van Why
 - Alyssa Haroldson
 - Pat Pannuto

# Updates
 - Brad: Bors is failing all of my PRs and I need to figure out why
 - Brad: TickV support in Tockloader. Can read/write stores on flash or in a local binary file. This is a little tricky on the nRF52840 because the only way to write external flash is through Nordic's poorly documented, closed-source tool. Writing seems to have issues. It's resolvable hopefully. But generally this lets us interact with TickV databases from the host.
 - Brad: From Alistair - he demonstrated that his LoRa stuff is working!

# Agenda
## Outstanding PRs
 - Branden: Brad, are any of your PRs holding you back that we haven't reviewed them.
 - Brad: Yes, looking.
 - Brad: https://github.com/tock/tock/pull/3464 removes the last pub static mut and seems uncontroversial
 - Alyssa: Yay
 - Brad: https://github.com/tock/tock/pull/3453 is an ADC implementation for the nRF. The ADC HIL was written for the SAM4L, so the major challenge with the nRF is that the range of frequencies you can sample at with the nRF is not as wide as the SAM4L.
 - Branden: Without using another timer peripheral, right? (yes)
 - Branden: What is the problem with the HIL?
 - Brad: Well, if you write an app for the SAM4L and port to the nRF it won't work.
 - Branden: It would get an error right?
 - Brad: Not currently. That could be a resolution
 - Brad: https://github.com/tock/tock/pull/3448 also needs review. Nordic implemented the hardware for doing Bluetooth, so anything the Bluetooth spec didn't require they didn't support. So for our HILs, we can do encryption, but decryption isn't supported.
 - Branden: How does that make sense? Doesn't BLE send AES both directions?
 - Brad: Not sure. I'd guess it's not required.
 - Brad: Also, I don't know of any way to use the cryptocell hardware without the precompiled binary they provide. So this is likely as good as we'll get.
 - Brad: https://github.com/tock/tock/pull/3466 turns off SPI on the nRF when it's not in use. Necessary so we can read from external flash as both the kernel and JTAG need to access the same pins. Probably what we wanted anyways. Part of the pesky problem where the SPI HIL has the "init" function, and it's not clear what that needs to do. So now read/write turns on, does the operation, and then turns off.
 - Brad: Finally, the rest of the PRs are more minor or really for Alistair. Mostly TicKV related


## Tutorial
 - Brad: I wanted to hear from Tock people about tutorial planning. What are people feeling? It doesn't seem like there's been lots of excitement from people other than me about doing dev
 - Branden: Hitting end of the quarter. Excited about demo pushing some functionality in Tock forward. Happy to work on it starting next week.
 - Branden: I am worried the audience might be very small. But I think that's likely okay. Still going to be useful even with a small audience.
 - Pat: Similar. Going to be a couple weeks, but interested.
 - Branden: One way this could work is for Brad to come up with a task list and put it out there, and then the rest of us can tag on
 - Brad: I am a little worried about "last-minute code", since we have a much higher expectation of code reliability these days compared to like five years ago
 - Pat: Maybe we should have an earlier deadline for beta testing. We can do a real, "virtual tutorial" with some of our students as attendees. Force use to have the whole thing written in advance and work through any bugs / technical glitches
 - Brad: Seems reasonable
 - Branden: I'm a little concerned that Amit or Alex might have stronger concerns about exactly what the tutorial covers
 - Brad: I think the userspace side is the biggest open question there. The kernel side we've got a few ideas put together
 - Brad: I do want the security key demo, but it's hard for me to see what's reasonable and real versus what's fake and doesn't have the same level of interest
 - Brad: TLDR: if you have ideas about what to highlight from userspace, that would be helpful
 - Pat: One thing we talked about before that's hard to do in a tutorial is the advantage of hardware abstractions. For a tutorial that would require multiple hardware for multiple people. Maybe a few extra boards at the front of the room. I think it's pretty slick if you can just have the same app run on multiple boards
 - Brad: Hmm. I wonder if a software environment would work for that?
 - Pat: There's that LiteX thing, but I don't know capabilities/limitations there

