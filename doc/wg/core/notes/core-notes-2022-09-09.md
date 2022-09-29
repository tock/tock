# Tock Core Notes 2022-09-09

Attendees:
- Branden Ghena
- Brad Campbell
- Alistair
- Leon Schuermann
- Hudson Ayers
- Vadim Sukhomlinov
- Chris Frantz
- Phil Levis
- Pat Pannuto
- Johnathan Van Why
- Alexandru Radovici
- Alyssa Haroldson
- Amit Levy

## Updates

 * Phil: With 2.1 being out, I pull the AppID up to the current master. So it's synced up, but we have stuff to talk about today so it's not ready to go.
 * Hudson: I talked with Alyssa about deferred call series of PRs. It's a pretty tricky problem to find a solution that works well for both dynamic and normal deferred calls. Some creative use of unsafe could lead us to a solution, but still in progress.
 * Phil: This seems to recur in Tock, where we find a hard thing, talk to Rust experts for a while, and hopefully there's a solution.

## PR Review
 * We merged a TON of PRs that were waiting on the 2.1 release, and we have a lot to talk about today, so we're jumping past this.

## AppID - KernelResources Structure
 * https://github.com/tock/tock/pull/3124
 * Phil: A major question right now for App ID is in #3124.
 * Phil: Brief overview, when you create a kernel you pass in a verifier. It looks at credentials and decides if they are good or bad. Brad has taken the position, which I mostly agree with, that we don't want to pass arguments to the kernel. Instead, KernelResources specifies all kinds of stuff, and this should become a field in KernelResources. This has some nice side effects about lifetimes.
 * Phil: My response is that this could happen, but KernelResources is this organically grown grab-bag of stuff. So we should probably clean it up and decide what belongs and what doesn't. Example, if you look at the structure there's stuff related to hardware and security policies and it would be good to structure that better. Timers, watchdogs, filter, etc. We should probably decompose into multiple things.
 * https://github.com/tock/tock/blob/abfe6c3a6cb6c5f4e6ae074c2e3a667624a985e4/kernel/src/platform/platform.rs
 * Phil: So, that's the first discussion about AppID.
 * Hudson: What do you imagine this split would look like? Multiple traits, or organization within this trait, or hierarchy?
 * Phil: I think it makes sense to break into 2-3 traits. One for security, one for kernel configuration, and one for hardware. The second two _could_ be merged, not sure. But this is the structure for how you configure the kernel. And it now has 8 types in it. So it's time for more structure.
 * Hudson: The main reason for all these associated types is that it reduces the number of generic parameters we have to store.
 * Alyssa: We explicitly use this pattern on Ti50 in places. Reducing generic parameters by passing in a single type that holds multiple types. We sometimes use zero-sized types instead of holding the resources themselves as well. Helps maintaining things.
 * Phil: I think the technique is fine. But I still think we should decompose into classes of things.
 * Leon: I thought we wanted KernelResources to reduce generic types complexity. I always thought of it more as a technicality.
 * Alexandru: I thought Phil wanted to split it into traits, not add generics to it.
 * Leon: Right, but the primary utility wasn't usability for people who configure boards, but rather for making code in the kernel simpler by reducing types.
 * Alyssa: You could still have both with a hierarchy and subtraits. Personally, I'd want to see a specific use case for splitting things up before doing it. Not only for the breaking change, but I'm not sure we've reached the complexity where it's impossible to understand the entire thing.
 * Alexandru: I agree with Phil. The code would be better organized if split. KernelResources is large and handles a lot of things.
 * Phil: For example, I might have a kernel with particular security policies. But I might change which drivers it supports or compile it for different architectures. Right now, those two things are coupled. There's no way to split out the security stuff from the architecture stuff. We could continue this way for now, but it seems like it will be an issue going forward.
 * Hudson: It being a breaking change for out-of-tree boards is important. We do that all the time, but we _should_ minimize it. We could have a hierarchy, but it feels a bit like adding layers for layers sake. Is the concern that because there's so much someone will ignore something and not set it right?
 * Phil: From a least-privilege perspective, when I want to allocate a grant, there's a kernel resources for that. Which means that the allocate grant code _could_ touch the verifier or the scheduler.
 * Brad: That's not quite right. The kernel has access to all the configuration options, but you wouldn't pass a KernelResources to the grant call.
 * Phil: Well, there is this: https://github.com/tock/tock/blob/50550987a73dd596bf0384adc132d20bf722ea28/kernel/src/kernel.rs#L99
 * Brad: Sure, but that's still in the kernel. It wouldn't go deeper.
 * Alyssa: In those cases, we could just take one or two items by trait, passing in specific things instead of KernelResources.
 * Brad: One question is where does the split happen. Does the split need to be exposed to the board author, or internal to the kernel trait?
 * Alyssa: Where do you want to isolation to occur?
 * Phil: So, backing up, that this is a breaking change means we shouldn't do it lightly. And we definitely don't want to iterate a whole bunch of times. So I'd say we should table this for now, but keep it as a possibility for later.
 * Hudson: I agree. AppID doesn't need to block on this.
 * Brad: We should open an issue for this. (agreed)
 * Alyssa: We have multiple traits with overlaps like you're talking about. When we refer to an inner one, it builds a new implementation of internal stuff with a zero-sized type. That'll let anything that implements KernelResources implement, say, SecurityResources. So you can still pass in the raw thing, but lock down what the inner function gets to access.
 * Phil: That makes a lot of sense as an approach. I'll make an issue with some of this discussion and link in the PR.

