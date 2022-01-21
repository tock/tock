# Tock Core Notes 2021-09-10

## Attendees

- Branden Ghena
- Alexandru Radovici
- Leon Schuermann
- Gabe Marcano
- Phil Levis
- Johnathan Van Why
- Jett Rink
- Pat Pannuto
- Brad Campbell
- Amit Levy
- Vadim Sukhomlinov


## Updates
 * Hudson (posted online): Out this week, but my update for the week (outside of the ongoing panic process info discussion) is that I have tried updating to the Rust 2021 edition and found decent code size savings as part of the transition (4kB, out of tree), presumably thanks to disjoint closure capture support.


## SHA/Digest PR
 * https://github.com/tock/tock/pull/2697
 * https://github.com/tock/tock/blob/efca29265736832c68809a5e580ab1a69ee8632b/kernel/src/hil/digest.rs
 * Amit: This came in a bit ago from Alistair. Phil thinks it's in pretty good shape and we should take a look at it.
 * Phil: We've talked back and forth on it a bit. I think it's in a good place and could use last comments.
 * Phil: If there are comments we feel strongly about, please put them on the PR. Especially since Alistair isn't on this call.
 * Branden: What happens if no `set_mode*` function is called before run?
 * Phil: I think an error code makes sense there.
 * Branden: Oh, in fact a comment says just that. Specifically ENOSUPPORT unless there's an obvious default.
 * Amit: Anything else?
 * Leon: Why do we have a verification function and trait?
 * Phil: It means that you don't have to allocate a buffer for the hash because it's handled internally.
 * Leon: I think I looked at this in the past and didn't see a way we could get away without a buffer usually. I've forgotten why I thought that was the case though. Need to reread and think for a bit.
 * Amit: I'll commit to adding some small comments (about comments) to this PR, but after that I'm ready to approve.
 * Jett: One thing I'll bring up (and add to the PR), it looks like there are two layers of client callbacks. It seems that this could make for confusing code to debug. Two ways to register clients. You could do both, and then what would happen.
 * Phil: I think `set_client` just calls the underlying ones (all three of them).
 * Amit: So maybe there should be a default implementation which does that. I will comment.
 * Amit: So just to clarify, the thing to do is as a client normally you'd just call `set_client` which sets everything. But you could have different clients for different parts if you wanted and would call those separately.
 * Jett: Another way to do this, there's a `verify_done`, etc., but maybe you don't have to receive them. So you could have one client which has three functions.
 * Amit: This does require the implementation to have three fields for three different clients. You could instead split into multiple clients at a higher layer. So the question is whether the common scenario is multiple separate clients or one client?
 * Leon: I think that multiple clients makes sense when we have hardware that can calculate a digest, verify, or do both. But if we decide to not have verify separate, then a single client would suffice.
 * Amit: Hold on, you don't need different traits to have multiple callbacks for the client. So the question is whether we should have different structs with different functionality implement the different callbacks.
 * Leon: I think you wouldn't. If we had something that was both calculate and verify, then it could maybe do both independently. So then multiple concurrent clients would make sense.
 * Phil: I advocated splitting the traits for least privilege. They both need data, which would then maybe be shared. So you'd have a single client trait which has three callbacks, but maybe you don't have access to the trait which creates that callback anyways.
 * Jett: The implementation if you had three would just to panic in that callback or nothing, or whatever makes sense.
 * Amit: This discussion could probably best go on the PR instead of the call.

