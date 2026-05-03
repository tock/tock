# Tock Meeting Notes 2026-04-15

## Attendees
 - Branden Ghena
 - Leon Schuermann
 - Johnathan Van Why
 - Brad Campbell
 - Hudson Ayers


## Updates
 * None today


## ProcessID
 * Leon: Network WG call notes: https://github.com/tock/tock/pull/4789
 * Leon: We have a motivation from IPC to make ProcessID unique. What we need for IPC is a unique handle for identifying a given process. This handle needs to be sendable to userspace. And since communication is often stateful, we need to know when a process has crashed or restarted, that tells you if the state could be reset or if the handle might be invalid.
 * Leon: So we were looking at ProcessID, which has statefulness about restarts built in, but wasn't previously guaranteed to be unique.
 * Leon: AppID is another option with a long byte string for identifying an application. ShortID is another option, as a compressed version of AppID. Both are unique among running apps, although they don't inherently signal process restarts.
 * Leon: So, we're generally in agreement that making ProcessIDs unique over the lifetime of the kernel has some tradeoffs. 32-bit values could lead to a lifetime issue if there are many resets. If we make the 64-bit, the lifetime issue disappears in the real world, but there is a code size overhead.
 * Leon: There's also a question of whether ProcessID can or should be trusted.
 * Leon: AppID and ShortID have issues too though. If an app restarts, it's still got the same handle. So it could restart or get swapped out for a different version of the same process, the handle would be the same. So we'd need another mechanism for restarts.
 * Leon: More importantly, not all apps have AppID or ShortIDs, so if we can't necessarily use them as a handle... They have the option of being LocallyUnique which essentially means they don't exist
 * Johnathan: LocallyUnique was really about backwards compatibility with Tock 2.0. So I think we could require it moving forward.
 * Brad: It's really about the synchronous process loader. Apps don't need to have anything.
 * Johnathan: In the AppID TRD, if you have the LocallyUnique ShortID, then you miss out on features that require it. So it could be reasonable to require it.
 * Branden: I'm worried that there's an issue in using this in practice though. I'd love to be corrected, but I think every app is default LocallyUnique right now. And I think the only other option in Tock is putting it in the TBF Header.
 * Brad: There are a few of them. I'll look
 * Leon: One of the other things that's tangential here is that I, and maybe others, have not meaningfully engaged with AppID before. So I'm trying to understand some things. Is AppID only a concept, and only ShortID actually exists?
 * Brad: That's right. 
 * Branden: That's intentional right? Because use cases could have different implementations?
 * Johnathan: No
 * Brad: Yes. That was the design. We wanted the ability to see if AppIDs match, but there's no AppID type. There are many instantiations. So there is no code in the core kernel that you could point to and say "this is what AppID is".
 * Leon: More technical question, if I hold a reference to ProcessStandard, does that have a reference back to the full AppID?
 * Brad: No
 * Johnathan: It needs to
 * Brad: AppID doesn't exist as a type, so there's nothing to have a reference to.
 * Johnathan: Isn't there an array of bytes _somewhere_? In practice, likely in the TBF headers?
 * Brad: I think the design very consciously says that usage in the kernel should use the ShortID. You can compare and reason about AppIDs, but you're on your own for where they are, how they work, etc.
 * Johnathan: So how would two apps authenticate each other? They'd want access to each other's AppID to check it in some way
 * Brad: What would the authorization check do/accomplish? How would it work? I have an answer, but I'm not sure it applies to you?
 * Johnathan: The Android Tock applet use case. One of the apps is an authenticator app that provides one-time passwords, and the other is a banking app. The authenticator and banking app both want to identify each other to guarantee that they're correct before sending the password. The thing that identifies the app uniquely is the AppID.
 * Brad: You could assign a ShortID such that anything from company X has a certain ShortID.
 * Johnathan: Collisions are an issue. There could be more than 2^32 companies. And ShortIDs are determined when the apps are installed. Like with a counter or something.
 * Leon: There could be a hash algorithm instead, which is stable, but there could be collisions. In either case, we need to be able to map full AppID to ShortID at runtime, stably.
 * Leon: You could assume there are no collisions, and worst case if there are collisions you can denial of service.
 * Brad: These are tricky things to talk about. And it's hard to recall the discussions from two years ago. There are definitely gaps that I'm not understanding. You can't trust that someone owns any number. That has to cryptographically be authenticated. Then ShortIDs can be assigned based on that.
 * Leon: I think there's a missing mechanism for apps at runtime to get a mapping between long AppIDs which are stable and ShortIDs which are not.
 * Brad: What do you mean by stable?
 * Leon: You have some bytestring, and trust the kernel to cryptographically verify that the bytestring is acceptable. The kernel loading it and assigning ShortID makes the connection between ShortID and AppID.
 * Brad: Why do you need the long number though? If the number's in the TBF Header, any app could put that number in the header. But presumably you're saying that there's some private key that signs the app and ensures that AppID matches.
 * Johnathan: Your number is the public key of the signature. The app gets signed with the private key and has the public key as the number.
 * Brad: So you use some trusted public key to verify that it's signed correctly. The kernel already matched the public key.
 * Johnathan: The kernel doesn't know the public key in advance. It pulls it out of the header and then checks that the app was signed with the private key.
 * Branden: So if you use the public key as the AppID, and sign yourself with the private key. So you're showing that you own the private key. No one else can be using that public key as their AppID because they wouldn't be able to sign themselves.
 * Leon: Then you can use the Public Key to identify the application.
 * Brad: So you're saying the other app has a list of public keys that it trusts. Okay. So the missing piece is the ability to ask for the Public Key that identified the app.
 * Leon: The interface I want is that an application gives an AppID buffer. Then the kernel gives two results, either yes I've verified that this app was signed with the matching private key and here's it's ShortID, or no I haven't verified this app so it doesn't get a ShortID.
 * Brad: Does it give LocallyUnique or not installed?
 * Leon: I wouldn't run as a process
 * Brad: Right now that would mean it basically doesn't exist. We don't keep track of those.
 * Johnathan: There could still be data under the ShortID.
 * Branden: There's no ShortID if you haven't authenticated the app
 * Johnathan: There should be a mapping of previously-assigned ShortIDs. So we don't reuse them
 * Leon: Now we're a bit off track. That's a question of relying on ShortID having a persistent AppID mapping with no collisions.
 * Branden: So backing up, Brad sent some links for how ShortIDs are made.
 * Brad: Basic app checking has multiple ShortID options: hash of name, null. https://github.com/tock/tock/blob/master/capsules/system/src/process_checker/basic.rs#L249 There's also the nRF version which is based on key https://github.com/tock/tock/blob/master/boards/tutorials/nrf52840dk-dynamic-apps-and-policies/src/app_id_assigner_name_metadata.rs
 * Leon: So we can assign ShortIDs for apps which have not been verified. As long as mapping an AppID to a ShortID doesn't collide with them. As long as we don't rely on those ShortIDs for authentication or security. This is a bit different from my interpretation of the TRD.
 * Leon: Okay, we were concerned that ShortIDs were not prevalent in Tock at all.
 * Brad: Security sucks and is annoying. People don't want to deal with it. Storage requires it. IPC might require it.
 * Leon: We could still not sign apps and communication without authentication though, right?
 * Brad: Yes.
 * Leon: Then we still need ShortID to AppID mapping for authentication.
 * Leon: We still need some restart handling for IPC.
 * Johnathan: There could be some session ID there.
 * Leon: How would that work? If the app restarts it would go back to the initial value.
 * Branden: We could have a restart counter attached to processes, and have both restart counter and ShortID be the handle we give to userspace.
 * Johnathan: We could have a counter which starts at one and each time a process restarts it increments. Without involving the kernel?
 * Leon: We do want something in the kernel. We want to ensure that the handle is consistent across all IPC capsules.
 * Leon: We currently don't have an interface for knowing if a process restarts for capsules. That's isolated from capsules right now. But you'd need something for IPC to know about it
 * Brad: Is it not enough to just notice the next time something tries to communicate?
 * Leon: IPC could discover on the next interaction, but that requires state to compare from previous to current.
 * Brad: Why does it need to compare the state?
 * Leon: You have a process copying from ShortID A to ShortID B. How would we know if ShortID B restarts? Either the capsule can inspect some state, or the app gets some notification that it occurred.
 * Brad: Okay, you're saying that there's some way to get ProcessID given ShortID.
 * Branden: We definitely need that. We need to know the ProcessID to copy. We get a handle from an app and then use that to access the grant space for the destination so we can copy data into that.
 * Brad: So App A gives IPC a handle. Then IPC asks the kernel to get the ProcessID given the handle. Then you could look up the restart counter in the grant?
 * Branden: I don't understand this
 * Leon: You could have a global counter. App A tries to discover App B. The IPC mechanism copies to global counter and increments it. Then creates a grant region in App B with the counter value. Then you return to process A and give it the ShortID and the counter value.
 * Branden: Okay, I do understand this now. This makes sense.
 * Brad: So the counter value could not match because you restarted and then talked to someone else
 * Leon: This is recreating ProcessID though. So we're just passing ProcessID + ShortID essentially. When we do this IPC grant-based counter, that counter being a global variable across means we can look up by iterating grants.
 * Branden: Why not just use ProcessID then?
 * Leon: Essentially we'd be creating an IPC-specific ProcessID
 * Brad: I agree this would be doing ProcessID and it doesn't makes sense to have two of them. Using ProcessID internally seems fine for solving issues. The question is whether it's public
 * Branden: IPC docs describe the handle as an opaque handle and expects userspace to not expect it to be permanent or reliable.
 * Johnathan: What if we stick with 32-bit ProcessIDs, but boards specific number of apps times number of restarts, and we enforce those to guarantee no overflow.
 * Leon: It's problematic for some use cases. Ephemeral apps that are in memory or dynamically loaded apps.
 * Brad: I think that's a compelling reason. Am I the only one who doesn't want to switch to u64?
 * Branden: As long as it doesn't take too much space, I don't care.
 * Leon: I haven't been able to get rid of the 500 bytes of code space, but I've spent hours on this and I can't figure out why it added that much usage at all. It could be that a different version of the compiler would change that.
 * Branden: So it's not just a bunch of u64 handling stuff
 * Leon: No, that's already there. The code size is added just by adding 32-bits of space to the Process struct.
 * Brad: You tried quite a few boards?
 * Leon: It's average 500 across all boards.
 * Leon: I was all for using ShortID for a bit there.
 * Branden: For liveness reasons, we need something that tracks
 * Leon: If there was a counter per ShortID, we'd need a new data structure.
 * Leon: You can discover based on various things, process name, something the process sets. Whatever.
 * Brad: Can't we have some opaque IPC Handle?
 * Branden: Is it okay that internally that's actually just a ProcessID? As long as it's guaranteed to be unique
 * Leon: And it does need to be unique for other kernel stuff
 * Brad: I think that's okay. I'm imagining this as documentation saying there's an opaque handle with some guarantees. And then that could happen to be ProcessID right now.
 * Leon: Yes, and I'm happy to block getting the actual 32-bit value under a capability.
 * Leon: Plus, we could make it so we can switch out the underlying implementation as long as the size is stable.
 * Branden: I was worried that ProcessID as a handle was going to cause some security concern that I hadn't thought of.
 * Leon: I think ProcessID isn't confidential information. 
 * Brad: What about using ShortID as 32 bits and today's processID as the other 32 bits, for this handle?
 * Leon: That could work.
 * Brad: That would have restart counter, and have uniqueness.
 * Leon: For sure you're talking to the same application. It would be probabilistic whether it's the same instance, but in practice reliable enough. Reasonable assumption. Small but unlikely chance that you could be talking to a restarted version of the application or different version of application.
 * Branden: What's the gain of adding ShortID?
 * Brad: I'm not sure
 * Leon: Wouldn't need a 64-bit ProcessID for uniqueness for IPC. Would ensure that collisions don't leak secrets to different applications.
 * Branden: The cost would be requiring a valid ShortID. Can't be LocallyUnique. Not sure how big of a cost that is.
 * Leon: It does sound like a 64-bit handle is needed for IPC. So we should update that now.
 * Brad: For restart handling, what if Process A gets a handle to talk to someone. There must be some other check that's happening.
 * Branden: Yes, it's possible to make up a handle. Some options. First, you could not care (toy platforms and testing). Second, you could sign all your apps before loading them, and trust your own signed code not to make up handles. More complicated solution we talked about is having process descriptors, like file descriptors, where the kernel keeps track of which processes you're allowed to talk to. That requires dynamic allocation though, so it's more work than it sounds and we don't intend to do that by default.
 * Leon: There's another mechanism that matters here. As a receiver of a message, you get a kernel-authenticated handle to a sending process. So you can know who they are. Someone could spam you and consume your upcall queue, and for this we have an allowlist mechanism in IPC where you can say who you want to get messages from. So attempts to send messages are budgeted against the sender but don't cost the receiver.
 * Branden: You could build a userspace system for handling this. Send messages back and forth to do userspace authentication.
 * Brad: Back to Johnathan and AppID briefly, if we need that mechanism it's not out of the question, just not part of the current implementation. I don't know that it's talked about, but it's not prohibited.
 * Johnathan: This conversation today reminded me of some concerns about AppID stuff. But I wasn't putting in effort to write the TRD so that's okay
 * Brad: Some way to check that the authenticator used some public key and to give something to someone who asks isn't a mechanism we have but it totally could be.
 * Johnathan: Specifically the AppID string is what makes the whole system useful. It needs to not be secret or it's useless
 * Brad: We have to be careful though. If you're using a company private key, you can only have one app.
 * Johnathan: Probably public key plus app number. Or one public key per app
 * Brad: Yeah. And that complexity we never really got to as we don't have a way to name that format. How do we name and communicate that and share that format with other things
 * Johnathan: I put thought into it, but the AppID pushes that out to the board and lets the board define that. So different boards could have different AppID meanings. An app with a hardcoded AppID just has a string of bytes. It doesn't have to understand it.
 * Brad: Right, but someone in the kernel does.
 * Brad: So I can see how the kernel would manage this, but communicating outside of the kernel is open
 * Johnathan: I've put thought into that

