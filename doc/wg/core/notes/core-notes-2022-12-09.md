# Tock Core Notes 2022-12-09

Attendees:
 - Branden Ghena
 - Pat Pannuto
 - Alexandru Radovici
 - Leon Schuermann
 - Brad Campbell
 - Phil Levis
 - Hudson Ayers
 - Johnathan Van Why
 - Chris Frantz
 - Alyssa Haroldsen
 - Vadim Sukhomlinov


# Updates
 * None


# 64-bit timers for userspace
 * https://github.com/tock/tock/pull/3343
 * Phil: Alistair noted that RISC-V has 64-bit timers and it would be good for userspace to be able to access them. So he made the PR.
 * Phil: I think there are some issues here, where different architectures get different-sized timers. But the better discussion is whether 64-bit time should be part of the system call API and a basic primitive.
 * Hudson: What's the alternative? Separate system calls? No support?
 * Phil: If we want 64-bit time to be accessible to userspace, I think the right way to do it is to add a new device. You could just add a command to the existing capsule, which would require processing properly.
 * Phil: We could instead just have 64-bit time to be the status quo.
 * Alyssa: Sounds expensive for some cases.
 * Phil: That's right. The cost is that for a 32-bit architecture, we'd need to properly handle overflow and a multi-word time value. You can do it and the code isn't big, but it's extremely sensitive and tricky to get right. The low 32-bits are free-running, while the upper 32-bits are updated by interrupts. You could end up having the two out-of-sync because you need two separate reads. Fussy. But it can be done.
 * Alyssa: I'm worried about the static code size impact.
 * Phil: I think that if done right the code size wouldn't be large at all. 100-200 bytes at most. It's very little code.
 * Phil: The current TRD105 https://github.com/tock/tock/blob/master/doc/reference/trd105-time.md#8-required-modules does say that you MUST provide 64-bit time in the kernel. It's not finalized, so we can still change our minds on it.
 * Hudson: I thought Ti50 tried to export 64-bit time to userspace somehow. I remember some issues around the update. If you already have some hack to expose it, that might be helpful.
 * Alyssa: I'd have to go look. Not sure.
 * Phil: For systems with a native 64-bit timer, there would be no code-size implications. Only for 32-bit wide timer limitations.
 * Alyssa: If I remember correctly, we have a 64-bit timer.
 * Phil: I think RISC-V mtimer is 64-bit.
 * Chris: For my part, I would like to see 64-bit time supported at _least_ as an option people could turn on if they want. For ms/us time, 32-bit timers can really hit a limit. And synthesizing something yourself isn't great. So the kernel doing "the right thing" for you would be really helpful.
 * Alyssa: Sounds reasonable to me, as long as the code size impact isn't huge (~1 kB).
 * Chris: I also like being able to turn it on only if you want it. So if you know you don't, then just don't include it. But 32-bit is too small. About one hour for microsecond tick. About 49 days for millisecond. Both are reachable.
 * Phil: The way the alarm/timer system currently works is that rollover is fine as long as you use unsigned operations/values. But because the increments are 32-bits, then you can't set something further in the future than half of that window. So half an hour and 25 days.
 * Chris: So having a facility for the 64-bit representation would be good.
 * Branden: That sounds to me like a separate capsule
 * Johnathan: I would argue that we should only have 64-bit time and not waste code size exposing 32-bit time, since you're just asking for wraparound bugs. That could still mean a separate capsule, and not offering the 32-bit capsule.
 * Hudson: My understanding is that the implementation would track overflow at the lowest level to track overflow.
 * Phil: Not necessarily. You could put a thin layer on top. That wouldn't be a big deal.
 * Hudson: I ask because maybe a lot of the complexity in the time capsules could go away if they only deal with 64-bit timers.
 * Phil: Unfortunately no. Most of the complexity isn't about overflow, but rather about when a request comes in what happens if the request was generated in the past and the time for it to fire has already happened. The ability to realize that time is passing while processing is happening, that's the vast majority of the complexity and edgecases.
 * Hudson: I thought the only reason we needed to pass in the current time was for overflow.
 * Phil: Well, all timers overflow
 * Johnathan: The whole point of a 64-bit timer is that it doesn't overflow. You have to start it at zero and you have to cap it at some reasonable frequency (like 10 Ghz) but those are reasonable for Tock to do. 10 GHz still gets you 50 years.
 * Alyssa: Ti50 isn't concerned about 64-bit timers overflowing.
 * Phil: Linux wasn't concerned either.
 * Alyssa: We do actively consider 32-bit timers. Part of what we do is have two timers: a higher precision likely to roll over and a lower precision that won't. It depends what we are trying to do.
 * Hudson: The reason I asked all of this is because I wanted to avoid two capsules. I thought it would be easier to implement all of this at the chip level, so we wouldn't need a 32-bit capsule. It also seems that if you have one timer that is 32-bit and one that is 64-bit, then you end up with apps targeting one or the other. Since lots of userspace libraries build on it (like UDP) you end up with non-portable apps based on which one is targeted.
 * Alyssa: I definitely don't want to split the world. You could theoretically have two with interchangable interfaces.
 * Hudson: Not if one has 32-bit values and one has 64-bit values. If only some boards would have the one interface, then we'd need to use that for all userspace libraries that want to be portable.
 * Phil: Yeah. It depends on what we think of as a "standard kernel"
 * Hudson: If the cost is only 100 bytes, then it's not worth the maintenance burden of having two capsules.
 * Alyssa: Agreed. I was thinking having a 32-bit capsule and a super-set of functionality for 64-bit capsules.
 * Phil: Right. When I said it was 100 bytes, it would the cost for a 32-bit platform to just let userspace get the time in 64-bits. It doesn't allow setting alarms in the context of 64-bits.
 * Phil: I think the outcome is that I should poke around and see what the code size implications should be. It sounds like there is a lot of interest in 64-bit access for userspace.


