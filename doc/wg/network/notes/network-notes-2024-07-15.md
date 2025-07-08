# Tock Network WG Meeting Notes

- **Date:** July 15, 2024
- **Participants:**
    - Branden Ghena
    - Leon Schuermann
    - Felix Mada
    - Ben Prevor
    - Alex Radovici
    - Ionut Cirstea
- **Agenda**
    1. Updates
    2. 802.15.4 Libtock-rs Driver
    3. Console Multiplexing
    4. PacketBuffer Upstreaming
- **References:**
    - [libtock-rs #551](https://github.com/tock/libtock-rs/pull/551)


## Updates
- None


## 15.4 Libtock-rs Driver
 * https://github.com/tock/libtock-rs/pull/551
 * Branden: overview of what's going on. 15.4 driver is being added to libtock-rs, which we're very excited about. It likely works right now, but Rust says it's unsound.
 * Branden: Big goal -- big research deployment in Poland, would like to use it, we need to help the author
 * Leon: We want to figure out how to handle buffer management so that they are shared with the kernel and returned to userspace in a sound way. They way this PR currently works with them breaks Rust's demands
 * Leon: The way this currently works is that it allocates these buffers from ephemeral memory. So they could theoretically be deallocated before unsharing them with the kernel. And the kernel would not be aware of that change. In that case, the kernel could overwrite application memory that's in use by a new memory object
 * Leon: The Core team call discussed this briefly last week. The possible solution was to place a requirement on the buffers to no longer have a Rust lifetime. Somewhere around here
https://github.com/tock/libtock-rs/pull/551/files#diff-cb5d770d7877fbe88b63f507acfb99e1e2fc63476b6d198a733b9311b6fc937bR260 instead of the `'buf` lifetime, they could be static. And that might be a sound design at the cost of taking away flexibility.
 * Branden: But an application doing 15.4 stuff isn't going to stop doing 15.4 stuff, so it seems fine that they're static
 * Leon: And this generally matches the Tock design, that buffers are preferred to be static so they don't fail at runtime
 * Leon: However, that's all information from the discussion on the Core team call plus a bit of skimming
 * Alex: We hit the exact same problem with the CAN implementation. We stopped because we didn't want to use Unsafe in a Rust application. We discussed this with Johnathan on a Core team call: https://github.com/tock/tock/blob/master/doc/wg/core/notes/core-notes-2023-03-03.md (which also included some Slack discussion which has now disappeared forever)
 * Leon: I see a crossroads here. We can try to understand this in real-time and maybe propose a solution. I'd argue that's not a great use of our time as we're not libtock-rs experts and haven't dug deeply into this. Or we can extract some more high-level plans: so I'd propose first seeing if using `static` everywhere fixes the issues. I'd be skittish developing a solution that's more fringe without Johnathan's approval, including using unsafe in a way that's not already common in libtock-rs
 * Alex: Problem is that if we start using unsafe, philosophically, what's the point of having this super complicated scope API that Johnathan made?
 * Leon: Unsafe for the system call bindings only, which need to be unsafe
 * Alex: If you want to swap the buffer though, you basically have to use unsafe. Regardless of whether it's static or not. There's no API that gives you back a buffer. So in libtock-rs you share it and when the scope is destroyed you get it back. But unless it goes out of scope, you can't get it back.
 * Leon: Ultimately, even for the static version, we'd have to dedicate an allow to be only usable on static buffers, and if you get a non-null buffer back, you can cast it to static and know that's sound. Right now there's no way to know that the buffer you get back is static
 * Alex: That was the debate with Johnathan, you summarized it well
 * Leon: Okay, so what we need to do first is 1) clearly communicate with the author about what's going on and 2) communicate the difficulty in shepherding this. That we appreciate this PR and do want to see it merged. But also that we're concerned about unsafe code in the kernel.
 * Leon: So, we could have them use it downstream for now. And it's going to be a long process to make it sound upstream
 * Branden: Paths forward:
   - Happy to draft a version of that message and run it by folks
   - Leon should leave a more technical comment that this is a libtock-rs problem, not a "this PR" problem
 * Leon: Less convinced about the fact that making it `'static` is going to solve our issues, but can do the latter for sure!
 * Alex: This was previously discussed on the Tock core call, https://github.com/tock/tock/blob/master/doc/wg/core/notes/core-notes-2023-03-03.md (which also included some Slack discussion which has now disappeared forever)