## AppID TRD
 * https://github.com/tock/tock/blob/9ba11c97b8fb39082b63567083b8ff01737702eb/doc/reference/trd-appid.md
 * Amit: Question from Johnathan, does it make sense to have multiple applications running concurrently with the same ID?
 * Johnathan: One question we debated before is whether all applications are distinct. Different processes at different times could have the same AppID. So a rebooted app has the same original ID so it can access shared storage.
 * Johnathan: Can you have two application binaries on a Tock system that are part of the "same" application and share an AppID but run separately. Or should they stay separate AppIDs? Also, what if you run one flash executable in two applications. Do we support that? If so, they would end up with the same AppID in the current design.
 * Johnathan: One advantage of distinct AppIDs in all cases, is that IPC is simpler. There's only one target of a message.
 * Johnathan: But one downside is that a notional "application" could be made of multiple processes.
 * Leon: I think it's reasonable for multiple concurrent processes which share the same AppID. If we want to specifically address a process then we have ProcessID for that case. A good solution to some of the issues is a mechanism for discovering all the ProcessIDs for a given AppID (possibly it's own AppID).
 * Amit: I'm worried that the TRD uses AppID in the way that the code uses ProcessID
 * Phil: It might be, but if so, it's miswritten
 * Phil: We also have process name, right? If we walk through the use case, it would be weird to have to processes with the same AppID and the same Name. So IPC could use AppID and Name.
 * Leon: Why I think it's a good idea to have multiple processes with a shared AppID. There could be multiple instances of a single binary, which would share AppID. The code they're executing could end up different due to sensor values. But for accessing storage or keys, the two processes could be acting differently, which means they're essentially the same case as different binaries which share an AppID. So I think we should allow sharing for both or neither.
 * Amit: I'm not sure I understand the use case for having the same binary running multiple times concurrently. Unix style processes allow binaries on disk to run multiple times, but they're totally independent. Another model is Android activities, where application names are the global identifier for a binary, and you can't have multiple of those. It seems that the use case that Tock was thinking of is more like activities. Could someone articulate the way we might use multiple instances of the same binary?
 * Leon: We would have a significant decrease in code size for multiple processes. Think of a game with two players. You've got the same code on each player, just one subtle startup difference between the two of them which then changes their purpose, even sharing a codebase.
 * Amit: For that use case, all the ways we'd want to use any application identifier, we'd want those to be distinct. Different peripherals would be given to each. IPC would go separately to each. So I see that as an argument for having the ability to format binaries on flash so that two TBF headers point to the same code.
 * Johnathan: An assumption you made is that two processes with the same AppID would be treated different for security purposes. I was assuming they would be treated identically. So that's another decision to make.
 * Amit: Well, I was saying that in Leon's example that we would want those two processes to be distinct for security contexts.
 * Phil: But that security context is dynamic. Where is the fact that instance 1 gets to access these IO pins and instance 2 gets to access those IO pins.
 * Amit: In Leon's example, I'm saying that even if they're generated from the same code, the two processes should be treated a seperate things with separate identifiers.
 * Leon: So you're saying you want AppIDs to be persistent process IDs.
 * Amit: Weaker than that. I want to see a use case where two processes should share identifiers. So when do we want to name a group of processes?
 * Leon: I think we have a use case for a group of processes. But AppID isn't the only mechanism for it. Storage regions are one great example. If AppIDs are the only way to access that, then we need to handle this. But if we have an intermediate layer that maps multiple AppIDs to one region, then they can be unique.
 * Phil: So the question is that are AppIDs for security, where a group of processes should share it? Or are they for communication naming, in which case they should be unique? My take: there are lots of situations where we want to share security, but that cost for duplication seems small compared to the cost of doing name lookups.
 * Leon: So if we do say AppIDs are unique in the kernel, then we do have to add infrastructure to allow, say multiple AppIDs to access a single storage region.
 * Amit: We would need some way to express that. But it seems to me more likely that you'd want different binaries to access a single region, then two processes of the same binary to be separate regions.
 * Leon: I was just using two processes of the same binary as an example of how this problem occurs. I'm thinking in terms of different binaries now too.
 * Amit: Okay, then we would need some way of naming storage regions differently.
 * Leon: If we do want to say that different binaries have different AppIDs, then we should also say that the same binary in different processes should have different AppIDs.
 * Amit: I agree with that.
 * Amit: Does that resolve your question Johnathan?
 * Johnathan: It sounds like we're converging on giving distinct processes distinct AppIds, and then having a separate mechanism for naming groups.
 * Phil: Yes. And I think we have a good sense of how we want to manage storage and security. The multiple processes in a single app are still pretty theoretical though. We don't want to preclude them, but we don't need to design around them yet. We can come up with a group naming solution later when we have a real use case, maybe like a bitmask for the AppID.
 * Jett: So the AppID is really treated as an ACL_ID?
 * Johnathan: More like a persistant process ID. To answer the root of your question, that was the original purpose of the AppID TRD
 * Jett: So is there still going to be a separate ProcessID? What if it restarts?
 * Phil: Exactly. So if a controller process reboots, it should have a new ProcessID, but the same AppID.
 * Amit: Yeah, memory and callbacks need to be invalidated. But security settings are still the same.
 * Alex: I always thought that multiple processes in an application should share AppIDs, are we moving away from that?
 * Phil: Yeah. We realized we'd never thought it all the way through.
 * Brad: One comment to push back on what Phil said about handling multiple processes in an app before we have a use case. I think it's reasonable to separate what we want to build and use from how we want to conceptualize and define things. I think we should have an identifier that covers a conceptual application. And we should be very specific about what the thing we want right now is, which is like a TBF header ID. So that we are clear about what we mean.
 * Leon: Making AppIDs persistent but unique, I think opens up room for making groups of AppIDs. The question was whether AppID should be that group or if that should be a separate thing. I think there's enough justification that we can build processes into a logical application later, but should handle the one-to-one communication case now.
 * Alex: But the name AppID is confusing. Maybe it should be renamed to persistent process ID or something like that.
 * Johnathan: Or maybe securityID or something like that. We historically had the name app as a vague thing for years that sort of means running process but sort of not.
 * Amit: What do people think of adopting the Android language of "activity".
 * Leon: I ran away from android for a while, but I wouldn't know what to make of that. We should probably add our own Tock-specific more verbose lingo. That's just my opinion.
 * Johnathan: I agree that I'm not familiar with Android.
 * Alex: Android has separate services and activities too. The framework is similar, but a little weird.
 * Phil: One challenge with PersistentProcessID. If you update the binary, it should still have the same AppID.
 * Phil: Three distinctions: particular binaries, binary which goes through updates which can run, and multiple processes which work together.
 * Leon: You could also have multiple instances of a program in flash.
 * Phil: You wouldn't run two versions of a program at once. You wouldn't have two key managers, for instance.
 * Leon: But I can run VIM twice
 * Phil: But this isn't a multitasking system in the same way. We also wouldn't fork ten apache processes.
 * Amit: Okay, this seems like something that we can't come up with the right name in 13 minutes. It does seem important and non-obvious.
 * Phil: I'll think on this and send a V2 next week. I'll try to come up with a better name.

