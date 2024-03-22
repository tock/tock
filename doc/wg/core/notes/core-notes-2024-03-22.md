# Tock Meeting Notes 03/22/2024

## Attendees
- Branden Ghena
- Hudson Ayers
- Leon Schuermann
- Amit Levy
- Tyler Potyondy
- Johnathan Van Why
- Brad Campbell
- Alyssa Haroldsen
- Pat Pannuto


## Updates
### Failing CI on recent PRs
* Amit: If you have any outstanding PRs that failed to build on the CI even though they should have, it was an issue with static mut on the new stable rust. It's been patched temporarily, so the PRs will pass, but you have to rebase on master and re-push for CI to pass. I tried to do that for other people in most cases.
### Dialpad Logistics
* Hudson: Last week's meeting wasn't recorded automatically. I turned back on automatic recording. There's also an option to share recordings with additional dialpad accounts.
* Amit: Unfortunately, accounts cost money monthly per seat. So it's harder to do.

## Removing MacOS CI Builder
* Amit: Rationale is that it takes forever. At least twice as long as everything else. And we run everything twice: once for initial PR and once for merging. If the MacOS builder regularly catches important MacOS problems, it's worth keeping. But I think it doesn't. Not any Tock OS code bugs or mis-compilations that wouldn't be caught on other builders. It's more focused on the build process for MacOS as a dev environment. Overall, I think if it's not essential it would be quality-of-life improvement to remove it
* Johnathan: I vaguely remember it catching a build script issue before. We could make an issue where we track whenever it breaks on MacOS in the future, so we could re-enable later if it was a bad call.
* Hudson: Is the main pain the slowness of the merge queue? And would that be resolved if we only ran the MacOS CI on the initial PR and not the merge queue?
* Amit: The merge queue is more painful. We can always hit the "merge when ready" button.
* Leon: Not true. It's marked as a required check, so we can't hit the button until it's ready.
* Amit: Another option, does running it nightly (or something) on Master. Would that be sufficient?
* Leon: We could have a github action job for that
* Hudson: The question is whether we'd notice if it broke
* Leon: The libtock-c mirror check opens a github issue if a build fails. So that would work
* Hudson: That seems great
* Brad: I'm very for this. I wanted the MacOS check gone for a while
* Pat: I usually advocate for keeping MacOS around. Why is the merge queue being slow an issue? It's hit it and forget it. So who cares?
* Leon: The merge queue just goes through PRs one at a time and is pretty slow
* Amit: And merges can cause conflicts
* Brad: And often we merge a bunch of PRs at once when people have time. And you have to babysit it to fix things
* Pat: Okay, this isn't too strong of an issue for me
* Branden: I'm also for this. If we get issues that MacOS CI is broken, we'll fix it pretty quickly in practice.
* Amit: MacOS CI does break rarely. If it's every few years, we're fine. If we find the frequency is often, then we'll revert this.
* Pat: I do think of CI as our statement of what first-class environments are. And not including MacOS does make it feel less tier-one. But I do see pragmatism winning out.
* Amit: Yeah. I do see that. The issue is just that the github CI runners are stupid slow
* Pat: Maybe we could buy a Mac mini and put up the runner ourselves?
* Amit: Interesting idea
* Leon: There's also a security issue. You have to do a LOT of work to run stuff in a VM. Someone could try to take over the machine. We also want a clean, reproducible build environment.
* Pat: I think we'd just still only run it once it goes into the merge queue
* Johnathan: But someone could open a PR that changes it to run immediately right?
* Amit: There is a limit that first-time contributor checks don't run
* Leon: You also can't change the file for how it runs in a PR. It respects what's in master
* Brad: I think realistically we aren't going to have our own Mac mini runner.
* Hudson: Seems like more effort than fixing the very few MacOS bugs per year
* Amit: Okay, I'll take on the task of handling this