# Use case for new allow syscall
 * Phil: Idea from chats at Google we wanted to make people aware of.
 * Phil: Use case is some security-oriented microcontroller that can access an external RAM bank. It'll have to do operations on that DRAM, like compute a hash over a block of RAM that's not inside the process image. I've got a big block of say, 16 MB and the process wants to ask the kernel to compute a SHA over it. The memory isn't part of the process address space and outlives it. And we definitely don't want to copy it into a user space buffer. So the question is whether there is a new allow which could provide static buffers to the kernel, so it could pass them to DMA without a copy.
 * Branden: Why not treat the external memory as a device and pass block numbers to the driver? Then you wouldn't have to touch allow
 * Phil: At some point we need an address to pass into DMA. There could be a translation somewhere in the kernel. Some trusted code to change block number into a static address.
 * Branden: Yes. The problem definitely still exists either way, but it seemed shunting it into the kernel lets you avoid syscall changes, which is a plus.
 * Phil: We were thinking it would be neat if this didn't require a totally different mechanism. So the same SHA calls could work on either type of memory.
 * Branden: That makes sense and is desirable.
 * Hudson: What's tricky is that processes shouldn't pass memory from their reserved region as "static". Because they could be restarted. Would this memory be owned by the process that's passing it?
 * Phil: I do think there's a separate question of where these addresses come from and how the kernel can be sure the process isn't making something up. Let's pretend there is some way of doing that. Then how does this propagate as something like an allow on the system. Assume the checks already occurred and the memory is okay. How do we do it?
 * Phil: I wasn't looking for an answer yet. I just wanted to get people thinking about it.
 * Hudson: It does seem like it would be challenging for this to use the same SHA system call. The code right now assumes there's a shared buffer. I guess this could dovetail with the having a enum for buffer type and work on any.


