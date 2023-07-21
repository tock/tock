# Tock Core Notes 2023-04-14

Attendees:
 - Branden Ghena
 - Johnathan Van Why
 - Hudson Ayers
 - Leon Schuermann
 - Alyssa Haroldsen
 - Pat Pannuto
 - Alexandru Radovici


## Updates
 * Pat: Master student here (Tyler) who is working on a Thread network stack for Tock and will be around

## Refactor capabilities
 * https://github.com/tock/tock/pull/3409
 * Hudson: The PR looks huge and changes hundreds of lines of code, but it is really just a simple change to capabilities
 * Hudson: Instead of traits, they're stucts with a private field and are unsafe to create. Generic over type T where type T is one of several specific capability structures. There is a high-level struct, which it could be parameterized by. These are all still zero-sized.
 * Hudson: The improvement is that you don't need traits anymore, so no vtables ever (which wasn't a problem in practice anyways) and the generics syntax everywhere improves.
 * Hudson: I was in favor of this, because not using trait objects and generics simplifies things.
 * Hudson: The overhead hasn't really changed though, and it's a lot of churn, which has concerned Phil. So anyone using capabilities externally or capsules which have capabilities would have to follow the changes and make them themselves
 * Leon: I think this is slightly inelegant in using the particular module structure to stop things from creating structs. Inside a module you could create the struct without calling its constructor.
 * Hudson: The module is just capabilities.rs right?
 * Leon: yes, but it requires that module structure
 * Hudson: That doesn't seem like an uncommon pattern to me. Maybe not in Tock, but in other places in Rust.
 * Hudson: We could create an explicit module inside the file.
 * Pat: We do have some capabilities like network stuff that are in other files and other locations
 * Leon: I think with this approach we could put in any generic type T, which could be an empty enum
 * Hudson: But just having a capability that wraps a random type doesn't satisfy a requirement for a specific capability
 * Alyssa: Is it important to have a trait which shows whether a capability is _actually_ a capability? I think no
 * Leon: I guess I'm saying that we don't have creeping of module requirements in other capabilities right now. Right now we only define the capability wrapper once
 * Alyssa: About the implementation, why pass a borrow of a capability and not just make the capability copy?
 * Hudson: We discussed this a little in the PR. If it's copy, then one capsule receiving it could make copies and hand it out. Obviously there would have to be an API for that. But it feels weird. In practice, I don't know how much of a concern it is.
 * Alyssa: Or we could make it clone
 * Hudson: Well that's safe to do anywhere
 * Alyssa: It's also safe to copy a reference to the capability as implemented now
 * Hudson: Because creating a capability requires unsafe, at least that has to come from main.rs. Where a capsule shouldn't have the ability to make more
 * Leon: An immutable reference could be limited to a lifetime, which could be nice
 * Alyssa: We could have capabilities wrap a lifetime too. It could still be a zero-size type and could not have copy implemented
 * Leon: That could be nice
 * Hudson: Rust isn't going to make an optimization to remove a parameter that isn't used? If it's inlined it will remove it otherwise no?
 * Alyssa: I think so. C wouldn't be able to
 * Leon: If we store capabilities somewhere, that won't be optimized away.
 * Hudson: They should be stored as an owned type
 * Alyssa: I think making it copy, or clone if we're concerned, would make the most sense. Be a fully zero-sized type
 * Hudson: We did have that conversation. The author changed it to be copy and said there was no size change
 * Alyssa: I think it makes more sense too. What does it mean to take a reference to a capability
 * Hudson: I'm still thinking through the ability to copy and share capabilities. You could copy the reference, but those are at least limited to the scope of the function call
 * Alyssa: I could see the lifetime being useful. Could do that just as easily by putting a lifetime in capability
 * Hudson: But then everywhere that accepts them would have to be parameterized by the lifetime
 * Leon: They could be static for now
 * Hudson: Then everything does have to live forever
 * Alyssa: It could declare it as static with new
 * Hudson: Then we lose the ability to restrict the lifetime, right?
 * Alyssa: We'd be leaving the option open for the future.
 * Hudson: I see that as less compelling than the PR implementation which leaves it open now
 * Alyssa: Oh, that doesn't match the PR overview anymore
 * Leon: So we create capabilities and pass them to something that holds on to them for a long time, likely the duration of the kernel. Or we pass them into functions that are short-lived. It seems like not much of a burden to make those functions generic over a lifetime
 * Alyssa: Adding lifetimes to places is toil
 * Hudson: I don't like it. It's not a huge inconvenience, but definitely additional churn
 * Alyssa: Considering how often we depend on things being static, does it even matter to have capabilities not be static?
 * Hudson: References right now have a shorter lifetime
 * Alyssa: Yes, but all of the static dependencies exist right now. So a capability is really just another static dependency
 * Hudson: With the current implementation, if a capsule duplicates a reference it couldn't hand it to something that could store it, since it has a limited lifetime. If it could be duplicated and held onto, then that could be used later
 * Alyssa: I think it would still make sense to have a lifetime in the capability, then we could control things
 * Hudson: But then it would always be static
 * Alyssa: New could have a different lifetime
 * Hudson: But it's got to be created in main.rs, and it would have to be static. Then the capsule couldn't further limit it
 * Alyssa: You'd need a reborrow ability. Subgrant or something
 * Hudson: Would you actually think to do that?
 * Alyssa: I still think capabilities should be copy though
 * Hudson: Is that because it's cleaner?
 * Alyssa: Yes, primarily
 * Hudson: I think if we go the copy route, then we do and decide that it's safe. I think copy with an internal lifetime is likely little benefit for the cost
 * Alyssa: Adding lifetimes is never fun
 * Hudson: And people are always going to instantiate in main, and likely won't bother reborrowing
 * Alyssa: You only need to do it if you have a wrapper type that can't be copy
 * Hudson: But you'll copy something that contains a phantomdata over a static reference, the copy will still have a static lifetime
 * Alyssa: If it had the capability with a lifetime, it could still shorten the lifetime
 * Hudson: I just don't think it would happen in practice
 * Alyssa: We could do a covariant lifetime. So if we wanted to say the capability is only valid for 'a we could do that. But we could have most things construct or accept static
 * Alyssa: I don't see obvious solutions here
 * Hudson: I'd prefer copy everywhere and not have lifetimes at all. I'm not sure limiting lifetimes is actually a better security thing
 * Branden: You did trust the capsule once. So you don't really have to worry about it passing the capability onwards
 * Alyssa: I mostly agree. I think implicit copy no lifetime is probably best
 * Hudson: I think it's marginally better than what we have now
 * Alyssa: It removes a lifetime everywhere
 * Hudson: They're almost always inferred, so not much clutter
 * Alyssa: So why not put the lifetime in the capability?
 * Hudson: Then it couldn't be inferred, right?
 * Alyssa: No, I think it could
 * Hudson: If a capability is generic over some lifetime, then you have to specify that lifetime everywhere right?
 * Alyssa: I think you can just depend on the lifetime elision rules?
 * Hudson: You can elide lifetimes that are generic parameters of a struct
 * Alyssa: It's not recommended but you can. '_ is how you don't elide it. But you can fully elide it in stable Rust
 * Hudson: Even if the struct has other parameters?
 * Alyssa: You can do it with refcell right now
 * Hudson: So summary, Alyssa is in favor of changing back to copy for the theoretical ability to restrict the lifetime. I am a little reluctant to do that, but maybe I just didn't understand the mechanism and its costs in adding lifetime annotations
 * Alyssa: Any function signature that uses it right now has a lifetime, so I think it's no change
 * Branden: So it would really make sense to see all three options: prior version of PR with copy, new version without copy, or existing Tock code. We should really see all three and figure out which is best
 * Pat: If it's just ergonomics, then maybe this goes on a big list for Tock 3.0
 * Alex: I agree with this
 * Hudson: Phil is right that this will break everything that is out-of-tree and require updates. The code is easier to read as a result, but there's ergonomic upsides and downsides, so maybe this is a wash until the next time there is a major breaking change like Tock 3.0
 * Alyssa: I do think the capability model was very confusing right now and unexpected. So I would like a change for a future version of Tock. I'm fine putting this off

