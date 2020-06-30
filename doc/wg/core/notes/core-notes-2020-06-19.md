# Tock Core Notes 06/19/2020

Attending
 - Garret Kelly
 - Branden Ghena
  - Philip Levis
  - Brad Campbell
  - Leon Schuermann
  - Vadim Sukhomlinov
  - Jon Flatley
  - Alistair Francis
  - Amit Levy
  - Hudson Ayers
  - Samuel Jero
  - Johnathan Van Why

## Updates
 * Phil: Alarm/Timer re-designed implemented for SAM4L; mux/virtualizer ready, need to do interface; Amit + Guillaume jumping in on other boards. More details to come once WG has finished up.
  * Brad: Can now build RISC-V apps in libtock-c, *almost* by default (toolchain struggles), but it's in master
  * Brad: Arduino nano33 BLE board bringup coming along; serial over USB seems working/promising
  * Alistair: Bluetooth working on the Apollo3 board (interface is internal SPI bus), currently running their Bluetooth stack (C library from userspace; disabled MPU; direct reg writes); still buggy, but can see advertisements

## Inaugural Weekly Tock 2.0 Discussion
 * Phil: Scope/scale, deal with syscall ABI issues, but not major internal changes
 * Phil: Given external momentum, pressure, other users; prioritize getting this done and not leaving things in flux
 * Phil: In 2.0 scope: unallow/unsubscribe; exit syscall; yield-no-wait; read-only allow + maybe userland execution?
 * Phil: Authority is https://github.com/tock/tock/issues/1607
 * Phil: All the other big things (dyn objects, no static, etc) are important, but not 2.0
 
 ### What lives in the Tock repository?
 * Phil: We split out libtock-c, libtock-rs from mainline kernel, but maybe things like syscall bindings should be in-kernel
 
 ### How does the work get done?
 * Amit: There's a working branch on my fork; to be mainlined via mondo-PR or maybe small series of PRs
 * Amit: Consider this an invitation to folks interested in discussing this .. but also being on the hook to implementing this .. to join the effort (currently approx Phil/Amit/Jon F/Jonathan VW/Vadim?)
 * Phil: Primary effort here is to ensure that implementation and semantics match with the security model. Johnathan has been leading this well
 * Amit: This effort is about to take off in force
 * Brad: I have four comments
 * 1) This makes sense, motivated folks should drive it
 * 2) I have Q's about the proposed design, but not sure when we want to discuss
 * 3) Separating out libtock-c has paid many dividends, hesistant to go back
 * 4) Pragmatics: I think we could have a 2.0-alpha, 2.0-beta for a while; lets us merge in stuff, update external repos, without locking in 2.0; gives us a window to test/try
 * Amit: (4) sounds reasonable
 * Amit: (3) This isn't a wholesale return, but narrowly scoped to just the parts that are related to the system call ABI. Not certain this will make sense, just an idea right now.
 * Phil: (2) We will start with design; the key is that this is a focused, concerted effort taking off over the next few weeks. Anyone of course welcome to the design discussions.
 * Amit: (2) My thought is that as we plan design discussions, we'll have some announcement medium (email?) for the (phone call?) discussion
 * Brad: (2) Okay, that sounds different from what I heard earlier, which was more strictly implementation effort.
 * Phil: (2) As a concrete example, over the last week there was the beginning of a discussion around the allow issues. Don't want to let such design be too hung up on what's there.
 * Brad: One specific Q: I want to be part of the discussion on return values (1->2? why 2? why not more? etc)
 * Phil: At least 64 bits. Might be more? Trepidation comes from existing kernels, which tend to do 1 or 2 registers .. want to understand why before we do something different.
 * Brad: Sounds like this is still an open question; could be great opportunity to experiment
 * Phil: We should follow what others have done, unless we really understand the tradeoffs we are making
 * Amit: And this will necessarily be iterative; e.g. unallow/unsubscribe is subtle enough that questions will come up while implementing

## Host-side testing (Jon Flatley)
 * https://docs.google.com/presentation/d/1zEamuINkO_FRBRUTEErGYdXX8s-0_JxoetnDOZova3g
 * Jon: Intro: Work on OT at Google, working on host-side testing framework
 * <screen share of slides .. to be posted> [TODO replace with link]
 * Brad: This is generally very exciting; first fresh implementation of the Process type. Definitely questions on how Process has been done before, so this is some nice validation
 * Brad/Hudson: Upstreaming would be great, but need to think through how to balance the visibility needed here [Jon: yeah, it's a quick hack to work, not final design proposal] versus safety in deployed kernel (e.g. can't expose 'decrement_work' as pub)
 * Branden: Can you compare to Tock on QEMU?
 * Jon: Haven't done much with QEMU. This was motivated by OT, which needs something like this. OT doesn't have a good QEMU target, nor is it likely to have one (due to maintenance). Our planned path forward is more Verilator than QEMU.
 * Vadim: Other benefits for debugging; branch coverage; richer analyses, etc
 * Amit: This is kind of a third stab at this, and hopeful that it will pan out this time
 * Amit: Mocking out peripherals as QEMU plug-ins is a much heavier lift than mocking out to a rust function, python lib, etc
 * Phil: Yeah, that's the key insight. We saw this with the 8/16-bit platform. Cycle accurate simulators valuable for timing details, energy modeling, etc. However, when testing code, something more like TOSSIM (native exec) was way more used/useful. Unlikely to have one tool that does everything perfectly. Precision / speed tradeoff. MSPSIM could not simulate 100 nodes in a WSN...
 * Phil: Q: Semantics of subscribe callback / yeild -- how to do with unix domain sockets, or will this be on signals? How do these semantics align?
 * Jon: Signals are a good idea.. currently, yield blocks the process waiting on the socket
 * Amit: That seems simple and reasonable [Phil: yeah]
 * Phil: Maybe signals are not necessary here, as don't need interrupt semantic in process
 * [Editor's comment]: This is a good idea for the Tock 2.0 syscall discussion, to concrete describe our semantics relative to "better known" ones
 * Vadim: Yes, but maybe signals can help more accurately model how things execute in certain cases
 * Jon {+others}: Yeah, it's a question of scope; timing details, race conditions, etc not really in scope [phil: enter Verilator, etc]
 * Amit: Q: You are copying allow buffers on every application transition? [Jon: yes] That's fine from performance, but I'm wondering if the proposed new semantics for allow, where there's something closer to an explicit ownership transfer, will make this simpler [Jon: potentially]. Not answerable for now, but something for 2.0 WG to think about.
 * Leon: Given that unallow doesn't actually exist yet, opportunity here.
 * Jon: Yeah, this is most broad impl; no optimization, but correct.

## Scheduler Interface
 * [deferred to next week due to time]

## Closing thoughts
 * This may be interesting for some folks to comment on: https://github.com/lowRISC/opentitan/pull/2474, let's help make this doc as good as possible
