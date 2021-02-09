# Tock Core Notes 2021-01-22

## Attending
 * Pat Pannuto
 * Amit Levy
 * Alistair
 * Leon Schuermann
 * Arjun Deopujari
 * Philip Levis
 * Johnathan Van Way
 * Hudson Ayers
 * Brad Campbell
 * Branden Ghena
 * Gabe Marcano
 * Vadim Sukhomlinov

## Updates 
 - phil: 33/43 syscalls; yield done; 10 more drivers and exit is that last of the todo list -- I'm starting to pick up some of the last lingering ones to get it out the door
 - brad: following up on last week's 'initial RAM' discussion: it's really a historical artifact that the kernel tracks the stack pointer at all; but that's more an arch-specific thing; I have a WIP branch that pulls it into arch-specific code rather than core process.rs ;; It's a simpler conceptual thing, but it ends up touching a lot of code; PR needs testing
  - pat: Introducing Gabe Marcano, PhD student @ UCSD, will be working on a benchmarking project
  
## Testing of syscalls only supported on libtock-rs
 - phil: Might have a quick answer, as there's a PR from Johnathan
 - johnathan: Sadly no, that's on the re-write side of the line
 - phil: The challenege here is that while Alistair has been doing a great job of porting things, we have devices with no libtock-c support but libtock-rs doesn't yet have 2.0 support, can (1) implement in libtock-c, (2) get 2.0 working in libtock-rs, (3) cut the drivers
 - amit: 4th option? delay testing
 - phil: that feels like a non-option; compiling but untested code in main is not great
 - alistair: are we planning to release 2.0 prior to userspace support in libtock-rs?
 - amit: maybe a question for johnathan? though it would be unfortunate to block one of the libtock-rs 2.0 vs kernel 2.0
 - johnathan: it's kind of a race between me and the rest of the core team for the respective 2.0's
 - johnathan: on the one hand, don't want to block kernel.. on the other, do have strong work incentive to have libtock-rs 2.0 ready on ~1month timeline
 - amit: is it reasonable / viable to get some of the person-power from the core team moving from kernel to libtock-rs, now that kernel is close? caveat: you (johnathan) would have to project manage that a bit
 - johnathan: maybe.. there's a lot of interrelated things that a WIP that I need to get out. Need to get the driver infra written, but once it's about porting drivers, that should be well-suited
 - amit: what are the drivers that are unique to libtock-rs?
 - phil: think they are HMAC and CTAP (alistair confirms)
 - leon: I think I can write a libtock-c driver for HMAC [alistair also willing; it's the simpler one]
 - amit: for CTAP, how much deep USB knowledge does one need for a C driver?
 - alistair: in libtock-rs, we use the CTAP crate... fair bit of work
 - phil: but all we really have to do is test the system calls.. right?
 - alistair: yes, but won't get meaningful subscribes without sending the right data/events -- really need it all
 - phil: in this particular case, the thing we are trying to test isn't the full CTAP functoinality, but the implementation of the system calls...
 - alistair: yes, but if you don't send valid data, you won't get valid data
 - amit: very difficult to write a test that covers syscalls without implement much of the stack
 - phil: can we hardcode one set of messages?
 - pat: can we complie CTAP crate as an object to link with C?
 - amit: no, reloaction strategy difference between llvm and gcc
 - vadim: also problems with static linking of libraries and mixed toolchains (missed some of the details here) -- for one specific use case, you can make it work
 - johnathan: maybe a miscommunication here between vadim and amit here -- llvm folks say it's not possible
 - amit: I might buy that with static libraries it's possible in some but not all cases
 - johnathan: it should be doable with statically linked binaries with libtock-c's risc-v support?
 - amit: don't really have a risc-v platform with USB, except maybe Open Titan?
 - alistair: ...kind of; OT USB is broken, would have to revert a bunch
 - amit: summary: CTAP in C userland is pretty high effort; some possible shortcuts available, but don't want to rely on them to get 2.0 out the door
 - amit: options: tie to libtock-rs, or be okay with this one driver being untested, or sequester this driver
 - hudson: 4th proposal: my understanding is that tock-2.0-dev is not mainline, it's a feature branch; can commit untested code there. Right now there's still a possibility that libtock-rs-2.0 beats kernel 2.0. Proposal: we should just merge this untested so that the purging of old driver infra can occur in that branch. We add this to a list of things that need to be resolved on 'actual 2.0'
 - leon: I agree with the state of 2.0 branch; many of the changes there only tested in isolation -- I had been imagining a 'big test' event near 2.0
 - phil: sounds good/okay to kick the can for now
 - phil: opens the Q about what happens when 2.0 is ready, but we have capsule code that's untested/untestable -- what should our policy/plan be?
 - leon: leave it out until it's tested?
 - phil: it's okay to have not-well-tested code in the repository, just needs to be clear (i.e. a capsules-dev/ folder?)
 - leon: brings back an old Q about 'support levels for capsules'; the dev folder is a simple form of that
 - amit: indeed; though maybe we can keep punting on the big question if it's just 1-2 capsules that will only be in the -dev state for small N months
 - phil: I worry about the signpost capsules in the same way -- what happens where there's no signpost hardware?
 - phil: don't want to give impression that signpost code is as tested as console
 - phil: not a problem for today, but we need to keep in mind
 - phil: conclusion: okay to merge to 2.0 branch for now; will re-assess when we merge 2.0 back to mainline
 - amit: conclusion: will need a clear label for untested, will not throw away
 - leon: really should apply to all capsules that aren't tested on the merged result [even things tested along the way to 2.0]
 - alistair: can try to write some simple, partial CTAP tests to help this out along the way
 - alistair: if anyone has imix hardware, can they test USB?
 - phil: I will try.
 - phil: What USB is exposed on imix?
 - amit: There's the little unix utility that Daniel wrote to interface with imix as a (raw bulk?) device
 - phil: you get that over target?
 - amit: yes, need to plug in both cables
 - amit: tool is in tools/usb -- bulk echo, bulk echo test, etc
 - brad: and need to flash a userspace app
 - phil: yeesh... okay, but I should figure this out; on it
 - brad: ideally it should just be flash the userspace app and run the tools, but, YMMV
 - amit: I'm happy to help...
 - phil: focus on IPC :)
 - hudson: I've also done USB testing, can help

