# Tock Core Notes 4/3/2020

Attending:
 - Branden Ghena
 - Leon Schuermann
 - Johnathan Van Why
 - Jean-Luc Watson
 - Philip Levis, Alistair
 - Samuel Jero
 - Garret Kelly
 - Pat Pannuto
 - Hudson Ayers
 - Andrey Pronin


## Updates!
 * Phil: HMAC HIL stuff, can discuss later
 * Pat: STM32 Discovery board (#1674) will be merged soon! Very exciting.


## Threat Model
 * Johnathan: Capsule referred to lots of things historically, but nowadays is mostly "untrusted code". We should make a specific decision rather than an ad hoc one.
 * Pat: Brad thinks it should be kept simple "Just say `capsule` and use it to refer to untrusted code"
 * Pat: What do we call things that are not in the kernel but are trusted? Things in boards/ or chips/ for instance?
 * Johnathan: I don't think we need a term
 * Pat: Worried that code in chips/ is trusted but not as vetted, and should the threat model note that?
 * Phil: No, that's just a procedural thing. We probably should be vetting code there just as well.
 * Johnathan: There's also no extra hardware or software boundaries around chips/ stuff versus kernel/ stuff
 * Samuel: I've found it helpful to distinguish device drivers from kernel code
 * Pat: I'm lightly concerned, because it's aspirational, that there are things we might never get to. For example, the Discovery kit will never be as vetted as the OpenTitan stuff, but to someone new they look very much the same.
 * Johnathan: That's a statement that code review quality can vary among trusted code.
 * Pat: That's the reality. There are fewer eyes on less used chips.
 * Phil: Eyes on code is hard to measure. Really want consideration and testing.
 * Hudson: I don't think we're going to say anything in security model about parts of Tock being "proven"
 * Phil: We'll have unit tests and coverage, but not proof. We could conceptually have reviewing guidelines, but time spent reviewing is hard.
 * Samuel: There's a difference between "trusted" versus "our confidence in the code". All code in chips/ etc. is "trusted" but we're really talking about our own "confidence" in it.
 * Johnathan: We have a thing on audited versus un-audited code, for third-party dependencies. Maybe that should cover the review quality. Maybe we should specify the review process per directory...
 * Pat: That might be the right thing to add. The threat model states what things are and are not trusted. But for things that are trusted, maybe we should say there is documentation (review history, etc.) into what confidence you should have in this code.
 * Johnathan: Is there anyone who disagrees with adding review quality?
 * Branden: I'm concerned that there would be an expanding gulf between aspiration and reality for code we accept into the repo and our review process.
 * Pat: I would hope this is a recording of what has been done in practice.
 * Samuel: We could have a couple levels of chips/. "Intitial" / "first-pass" compared to "well-supported" or "widely used". A possible third level of "verified" in some theoretical future
 * Johnathan: I would support that. Do we think we could group boards into those definitions?
 * Leon: One inspiration would be the primary versus secondary targets as Rust does. Not sure of their reasoning.
 * Samuel: would want "experimental" category
 * Branden: I think that's a good idea. We would need a clear denotation of what each state means and how things move. Should this be part of the threat model? Or just in the chips/ folder?
 * Johnathan: It would be part of the code review part of the threat model.
 * Pat: Summary: The architecture of Tock requires certain things be trusted. Engineering maturity still happens over time. Experimental things must be "trusted", but we'll explain how we mark confidence.
 * Pat: Summary: consensus is that all capsules should be considered untrusted. There is code that exists in the kernel that is not a capsule, but is just kernel code, and we'll have a new way of categorizing the level of trust people should have in this code.

 * Jean-Luc: Application isolation guarantees, data can't be accessed by untrusted capsules. There's nothing stopping a capsule from sending data from one application to another.
 * Johnathan: We have this concept that capsules should be limited in what they can access and do. When board integrator selects capsule, it should be clear what it can or cannot access. But Rust doesn't exactly encapsulate this. If capsule A and B both connect to capsule C, C could be sharing data between A and B. There's a rule that virtualization drivers, including system call drivers serving multiple applications, not pass data between clients, but you have to trust that the capsule doesn't do so, or not trust that capsule at all. Moe generally, in the board file it's not clear how data is allowed to flow between capsules.
 * Jean-Luc: Virtualization capsules could do this, and we do trust them. So maybe we need explicit terminology for those capsules which we have to think extra about
 * Phil: We trust that capsules won't compromise the system, but we don't trust them for availability. We need to define which things we trust.
 * Johnathan: So for virtualization we need to trust that this capsule won't share data among clients.
 * Leon: This is particularly true for crypto code
 * Johnathan: Current threat statement, capsules can access whatever they can access without unsafe. Trusted code should use capabilities to limit things capsules shouldn't access. The missing part is stating "what shouldn't a capsule access".
 * Pat: Data flow integrity. Should we be saying that data from one application should never affect another application as a policy?
 * Hudson: Earlier, Pat mentioned worry about threat model being too far away from what we do that we might never reach it.. That strawman you just provided falls too far into that threat possibly. Because I'm not sure how we would enforce that data flow integrity.
 * Johnathan: Maybe it has to be left vague because we can't specify it.
 * Pat: Answer could be that right now we can't guarantee anything about this.
 * Hudson: What about "Any application's data will only be shared with a capsule and capsules that capsule communicates with". Essentially that data from one application is never shared with another application, only with kernels.
 * Johnathan: But right now we just have to trust that capsules do that.
 * Phil: "Trusted" is a statement with shades of grey. What responsibilities this code has to maintain threat model. We can say "Capsules are expected to not do these things". Some are language and some are code review. Memory safety is language. Not share data across clients is code review.
 * Leon: Could be part of a checklist for code review. Everything we guarantee that's not part of the language.
 * Phil: I'd be scared to ever check one of those...
 * Pat: Maybe the contributor should have to check the box, just so they think about it.
 * Leon: Maybe not with the word "guarantee". Maybe more like "to the best of my knowledge"
 * Johnathan: I can add a paragraph pointing out where and why we are leaving things vague.

 * Johnathan: Third concern is whether application and process are synonymous. Is an application a higher level thing?
 * Pat: I think replacing application with process in the threat model is the right way to go. At minimum a process is the minimal instantiation of an application. After a reboot you might have a new process in service of the same application. So process is a proper subset for application.
 * Johnathan: That starts to get at the problem of volatile storage. And restarting processes at runtime and resetting grant regions. If you update a binary in flash, how do we let a new process access that flash space again? Maybe with some kind of app id.
 * Leon: I like the idea of app id that is persistent. Maybe a hash of the binary.
 * Johnathan: Updated apps might still want to access data in storage, but would have a different hash. So maybe some kind of private key instead. Could add app id to TBF header. And all processes with that same app ID could share that nonvolatile data
 * Pat: Current implementation is that there is an entire page reserved as part of the TBF header and remainder of page can be used by the specific process. Might not be the right thing, but that's the current implementation.
 * Leon: I definitely think we should have a unique id in the TBF header and use it to guide access.
 * Pat: I'm worried that we should pull this application idea from threat model for now and focus on process safety. And then future of Tock will look at persistent applications.
 * Johnathan: Seems reasonable to me. I expect opentitan to be moving forward with long-term application identities due to crypto concerns.
 * Leon: Agreed. We should be discussing application IDs moving forward though.
 * Pat: I really don't think we've considered persistent "applications" (rather than processes) yet and don't have a clear view of the space. But are interested in exploring it.
 * Leon: I'll throw some ideas onto the main list.
 * Garret: That also fits as part of the key-value store PR right now


## HMAC Digest HIL
 * Phil: Alistair has been doing great work to bring up HMAC on opentitan. One question is that this work is for opentitan but it's a HIL, so it's core because it's a HIL. Crypto is starting to become incredibly important, first sketched out by Daniel Giffen for AES a while ago for the SAM4L. There's a generic data path trait that has encrypt/decrypt and depending on what encryption mode you want, you have separate traits to set that. Could for example have an AES structure that implements the AES datapath, but then has counter mode and CBC mode traits implemented. That's one approach with a common datapath trait.
 * Phil: The other approach is to have separate traits for each encryption mode. AES CBC and AES CTR. SHA 256 vs 512 vs HMAC vs other digest types. We should decide on current path or specific trait path.
 * Phil: Brief summary is that common data path trait advantage makes things more pluggable. So flexibility is good. Downside is that CBC and CTR have different guarantees. And so you might not want them to be interchangeable. So we should make a decision to guide these new crypto traits that are coming up.
 * Pat: This could be the tiny-os too many layers problem, but couldn't you layer specific stuff under general stuff?
 * Phil: But then you use the generic trait, but still have to keep the assumptions of your specific piece. Probably wins and losses from both sides. It would be better to go with a clear answer if we can.
 * Pat: We should follow up on this in writing somewhere.
 * Phil: I'll make a post to tock-dev email list about this since it spans multiple PRs

## Code Size Status Updater
 * Hudson: I set up this Travis code size diff reporting. But my method only work for PRs originating from branch on main tock repository. Travis doesn't expose that stuff to a fork, because otherwise things could leak. So result is that my upgrade doesn't work for _most_ PRs because they come from forks of tock. : (
 * Hudson: Can we publicly expose an OAUTH token that only allows adding statuses to PRs? Technically it would allow anyone to add statuses to PRs, obscenities, with the identity of whichever account made the OAUTH token. It would not allow people to override Travis status since it would be a different identity.
 * Hudson: We could also look into github apps. But they seem nontrivial.
 * Branden: Travis runs and posts the status for forks.
 * Hudson: When Travis finishes, it says succeeded or did not, but doesn't expose travis token to code that's running. It doesn't call to github API from scripts, but instead sends results to Travis server and the server has an internal token it uses to post to github.
 * Pat: Where is the checking code executing now?
 * Hudson: My stuff is a travis script that runs on the travis server after the build ends.
 * Pat: So because there's no cloud service running that holds the key to post, that secret would have to be in the shell script.
 * Hudson: Yeah, basically.
 * Branden: Could we just spin up our own cloud service?
 * Hudson: Yup.
 * Leon: I think the NixOS people do something similar. I know the guy doing the CI stuff and can ask how he's doing that. Posting the closure size. Not using travis for that, but takes build artifacts that another server fetches from travis and then posts to github.
 * Hudson: That sounds great. I would love a contact.
 * Pat: Do github actions do this? (https://github.com/actions/labeler)
 * Hudson: No. Actions use secrets github tokens and therefore aren't exposed for forks. It's frustrating. There are some other services that sort of do this, like check run service, that have a server but I'm not sure that they are any better than publically exposing the token, because I could just send arbitrary things to that server.
 * Pat: Well hopefully we can learn something useful from the NixOS folks
 * Hudson: Yes, I was surprised this was so hard. I'll make a github issue with problems with the current approach.
 * Branden: Have you searched github for other organizations that post to statuses?
 * Hudson: Code coverage is a common service, it's a github app that's similar to what we want. It's an app that accepts code coverage reports in a particular format and then posts them on the status of a PR. So handrolled apps can do this, but they don't overlap with exactly what we want to report here.
 * Hudson: I'll make an issue. Thanks for the feedback.