## Remove process debug tracking and prints for downstream client
 * https://github.com/tock/tock/pull/2759
 * Jett: Debugging information costs a lot of flash, and we want a way to remove it for a downstream board. We want a solution that allows us to remove it if we want to. We've thrown around some requirements and solution ideas on the PR. I thought it might be good to discuss.
 * Jett: To talk about the PR itself, the code that Hudson uploaded uses the config struct that already exists. And then uses a boolean check in debug prints to early return from debug code. The compiler is smart enough to remove the content of the function, which saves like 12 KB of flash. There is some pushback on the nightmare of maintaining multiple config options.
 * Jett: So the question is how should downstream users select different options like this? Rust configuration, command line option, kernel resources at the board level??
 * Brad: I think we need Hudson to really have a good discussion on this, unfortunately (and he's out this week). We already have the interface we want, it's just that the darn rust compiler doesn't elide code that isn't used in this case. So what I don't like about the config is that you both have to configure and write your panic handler correctly.
 * Jett: it was two things. The debugs and the panics. Hudson split them out into two different things. It is annoying to make sure that the panic is written right and the config is set right. For debug printing we don't have those two things.
 * Jett: We can wait until Hudson is here next week.

## RSA PR
 * Amit: Alistair requested having a discussion on this, but he can't make these calls. We should schedule a separate meeting for anyone interested to join.
 * Phil: I can send an email to the helena mailing list and see who's interested.
 * Jett: Could I get privilege to send emails to that?
 * Phil: (Goes in and fixes privilege for a bunch of people)

