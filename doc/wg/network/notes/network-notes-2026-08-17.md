# Tock Network WG Meeting Notes

- **Date:** August 17, 2026
- **Participants:**
    - Branden Ghena
    - Tyler Potyondy
    - Marshall Clyburn
- **Agenda:**
    1. Updates
    2. Userspace Services
    3. IPC PR Status
    4. IPC Shared Memory Design
- **References:**
    - [Userspace Services PR](https://github.com/tock/tock/pull/4869)
    - [IPC Capsule PR](https://github.com/tock/tock/pull/5015)
    - [IPC Documentation PR](https://github.com/tock/tock/pull/5016)

## Updates
* Tyler: Had an undergrad who had fixes for Thread certification. Still need to follow up with him on that.

## Userspace Services
* https://github.com/tock/tock/pull/4869
* Marshall: Userspace Services are a way to allow userspace applications to provide some level of functionality that a capsule would normally provide. For example, the PR right now has crypto work that calculates a digest: hashes data from another application. This allows the work to be implemented in C and to be more swappable with alternatives in the future. For example, a fix could just update that one app, not the entire Tock kernel.
* Marshall: This work comes out of an idea from a while ago, with a more agile crypto system for Tock. For example, we could swap from AES128 to some new snazzy NIST algorithm that's more secure or computationally efficient. We wouldn't have to rely on Tock to provide a capsule for this, just update the userspace application. Makes code more modular and accessible to updates.
* Marshall: So, userspace service app provides a service which is consumed within the kernel. Possibly it's provided to other applications as well. Tock capsules can call into the userspace service via a HIL which looks identical to capsule implementations. This means that either other applications or even the kernel itself could be making requests of the userspace service.
* Branden: Really neat. So this isn't actually IPC, as nothing _knows_ it's talking to another process at all. However, there is communication and marshalling of arguments going on here, which overlaps interestingly with IPC designs.
* Tyler: The crypto working group is really excited about this. The ability for userspace services for crypto agility is really a key focus of the group: pushing things to userspace.
* Marshall: Yeah, network and security interest started this. Which fits both groups
* Tyler: What's the PR status?
* Marshall: PR is still open. Major work for PR is about getting documentation into the right places. There's some module documentation which is useful, but I knew we'd end up wanting more too. There are also some commits I've got that I haven't pushed yet.
* Marshall: We're also going to push some stuff to the Tock book and as a tutorial so people can understand and play around with this.
* Branden: Definitely interested in documentation. I want to use that to understand if there are limits or missing parts of the design and implementation.
* Marshall: I plan to push more updates soon
* Branden: Something on my mind is what this is useful for besides crypto. For example, thread would be an interesting use case because it build a lot of state on top of the hardware it interacts with. But, thread doesn't have a HIL right now so you'd need to invent one. And instead, you can just do an IPC interface with a Thread service.
* Marshall: Definitely something where you've got hardware and aren't just using the raw capabilities of the hardware. Something like sensor fusion. Maybe a machine learning system in userspace that's converting from raw data into some higher-level system.
* Branden: Machine learning is definitely neat. Works with a lot of data you don't want to transfer around everywhere. Also really wants to be implemented in C/C++ or some other language. Probably could also be implemented with IPC? That's not a downside, it's not like it has to be an either-or. An upside of the crypto use case is that being transparent to users is a big perk: then you could have a hardware implementation or a software implementation without changing things.
* Marshall: With userspace services, you're also provided a really rigid interface for interaction with the service.
* Branden: This is indeed unlike IPC, which just has arbitrary byte arrays. Presumably the client and server would agree on some format for those bytes, but the IPC system doesn't require it.
* Branden: Also something to think about is the overhead of all of this.
* Marshall: I did do some measurements of hashing speed of capsule vs userspace service. On 256 bytes of data. Took 2.8 ms for hash with capsule. Userspace service took 47 ms. So 10x cost.
* Branden: I don't have any measurements at all for IPC. That would be really helpful. I think having your documentation point to these test results as a back-of-the-envelope idea for what overhead will look like is really useful.

## IPC PR Status
* https://github.com/tock/tock/pull/5015 and https://github.com/tock/tock/pull/5016
* Branden: PRs are mostly sitting. Tyler approved the capsules.
* Branden: Documentation PR had a discussion between Johnathan, Brad, and I about how to support the "app store" model. It's not part of our initial design, but the goal was that it would be possible if you created a new IPC Registry for it. I should check in to see if they're satisfied with that conversation or not.
* Tyler: I haven't gotten to read the documentation PR yet

## IPC Shared Memory Design
* https://docs.google.com/presentation/d/10t5i1Glh1Ty86Eqo-VoCDSikgMftE_Y-NXnAaWRS2cM/edit?usp=sharing
* Tyler: On mutual exclusion, that's going to be tricky. Probably necessary for libtock-rs
* Branden: Implementation thoughts are that we could use MPU regions for this. We could make a new region in the originating app to block access and a new region in the receiving app to allow access.
* Tyler: MPU concerns are getting less challenging for new hardware. ARMv8 chips for example. The nRF52 series is increasingly aging, and there's an argument to be made to not optimize too much for that use case.
* Marshall: What do you think that means for existing platforms?
* Tyler: It's tricky to reason about the nRF52 with the ARMv7 MPU architecture. It's really not developed for people to write good software. It gets so much more ergonomic with the v8 architecture. Closer to the RISC-V PMP now. I don't think there's an implication for what it'll look like on the nRF52, but I do think it'll be easier for the v8 architecture, and the sizes and waste for alignment will be much better. Fragmentation will hurt the nRF52 quite a bit
* Tyler: My meta-point is to not focus on the existing constraints and apply them to everyone
* Branden: I'd really love to scale capability of IPC with capability of the MPU. So an app might ask to share a region and be denied because the size or alignment is wrong. Or denied because it's out of regions right now. And a lot of that will fall on the application itself to solve. This means an application may fail on an ARMv7 where the same code succeeds on an ARMv8 platform. My thought is that at the point at which you're doing complex shared memory applications, they're probably platform-specific. And simple shared memory should hopefully work on every platform.
* Tyler: Yeah. Where the MPU implementation is tricky is that it abstracts away a lot of nuance. Moving the app break for increasing heap size ideally doesn't fail. We want that to be a rare error. So the MPU does a lot of work to always succeed. If the interface becomes best-effort and pushing errors up to the app level, making it responsible for the requirements of its platform, then that seems reasonable.
* Tyler: Second part is that there will certainly be MPU bugs. Some parts of the MPU setup have been very checked, but parts of the MPU with IPC were not reviewed at all. So I suspect there will be lurking bugs there. The SOSP paper didn't look into IPC stuff.
* Branden: Yeah, unfortunately agreed. For example, there's a "remove region" functionality right now in the MPU, which seems to be totally unused in Tock. I expect it may or may not actually work.
* Tyler: How would this fit in the context of the current PR?
* Branden: Definitely separate. Categories of IPC capsules are: IPC Registry, IPC Relay, and this new "IPC Share" system. Possibly multiple capsules implementing parts of this.
* Marshall: For slice of app memory, if the original owner dies, then you have to make these zombie processes: maintain information to determine when it's safe to clean up. What's that look like for Tock?
* Branden: We really can't restart a process if it's memory is loaned out. Really a pain in the butt
* Branden: The state-tracking requirement is a great point. Really need to think about exactly what state would need to be added to processes for that.
* Marshall: What are the use cases for this?
* Branden: I think the first use case is probably some kind of dynamic application loading case. Where one app has a blob of memory which it wants to slice off to validate, and maybe even to load later. I should really think of more use cases though. Something with image processing maybe? Definitely want cases where there's a big chunk of memory such that we can't copy it, but actions upon that chunk of memory by multiple processes.

