# Tock Core Notes 03-26-2021

## Attending
 - Pat Pannuto
 - Amit Levy
 - Leon Schuermann
 - Branden Ghena
 - Johnathan Van Why
 - Brad Campbell
 - Hudson Ayers
 - Vadim Sukhomlinov

## Updates
 - johnathan: submitted a story about async in embedded, specifically on Futures, to the async foundations working group: https://github.com/rust-lang/wg-async-foundations/pull/85
   - amit: seems to be well-recieved?
   - johnathan: mostly, yeah; one person seems to be pushing back who was a designer, and may not get the story format concept, but seems to be moving forward overall
 - johnathan: libtock-rs is still moving, but still a little time until it'll be ready to merge

 - amit: Q, has anyone actually gotten SW-wolf (new board with the weird name) running? Leon?
 - leon: No, had similar issues, seems some moving around upstream, 
 - amit: Yeah, I got it running in non-nix, but the environment does seem fragile

 - 2.0 status?
   - brad: quiet, except for the merge of alpha-1, which is big news
   - brad/amit: mostly waiting on the callback swapping and syscall now
   - leon: callback swapping is probably the biggest blocker right now; if we can settle on the right design, we can move forward, but there's a huge amount of labor once we make a choice, so need to pick
   - hudson: the approach still has some rough edges, e.g. passing driver num, but personally I think I'm happy with it as is -- would be thrilled if there's anything better; Leon and I have spent a while, but can't find anything more optimal in the efficiency/usabiliy space
   - hudson: concur, mostly looking for approval on the decison at this point to move forward
   - leon: a few calls back we suggested that even if this isn't the most perfect approach, we can refine in the future, but what's critical is getting the semantics with userspace correct -- as rust evolves it can get better
   - amit: yeah, that's the main question; is this good enough right now, or if not are the semantics ones that we are willing to promise becuase we believe it will get better in the future, or should we revisit those?
   - amit: I'll concede I don't fully understand the semantics right now, which makes it harder to answer
   - brad: the code ends up being complex, but the concept is simple: callback comes from userspace, all kernel knows is process that passed it, the driver no that called, and the subdriver no; and since we say that driver + sycall no identifies syscall, you add process, and that's enough for the kernel to validate that whatever is passed to capsule matches. Basically, the key is being able to implement that check (at runtime)
   - brad: The complexity is that from userspace, the driver number is very easy to get; the other direction (capsule -> kernel), we don't currently track driver number; only have grants; kernel knows which capsule owns grant, but on the upcall path there's no way to know where it came from, and adding that tracking is the complexity
   - leon: yeah, the upcall does have this information normally, but the problem comes around from the Null callback and the Default trait
   - amit: okay, I think I get the intuition, but will need to dig more here
   - brad: This feels like one of those problems where there has to be a nice trick, but so far that trick has eluded us
   - amit: and we're worried about the check because of performance overhead, space?
   - brad: issue is that you need to know driver number, which means grant region must know driver number, so that Null upcalls are correctly associated with driver number. This means that the board (i.e. main.rs) must pass in the driver number when the grant is created. Now it's not just a number in the syscall table, it ends up duplicated around main.rs, components, etc
   - amit: it seems like an alternaitve might be a special case check when grants are being returned, where the empty/default callback can have None for driver numbers
   - brad: completely agree! doesn't work
   - leon: well, it works, it just fails some guarentees -- it ensures that userspace doesn't have access, but there are other issues -- Hudson has documented this well in a comment: https://github.com/tock/tock/pull/2462#issuecomment-796883929
   - brad: basically, the threat is when two capsules conspire to play memory-copy games; and it seems there's not good way around that