## Moving modules out of the kernel
* https://github.com/tock/tock/issues/3845
* Brad: In the kernel we have traits and implementations of those traits. But the implementations don't need internal kernel stuff and could live anywhere. That's the point, that you could make your own implementations, and ours are just for convenience.
* Brad: So, is it worth moving these implementations somewhere else?
* Amit: Yes
* Branden: What's an example?
* Brad: Implementation of what happens when a process faults. Another example is application ID assignment, we have some default ways.
* Hudson: I like the idea of them not being in the kernel crate, but I don't like them in capsule crate, as they aren't capsules?
* Amit: How are they not?
* Hudson: I feel like this would further confuse what it means to "be" a capsule
* Amit: These don't require unsafe, right? (Yes)
* Amit: Organizationally, there are capsules_core, and capsules_extra. So these don't fit in those, but there could be a separate crate either there or somewhere else without capsules in the name. I guess it makes sense that people wouldn't look for these where the sensor drivers are.
* Hudson: These are indeed not sensor drivers. I think it's reasonable to limit capsules to drivers of some type. I wonder where a good place would be
* Hudson: From https://github.com/tock/tock/tree/master/capsules capsules are drivers or virtualization or syscall interfaces
* Amit: We could have a new top-level crate for these
* Amit: Although, I'd define capsules as "untrusted kernel components". Not part of the trusted-compute base. But that's not super important to fight
* Hudson: Do the implementations touch internal kernel private fields?
* Brad: No, or it wouldn't work
* Amit: These are really things that are meant to be "pluggable".
* Amit: So is the only issue with this bikeshedding what their name is? (Yes)
* Brad: Okay, I'll make a first PR that moves these somewhere

## Signup for libtock-c updates
* https://github.com/tock/libtock-c/pull/370
* Brad: Two things. I'm looking for help here first. Just implementing the changes.
* Brad: Number two, I want to get rid of some old APIs. We have some custom system call interfaces for one-off chips from years ago. It really ties an application to a very specific interface, and every chip would need its own bespoke interface. This isn't something we use or want to promote
* Amit: I agree in concept. Can you give an example though?
* Brad: Things I put in the chips directory. Some of these are things that should be updated to the generic system call interface.
* Amit: Okay. I support this. This sounds like part of a libtock-c reboot, where part of that is removing unnecessary or stale old stuff.
* Amit: One question, should the removal be in the same PR or a different PR? My sense is that if we removed a bunch of these from mainline, we'd also remove the example applications for them. And probably no one would notice or care. So could that be a separate PR? Or should we keep it all together?
* Brad: I don't care
* Branden: There's no need to make extra work, but if it's easy to make it as a separate PR, that seems useful.
* Amit: Okay, let's do that.
* Brad: Back to the first issue of needing help.
* Amit: My suggestion, is it a lot of work to make a checklist of categories? (No) Then I can claim some of those categories. It's straightforward moving, right?
* Brad: It's a little more than that. Halfway to rewriting. New names and sometimes cleaning stuff up quite a bit. Some of the files are pretty non-standard, like the screen for example.
* Amit: Oh, it's the standardizing part that's work. I see
* Amit: And we shouldn't separate that from the moving of files
* Brad: No. It would be more work to change things twice. And it makes sense to make one big change
* Amit: Okay. Make the categories list of what's left, and I'll grab some and others can too
* Brad: One more question: Should we have a rule that the libtock library doesn't have any internal global state except for perhaps static functions.
* Amit: Meaning no global variables?
* Brad: You have interfaces that are public. Then internal functions that are private. But no private internal state
* Amit: Okay, so looking randomly at one of these, there's a statically allocated results struct.
* Brad: That's in libtock-sync. It MUST have some small amount of private internal state to synchronize things
* Leon: I think this is a good idea. Because it will eventually support having multiple instances of a system call driver.
* Amit: Okay, so looking at libtock-c in the PR, we do still have a static result, which could have been stack allocated instead or passed in. And now it's non-re-entrant and can't be reused with the current design.
* Amit: So, no private state seems like a fine rule. But we should keep our minds open to it when doing this PR and see if there's a good case for it we didn't anticipate
* Brad: I'm reasonably confident that there isn't one. And we rarely do it now for the async stuff. But there is some use of internal state that might even seem intuitive, so I think we just have to pick a consistency thing. I want everything predictable
* Amit: I agree. And we can back off if something causes us to question it
* Branden: I super agree that we don't want internal private state. And if you're documenting it, we should also decide what state we DO need for libtock-sync as we should be minimalistic about it