## Console Multiplexing
 * Alex: OxidOS is financing some interns working on tockloader-rs and adding support to it
 * Branden: there should be an initial PR or an RFC issue, just to make sure that the format of the "how you're packetizing console messages" actually makes sense
 * Branden: Amalia presented the packetization approach, and there were questions about the protocol. Making sure that we're not moving forward without early feedback.
 * Alex: For now, support basic tockloader-rs functionality. First concern -- how to distinguish between a kernel that does packetization, and one that doesn't.
 * Leon: Can you create an initial upstreaming PR as part of the handover? I'm happy to look over all of this if it would be helpful (SOSP rebuttal comes in next Monday and busy for a week)
 * Alex: Okay. This isn't coming until mid-August from us. Hopefully Amalia will be able to put in more effort here


## Tock Ethernet
 * Leon: I had originally though about putting PacketBuffer into the initial Ethernet implementation, but I think maybe we should just do a PR for the stuff first and then do the transition after
 * Leon: I think Ionut's Ethernet driver is in an excellent state and could go into Tock master soon. Ionut is welcome to make a PR against master
 * Leon: The idea was to make a common interface, but that interface should include PacketBuffer. So we could get stuff into master and then move to PacketBuffer after
 * Leon: We also pretty extensively reviewed this already, so it should be smooth to get into Master


## RP2040 WiFi
 * Alex: Porting embassy-rs driver for WiFi to RP2040 in kernelspace.
 * Branden: We talked about some of the more general issues with the RP2040. Brad said that we have a workaround for placing it into the bootloader mode with Tock's USB driver that works on the Nano 33 BLE?
 * Alex: The issue is that the bootloader is hardcoded, and I'm not sure if you can reboot it _into_ that bootloader. It's possible, but a thing that stops us is that the USB stack isn't working well on Windows and Linux
 * Branden: That's orthogonal though, right?
 * Alex: Sort of. But the board just doesn't work on modern Linux or all Windows. So it's hard to work on this at all
 * Alex: The USB stack jams after a few messages too. Could be a hardware issue?
 * Alex: There's also a crazy project that runs a debugger on one core and your app (so Tock) on the other
 * Leon: An issue here is that the USB stack was first implemented just for Nordic boards, and it's not a great interface. Then another group worked on the Pico, so there's quite possibly a mismatch between them
 * Alex: Ionut is pretty darn familiar with the USB stack and is going to take a look at the Pico
 * Ionut: I'd actually like to rework the entire USB stack from the ground because it's so hard to use
 * Branden: I don't think anyone's against it. It's just so much work


## PacketBuffer Upstreaming
 * Branden: Two concerns:
   - Where the buffer size requirement comes from? Bottom-up, or top-down?
     - For 15.4, PacketSize is specified all the way at the bottom, maximum packet size
     - Console, message size specified at the top
     Want to make sure we can handle both these cases.
   - "Generic proliferation."
     - "Great, to use it all these drivers get four more parameters!"
       - Usability pushback.
       - How can we alleviate these concerns to make sure others get value from PacketBuffer?
 * Leon: My thoughts on those.
   - Buffer size requirement is interesting. There's a size requirement from only one point, which is the bottom up. That's a minimum size requirement for headers and footers. There's not a maximum packet size that we can enforce bottom up (that would be MORE generic parameters). It's possible at runtime right now that an upper-layer 15.4 implementation tries to send a packet where the headers and footers overrun the maximum size parameter. And we can't guarantee that at compile-time right now.
   - For the message size from the Console. The lower bottom layer always requires a minimum reservation. The upper layer specifies what the actual size of the message is inside the buffer. So the lower layer imposes header/footer constraints, the upper layer passes message size information.
 * Branden: Not an explicit design decision, but it does feel weird that we guarantee header/footer size at compile time, but maximum size we don't guarantee?
 * Leon: Header/footer might be the trickier problem. Plus, the current implementation does still have that error of maximum size is too big
 * Leon: If we had a maximum size, we'd still have a runtime error, just at the userland interface instead of at the bottom of the stack
 * Leon: For generic proliferation: less of an answer and more of a thought. I'm hoping that we can use components to our advantage. Part of the reason this is particularly bad is that we are specifying some arbitrary numbers. Components could have constants by name instead, which would make more sense when reading it.
 * Branden: I'm actually okay with main.rs. I'm more concerned about proliferation within capsules
 * Branden: My bigger concern is that PacketBuffer is the "straw that broke the camel's back" because there are just so many more parameters
 * Leon: I agree that this is ugly. I think at some point we have to accept that Rust requires this for efficiency in Rust embedded.
 * Branden: Question -- if we step back and said "could we do PacketBuffer without the generic parameters and what would we lose"? Would it still be valuable without compile time guarantees?
 * Branden: I don't have an answer right now. I just think we need to be ready to address it when upstreaming PacketBuffer
 * Leon: Without generic parameters, we'd have a nice API but lose all the magic that makes it more reliable and robust