# TBF Parsing Crate
 * https://github.com/tock/elf2tab/pull/62
 * Alex: One of my students sent in this PR. We want to port tockloader to Rust for two reasons. 1) installing with pip and sudo or not is a pain. Very frustrating for students. 2) We could also reuse code better if everything is in one language.
 * Alex: A question is where we put this. Some mono-repo? Or something just for tools?
 * Phil: I love this idea. When working on AppID, there were three separate places where TBFs were parsed. I understand why everything evovled this way, but it was a challenging aspect.
 * Alex: I would be happy to begin working on it. We have some students and bandwidth.
 * Hudson: Brad did start one once: https://github.com/tock/tockloader-proto-rs But it's different from what you were thinking. Brad would know more
 * Brad: So, this is the implementation of the bootloader communication protocol for the board. So tock bootloader uses this. It's another case where we have the board side in Rust and the controller/host side is in python for Tockloader. Another duplication problem.
 * Hudson: And it would be a lot easier to test these against themselves if it was all Rust.
 * Brad: Maybe. The background here is that there was a tool called stormloader that was written in python. And Tockloader grew from that. Tockloader is very complicated. It's 2-3 orders more complicated than elf2tab in my mind. So I'm torn here. I don't think installing python is that hard, and it lets us iterate pretty quickly. If there was a port in Rust, that would be good to build off of. But I'm not excited about two versions where each have partial functionality. Where like the Rust version supports _some_ things and the python supports _other_ things.
 * Chris: What is the scope of the python tockloader? Does it search out FTDI chips and stuff like that to do firmware loading?
 * Brad: Three interfaces: OpenOCD, Jlink Tools, and Serial ports. It tries to intelligently identify serial ports by name. It can autodetect boards of the two JTAG interfaces to figure out what is connected.
 * Chris: In the OpenTitan repo we have opentitantool which allows us to interact with it. It's in Rust with structopt/clap as the CLI. We have a limited set of hardware things to interact with. I found writing the code in Rust to be nice and convenient, but I think there's a big advantage of Python in how easy it is to interact with other parts of the system. Where Rust would have to reinvent some of that. For example, opentitantool can interact with 3-4 hardware interfaces now and has abstractions for them: SPI, GPIO, UARTs and some custom add-ons for FPGA boards for loading a bitstream. All I'm saying is that it will be a LOT of work to reinvent Tockloader in Rust. I found, at least for our tooling, that the low-level Rust tooling for interacting with FTDI chips, for example, were somewhat wonky. I really like that you can make a rich command hierarchy with structopt/clap and that Rust gives you a lot of power in representing abstractions, but it's a not insignificant amount of code in open titan for opentitantool, and it's just one chip we're talking too. That's my experience from building such a tool. I don't want to discourage you, but I want to be clear that it's a BUNCH of work and to first consider the value of the existing stuff. Could instead clean up nasty corners and clean up stuff in the existing implementation.
 * Alyssa: Rewriting in Rust means the same API for enums and things like that.
 * Chris: That is a good counterargument. A nice unified API if everything is in one language.
 * Alyssa: It lowers the bar to changing the TBF implementation too. So that would speed up evolution of TBF stuff. But python is great at generally evolving code.
 * Phil: I think you touched on the tension. Python is great if we want to evolve the Tockloader interfaces. But changes to the TBF format that's in multiple tools would be better all in Rust. So the question is where the pain-point is.
 * Chris: Another part of my experience is that opentitantool is a library of functions for interacting with the chip. So writing a purpose-built test program to interact with a chip, no matter what it is or its interface is, that's all abstracted away by the library. So you can have a rust-based test program on host that loads code onto a target and interacts with it over interfaces. Very convenient to have. This is one of the things that we get out of the tool being in Rust. You can write some really good custom test flows
 * Alex: We were thinking of building on top of probe.rs https://probe.rs/ Still need to look into this more.
 * Pat: I was looking at Tockloader. It's current 6000 source lines of python. There are a LOT of features that would need to be brought up. So taking something like this on is good. But I think you'd want feature parity before switching over. And that's gonna be a lot.
 * Alex: My roadmap would be end-of-summer. Just experimental until it hits parity. I wouldn't push for switching before then.
 * Pat: That seems low-risk from our perspective.
 * Phil: To be clear, an important thing to tell Alex is if we thought that we must be in python, even if the Rust version hit feature parity
 * Brad: It's a good question. I think there are definitely benefits to being in Rust. A question is how hard it is to implement this functionality. If it's really awkward and challenging and there aren't good interfaces, then maybe we need some compromise.
 * Alyssa: Is there a Tockloader test suite?
 * Brad: No
 * Alex: My final question is whether it's worth splitting out the crate or if we should wait on that.
 * Brad: Is there a reason not to include it in the TBF crate we already have?
 * Alex: We have a TBF parsing crate? Is there a reason elf2tab wasn't using it?
 * Brad: Well, elf2tab _creates_ TABs, but the kernel parses them. So there isn't as much overlap as you'd think.
 * Brad: Here is the crate: https://github.com/tock/tock/tree/master/libraries/tock-tbf
 * Brad: To answer your question, I see no reason to make a separate repo, since Rust is so good at splitting things into crates
 * Alex: Makes sense


# Update on Licensing PRs
 * CI PR - https://github.com/tock/tock/pull/3345
 * Hudson: Johnathan made a CI PR for licensing, but it's blocked on more discussion on the original PR
 * Original PR - https://github.com/tock/tock/pull/3318
 * Pat: I'll pick that up soon, sorry
 * Johnathan: Is there a reason we do the two lines (copyright and license) in the order we picked? I thought the other order was more common
 * Pat: I thought the order mattered for some reason. It was some implicit top-down thing. It felt arbitrary to me and I don't feel strongly about it.
 * Johnathan: The boilerplate for both MIT and Apache put the copyright first.
 * Pat: I'll read a few more things, and if it doesn't make sense to buck the trend, I'll make sure I follow it.

# Capsule reorganization
 * https://github.com/tock/tock/issues/3346
 * Hudson: Out of time, but I wanted to drop a link to the capsule split like we discussed last week. It's gonna be a huge pain to keep re-doing this. So I want to discuss and come to some conclusion before making more changes.
 * Hudson: I don't necessarily feel strongly about the split as is. Please drop comments on the PR if you have thoughts.

# Future meetings
 * Phil: We'll meet next week, December 16th but not on the 23rd or 30th of December due to holidays. Does that sound right?
 * General agreement