## https://github.com/tock/tock/issues/2320
 - phil: useful to check in on?
 - leon: proposal didn't get much backlash? seems last discussion deemed reasonable?
 - amit: that's roughly my takeaway as well
 - phil: okay.

## Flash / Flash HIL
 - brad: many periodic issues with the Flash HIL; been a lingering issue for ~years now
 - brad: one group really wants to see it change / re-write; another camp of "it works, why change"
 - brad: would be good a plan/vision of some kind here
 - amit: let's put this on the agenda for next week when Alistair is back to join
 - amit: might just be a matter of consensus building; maybe different interfaces for on-chip/off-chip, etc
 - phil: yeah, things were designed for one use case, but other use cases have arrived
 - phil: thought there was actually a good discussion

## Exit [terminate vs restart]
 - phil: in the syscall discussions, exit semantics came up
 - phil: kernel can do whatever it wants w.r.t. restarting or terminating you...
 - phil: so are these syscalls hints or directives -- what promises does the kernel make to app?
 - brad: good point; one idea might be to document not as "these are the two versions of exit", rather "you get so many bits of exit-with-value, and the way we've programmed the kernel is that it takes than number under advisement; 0 means app wanted to exit so there's no benefit to restarting the app; non-zero means something went awry, maybe should try restarting"
 - phil: where does this number go?
 - brad: two places -> 1) the printout when a board crashes on exit, 2) debug information for the process (via process console)
 - phil: this conflates two things: something wrong happened with, please restart me
 - amit: mental model is maybe systemd-like, some services should keep running (i.e. webserver that restart) while others are one-off (cron-like things?)
 - phil: Agree with the idea of there are applications that do and don't want to restart. One example was a migration app (e.g. a db update "app")
 - phil: The concern is really the conflation of return codes
 - amit: we have a bunch of space for these things -- why not both? have register space for exit reason and restart policy
 - phil: absolutely, we have the space
 - phil: didn't see a reason before, but these are good points
 - brad: two values seems reasonable to me
 - brad: but agree, can't make promises, this is request semantics
 - phil: right, that made me nervous at first, but that's really what most OSes provide.. so here we are

Fin.