## Tockworld
 * Email from Amit: likely east coast, likely end of July
 * Johnathan: I might have a hard not attend at end of July

## Tockloader Rust Port
 * https://github.com/tock/tockloader/issues/98
 * Alex: We started re-writing Tockloader in Rust, which makes Tockloader be more consistent and will hopefully help installations
 * Alex: My student started working on it, and wanted to open a branch on Tockloader to get feedback from the whole team, not just me
 * Branden: Does it make more sense to have a branch or a separate repo under the Tock organization?
 * Hudson: I'll make a repo


## Maybe-uninit and Syscalls
 * Alyssa: Generally, I want to be able to handle maybe-uninit better
 * Alyssa: We talked about passing uninitialized data across the syscall boundary and maybe-uninit data. The issue I have is that I want to make sure it makes sense that passing uninitialized data across a syscall boundary, the bits are frozen and the compiler should consider them initialized.
 * Alyssa: Doing this in the kernel is also harder, because there's no write-only process slice. If I added one of those, mirroring the read-only and read/write, would that be interesting?
 * Leon: The C userspace creates memory by just doing it, so I don't think the kernel can really make any assumptions about whether userspace considers memory initialized or not
 * Johnathan: OpenTitan when the think boots, reading from memory will fault. I think one of the bootloaders zeros everything to make ECC checks pass, but we should check that
 * Leon: I think there's a violation if the kernel would read process memory which is 1) not stable or 2) would cause a crash. If that exists, it's the kernel's responsibility to resolve it. Must be initialized before giving it to an app. Or some guarantees from the architecture.
 * Johnathan: Agree with Leon
 * Leon: So I don't think we need typing for that. In general, we do memory setup in assembly before even letting any Rust code run. Stuff needs to happen before Rust starts up
 * Alyssa: Why? I'm not sure I follow
 * Alex: I can share an experience with ECC memory. Unless you zero it out fully, it will start randomly faulting. So I don't think you could have some RAM that's not fully initialized. It could randomly fault later
 * Johnathan: So the kernel has to initialize it before giving the memory to an app
 * Leon: There's no way for the kernel to trust whether userspace has initialized things like it promised. And it would be very hard to track these things
 * Alyssa: Okay, we can discuss more next week