## AppID - Role of Verifier
 * https://github.com/tock/tock/pull/2809#discussion_r937380785
 * Phil: Second major item is role of verifier and short IDs with security policies in the kernel.
 * Phil: Background: if you recall we have TBF headers specifying which system calls you can issue and which storage IDs you can access. These headers would be covered by integrity and authenticity in AppID. I believe Alistair has been using these successfully. So, in AppID you can take an app and make a short 32-bit ID and use it for security to determine which APIs can be accessed. So there's the notion that the kernel can impose security policies based on short IDs. This is different from a process/TBF-object stating what its security permissions are, potentially covered by integrity.
 * Phil: So, a signed TBF-object can specify things and kernel trusts whoever signed it. But with AppID, the kernel can also impose its own policies and not have to trust the TBF-object. So there's this interesting case of whether the policy is part of the TBF-object or part of the kernel.
 * Alistair: Missing one thing. If you're doing the enforcement through the TBF object, it's decided at compile time. But you can also add that enforcement to the kernel. If you have some key signing on an app, no matter what the TBF says, we won't give it the permission. So it's not one or the other, it's both. In case someone makes a mistake in the TBF object, for example. The checker would have a list of public keys that it allows on the board and what each key is allowed to do.
 * Johnathan: The short ID mechanism that Phil implemented is a compression version of that list. So we don't have to store everything and duplicate long AppIDs everywhere.
 * Phil: So that's right. The kernel could check, but it would have to compare the whole 4 kb key. You just want to do a word check at runtime.
 * Alistair: You'd just do it the first time.
 * Phil: Where do you store that state? Are you saying there's extra information attached to the process? That's what short IDs were for.
 * Alistair: There's a listing and you take the intersection to decide if it's good or not.
 * Phil: So in your model, it's required that the application specify in the TBF header all the things it can do. At loading the verifier also has a list of all the things that certain signed things could do. And you reject the app if they don't match?
 * Alistair: Yes. But many verifiers won't care and will just trust TBF headers.
 * Phil: So this really changes process loading. There's this additional check.
 * Alistair: Yes. But on each syscall, you don't have to check against the AppID. You still need syscall filtering on a little list for the process. But it's previously been verified so you never have to look up short IDs anymore.
 * Phil: Looking up the short ID isn't a big overhead. But every TBF object now, if you want a security policy on system calls, requires all processes to completely list their set of operations. You have to have syscall permissions headers.
 * Alistair: If the app doesn't list any, and the verifier has a smaller subset, the verifier could return that smaller subset and keep it in RAM somewhere.
 * Phil: But the verifier is just about checking, not imposing security policies on top.
 * Alistair: I'd argue that integrity is part of the header, but also whether it matches what the kernel expects it to be able to do
 * Phil: But that's a separate thing
 * Johnathan: Generally, you want to look at all the crypto stuff first, then do all the other verification separately. You don't want bugs from other verification to mix in with your crypto checking code leading to bugs.
 * Alistair: Fair. You could still do the whole crypto check first.
 * Johnathan: This just doesn't seem like the verifier's responsibility.
 * Phil: I agree with Johnathan. Verification is just integrity and authenticity. We could still have a separate mechanism for applying security policies based on headers of the app. It's just a separate question.
 * Alistair: It could be separate. Just adds a bigger state machine.
 * Alyssa: It can still use a lot of the same mechanisms though.
 * Alistair: I think so. It's fine if they're separate, but I think it should exist.
 * Phil: You're saying it's useful to be able to have a thing that decides whether to load processes based on TBF headers? (yes)
 * Branden: So it sounds like Alistair is asking for a separate Kernel feature, separate from AppID stuff.
 * Alistair: Yeah, I think a second place in the app-loading process that can check the headers.
 * Brad: I'm not seeing the benefit of combining them versus keeping them separate.
 * Alistair: Yeah, so two states before a process goes to runnable.
 * Phil: Currently a checker looks at a TBF object and checks if the credentials exist and are valid, meaning that it approves loading the process. So in the future, we could require a signature from one of N keys in order to load a process. It's just about authenticity and integrity. And you need that for all kinds of truncation attacks and things like that.
 * Phil: I think if we want to extend the method of loading processes to add features, we can. I'm a little wary in that, even doing this one thing has taken a year. I don't want to just throw in a bonus thing that looks at headers. It seems like a whole new thing to design.
 * Alistair: I'd say that just passing the headers into a function isn't a new design. We're already passing in the footers.
 * Brad: The part that's a new design is changing the intent of the implementation. So an implementer would have to choose which types of checking it wants to use and order and stuff. That all gets pushed to the board author. That's what would change the design.
 * Phil: And as Johnathan pointed out, you absolutely don't want to look at headers until you have verified their integrity.
 * Alyssa: What kind of "not look at it". Don't parse it because it could be malicious?
 * Johnathan: Headers aren't trusted, but in general yes. You really don't want to do any parsing at all if they could be invalid. This depends on the trust model.
 * Alistair: We are parsing the headers a little before going into the checker
 * Phil: Yeah, if you assume a potentially malicious header there's a big expansion in checks. The stuff we do look at first has a TON of checks in there now, to check size and alignment and what not. If you added each type of header, that could be a lot of code.
 * Alyssa: I'm not sure. What area of code are you looking at?
 * https://github.com/tock/tock/blob/eb3f7ce59434b7ac1b77ef1ab7dd2afad1a62ac5/kernel/src/process_standard.rs#L1236
 * Phil: In create of process standard. There are a lot of checks that occur. We might have to discuss more offline.
 * Alyssa: There are a lot of checks, but if you're dealing with data you need resiliency. Regardless of whether it's signed and verified.
 * Johnathan: Agreed, but it's a defense in depth approach.
 * Brad: I agree and wish we had better support from Rust to do that. Rust buffer parsing code is a lot like writing C where you hope it's correct and it panics. We think we're doing it right now so it can't panic, but you have to very carefully review every PR to that area to ensure that it can't panic.
 * Alyssa: The Linux kernel has been asking for this specifically. Some way to ensure that it can't panic.
 * Brad: That would be a big step and useful.
 * Branden: So stepping back, I think a summary is that Alistair wants to modify how app verification works to add a step where TBF headers are checked. And the thinking from the rest of the group is that should be a separate step from the existing crypto verification of integrity and authenticity that occurs.
 * Phil: Yeah, I think we need to verify integrity first before looking at headers and such. If we're going to change the semantics about what a checker can do and what it can consider, that's a bunch of steps backwards.
 * Alistair: That's not a great argument. Better to go back than to merge something that has issues.
 * Phil: You have a specific use case you'd like to support. I think it's an interesting idea, but it's a new idea to affect process loading based on syscall filters or other headers. Should we go back to the design stage?
 * Alistair: This was brought up at design. But to me, I don't think it's weird that we might look at headers in the verification process.
 * Phil: But there's a lot of subtle stuff there that's at the core of the security model.
 * Alistair: I agree. But I don't think that means it can't or shouldn't be done.
 * Phil: The question is do we merge this and look into that later. Or does that need to be part of this right now?
 * Brad: I don't honestly see us halting at this point. Keeping this PR outstanding requires a huge amount of effort to keep it in sync. It seems like we have to move forward. I do NOT think that we can't change these traits in the future. If we are missing something it's possible to add things and this isn't the final design. We need people to try using this and add implementations and I don't want that to go on hold.
 * Hudson: With the size of the PR and how long it's been open, it's a huge priority to get it in. It's a large amount of work not just for Phil to rebase, but for reviewers to keep track of the changes.
 * Phil: And after it's merged, I think a further iteration on the process loading algorithm seems reasonable. We can do that and design and implement and test. Seems reasonable to me.
 * Branden: How about Alistair? Would doing that harm the ideas that you're working towards?
 * Alistair: I'm on the halt train. However, I'm fine if everyone disagrees. Especially, if we can make changes in the future. Sometimes its hard to tell if the community will accept changes to things after they go in.
 * Brad: Upfront, we certainly don't want to change things. But if we need to make a change due to a meaningful use case or performance, we're willing to do that

## AppID - External Dependencies
 * Hudson: We're out of time. Anything you want people to look at Phil, before next week?
 * Phil: The key thing is "Allowing external dependencies for cipher implementations the recurring issue of flash HILs, notably: https://github.com/tock/tock/pull/2993, https://github.com/tock/tock/pull/2248" At some point if we want crypto in the kernel, we're going to have to have external dependences that implement ciphers. We do NOT want to do them ourselves. So that's something to think about having a carveout for.
 * Brad: That seems like a good issue
 * Alistair: There's an AES GCM PR open about this too.