## Debug in libtock-rs
 - johnathan: if you use derive(debug), you birng in a minimum of 15kB of code; if you manually do it, you end up calling in the core library's derive(debug), and you're back to the 15kB problem -- my goal is 1kB apps
 - johnathan: in unit-testing: it's totally fine, in apps that emit debug messages, the bloat is huge
 - johnathan: only alternative I know if is Jorge's ufmt ( https://github.com/japaric/ufmt), more downloads than elf2tab, so seeing some use, but it's got some rough edges
   - proc-macro hack: because it's old mostly
   - undefined behavior: there are open bugs that are sitting, seems unmaintained (e.e.g https://github.com/japaric/ufmt/issues/30)
 - johnathan: options
   - take over maintenance
   - use as-is for now, see if it's useful
   - try to get rust-embedded to take it over now, so we can submit changes
 - amit: is there an option that looks like, "fork it; ping Jorge that says 'we forked for now to fix issues, always happy to return'"
 - johnathan: yeah, update the README for "this fork may not persist", we're exploring
 - amit: license is compatible √
 - amit: if Jorge's not interested in maintaining it, seems that existence of fixes can only be good
 - amit: If you think it would be easier to roll our own that's fine too
 - johnathan: not sure it'd be easier, but probably not a tremendous amount of work
 - branden: what about a format capsule? we already spend the 15k in the kernel, why not ask the kernel to do it
 - amit: that.... may be tricky
 - branden: not sure the how of this is feasible, but there is a format right there
 - leon: the tricky thing will be expressing how things are formatted
 - leon: there's also a push to do a lot of the format work at compile time
 - johnathan: yeah, custom struct printing seems to be a lot of the hard part in practice
 - brad: the only option that seems bad is trying to get Jorge to be engaged, since there seems plenty of evidence that's not happening -- so I'm for whatever is easiest and gets things going
 - amit: seems like 'forking it for now to test' seems like simplest
 - amit: this hopefully won't close the door on using a library that other embedded things will use
 - johnathan: for it into the tock org?
 - amit: that or libtock-rs
 - leon: if we want the possibility to hand it back off, a separate repo is probably better
 - amit: yeah; though you can filter things out of repos, but yeah, whatever's easiest
 - johnathan: easiest at the moment is probably vendoring into libtock-rs
 - [Fin.] √

## Mismatch between application and kernel (Tock version)
 - brad: right now, you probably will just get a very quick failure, but that's not so helpful
 - pat: can't we just not re-use any of the existing syscall numbers, we use very few?
 - brad: looking for a load-time not run-time solution
 - leon: there was a floated proposal about a magic struct or fixed place in the binary with a version
 - brad: right, the discussion from a while ago is that we have this KV store in the bootloader, but don't have one in the kernel -- had a hacky one for the kernel but didn't see a lot of use, so kinda dropped
 - brad: now, with multi-arch, unknown boards, kernel relocation, etc it becomes more useful again
 - amit: why not just store it in the bootloader KV store?
 - brad: not all boards have a bootloader
 - brad: tockloader now has a flag where you can set a flag in the bootloader, but this has forward compatability issues
 - leon: this goes beyond version number, also things like networking stacks holding MAC addresses, etc
 - brad: could that also go in the bootloader? that's where we keep device ID right now
 - leon: but that creates a dependency kernel -> bootloader
 - brad: different idea, have a capsule that could tell you 'what version of the kernel is this'
 - amit: such that tockloader would invoke that capsule? sounds cool..
 - brad: not sure it's so easy to do..
 - amit: certainly in the world where the application loader is an active part of the kernel, that becomes feasbile
 - brad: if we had a fully virtualized setup (i.e. channels over UART) that would help
 - amit/brad: but then we'll have lots of things that are still jtag based likely
 - amit: we might be able to get away with 'flash the kernel with jtag/swd, but apps are always over uart/usb/etc' and that can interact directly with the kernel
 - amit: in the case where we do want dynamically loadable applications, that's also more inline with the vision
 - amit: and we do have some code to do that since the bootloader is already a mini-kernel -- not quite that, but has all the pieces
 - brad: if the kernel isn't responsible for placement/policy, you're right
 - brad: much of the complexity comes from not overwiting an app if the upload fails, finding MPU-friendly spots, etc -- if this tool could also just follow commands, we're largely there
 - amit: if the placement policy is still up to tockloader, but validating that policy is up to the kernel, is that easier?
 - brad: yeah, that probably wouldn't be too bad
 - brad: anyway; that's a nice vision, but probably farther out -- in the short term, should we revisit the fixed data structure in the binary?
 - leon: I'd love to have it as an optional component boards can use
 - pat: some of our platforms (e.g. nrf's) have things like UICR designed for this use case, would be nice to be able to use them if we can
 - leon: yes, but also don't want to rely on it
 - amit: right, but that's part of the appeal of implementing this as a capsule, which can choose an underlying store and emulate with flash / etc
 - brad: need to avoid contention between binary blob store and board mechanism
 - leon: could be that we always need the data strcutre in the minimum case, which then defines which config storage mechanism is in place for everything else
 - brad: challenging as it's linker reliant, but worst case tockloader will always fall back to the 'this is a kernel without support'
 - leon: one was to fix the fixed offset problem is this trick where the first instruction is a jump forward, and the region after that jump is the config space -- fixes the unknown address issue
 - brad: consensus seems that we should try this? having a fixed config spot would be useful?
 - [Yes]

Ciao e'erbody.
