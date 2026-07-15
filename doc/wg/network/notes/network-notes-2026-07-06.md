# Tock Network WG Meeting Notes

- **Date:** July 06, 2026
- **Participants:**
    - Branden Ghena
    - Tyler Potyondy
    - Rohan Sachdeva
    - Leon Schuermann
- **Agenda:**
    1. Updates
    2. Tock OpenThread Certification
- **References:**
    - [Tensile Thread Tests](https://github.com/tock/tock-hardware-ci/tree/main/tensile)


## Updates
 - Branden: Network working group ended up in hibernation for a bit since we were all busy


## Tock OpenThread Certification
### Overview
 - Rohan: Former MS student at UCSD, just graduated. Worked on OpenThread on Tock as part of MS studies
 - Tyler: We haven't synced up on everything Rohan finished, so I'm excited to hear this too. Motivation was libtock-c port with OpenThread, where it's been challenging to test the stack. Seems to have failures over long terms (5+ days). One thing we really wanted was to at least run the testing scripts from OpenThread on our setup. These seem to be what their certification test harness uses. So the high-level plan was to try this stuff out. Hoping to integrate into our nightly CI at some point, which would be better tests on the thread stack.
 - Rohan: Tests sorted into various families of tests. ~150 of them. Tests need 1-32 boards, one under test and other "golden" test devices. Also a sniffer, which you can use another nRF board for.
 - Leon: The sniffer would need the nRF52 USB directly connected, not just the JLink.
 - Rohan: Yes
 - Branden: How long to run the full test suite?
 - Rohan: About 2-3 hours
 - Leon: Does it gracefully recover from mid-test failures?
 - Rohan: Yes. It keeps running. Also has ability to rerun failures.
 - Tyler: This is so exciting to see! Really great work.
 ### CI/CD Pipeline
 - Rohan: Interested in CI/CD step. I can manually trigger a test suite to run on the six boards I have here, but I'm interested in what the Tock preferred method is.
 - Leon: Yes. Treadmill is a somewhat reliable hardware testing setup (https://github.com/treadmill-tb/treadmill). There's a scheduler that can request a subset of boards, run scripts to load code on them and review results. Platform is undergoing a refactor/rewrite for reliability.
 - Tyler: There are some existing OpenThread tests using treadmill. https://github.com/tock/tock-hardware-ci/tree/main/tensile We could add your tests in a similar way. Is there a limitation to having a 6+ nRF boards hanging off a single Raspberry Pi?
 - Leon: No, not really. Just a stability issue, and an issue of resource scheduling. Long term we may want to have a way to combine separate nearby RPis, but for now you could do that.
 - Rohan: One test goes all the way up to 32 boards, although most are 6 or fewer
 - Leon: From a testing perspective, is it important for all the boards to be attached to the same computer?
 - Rohan: I'm just running stuff on one machine with a big USB hub right now.
 - Leon: Yeah, totally reasonable. For 32 boards though, that would really lock up those boards. We'd love to spread them across computers for orchestration in the long term.
 - Tyler: Leon and Branden, could this just be added to the tensile tests?
 - Leon: No issues with that. This can live as a separate script for now. We should expect to scrap a lot of the prototype Treadmill stuff we're using now.
 - Leon: Right now there's also very limited documentation on how to write tests. It made it hard to write bigger tests.
 - Tyler: Our existing Thread stuff just runs on the Raspberry Pi with custom stuff. It doesn't depend on anything Treadmill-specific, other than running on hardware. So for now we can continue on that path.
 - Leon: Actually, the Tensile tests have been more reliable than the rest of the tests. So no issues there
 ### Bugfixes
 - Rohan: I had to make some changes to boards main files, and some changes to the libtock-c port. Nothing to the core kernel
 - Tyler: What about changes to the OpenThread files?
 - Rohan: Some small stuff, nothing major. I have a repo with stuff
 - Leon: You also mentioned a couple bugs. Were those bugs in the userspace integration side? Or were some in capsules?
 - Rohan: All on the userspace side.
 - Tyler: I'm a little suspicious that we're passing some tests as full-thread devices and routers. I thought we didn't have the functionality to be a router, since the 15.4 capsule doesn't support a certain type of acknowledgement we need. I expected minimal thread device tests to pass. I'm surprised about full-thread and router tests though. I expected a ton of these tests to fail, and then we'd fix the failure cases.
 - Rohan: I should double-check that.
 - Tyler: We can discuss further offline
 - Branden: Which roles does the test suite cover?
 - Rohan: Everything. minimal-end-device (MED), sleepy-end-device (SED), full-thread-device (FTD), Router, Commissioner, Leader, Border
 - Branden: How do we pass border router tests without another network connection?
 - Tyler: Actually, that's possible. Might just be testing the nRF side of things. I believe Marshall at UVA got border router working on Tock
 - Branden: But you said we couldn't be a router due to ACKs?
 - Tyler: Yeah, confused. It's possible it just gets by without the proper ACK support. We needed ACKs on receiving a packet, that's done in software. The flip side is we also need to listen for ACKs on transmit. Right now we just assume our message is acknowledged, rather than listening for it. So that might be why it's working. This means resends don't work, but in a setup with boards right next to each other, that wouldn't be an issue
 ### Validity
 - Rohan: Some of the tests are hard to confirm that they're validly passing without the paid setup. Particularly the Sniffer stuff
 - Leon: We don't have the data to figure that out?
 - Rohan: Right. We can see sizes, but not expected data values
 - Leon: Is that a business decision by them to not publish that information?
 - Rohan: Not sure.
 - Tyler: I'm okay with not having every test pass. This is already a great step forward. At some point we'd be reverse engineering the real paid test suite. The packet sniffing stuff might be hard to set up, and just having reasonable confidence without buying the harness is a big part. We could pay for the real harness if we want to, which runs on Windows.
 - Rohan: Right now I can check that certain packets arrive, but byte-level scripts are proprietary
 - Rohan: I also only tested with 6 boards. I need more hardware for the 32-board tests.
 - Tyler: We have more boards at UCSD that I can set up to run the big tests.
 ### Next Steps
 - Rohan: Establishing validity seems like a first step. Then making it usable by others and replicable would be great. CI/CD would be a great step after that.
 - Leon: Having scripts stay independent of CI/CD to start is really useful
 - Branden: What about libtock-c changes?
 - Rohan: Some buffer sizes. A few other small changes. Might open those as a separate PR.
 - Tyler: How did you find those? Did you discover these from failing tests?
 - Rohan: Yeah. I explored the failing tests and fixed stuff. There were definitely some failures at first.
 - Tyler: We should prioritize the libtock-c changes then. As we really don't want to lose those.
 - Rohan: I'm also going to have Tyler try this himself to see if it reproduces.
### Thread Group Conversation
 - Tyler: Cost to join Thread Group and do certification would be roughly $7k. Not nothing, but not crazy. What does the Network WG think about this?
 - Leon: There's some discussion on the Slack thread. Certification works on a real device, not an OS.
 - Tyler: Certifying a board lets you use Thread and say you're using Thread, and the logo.
 - Tyler: Almost everyone certified right now has the stack. We're actually interesting because it's disconnected across syscalls. So if we certified a board and libtock-C combo, it might be easier for other to support certified stuff on their own boards.
 - Branden: There's value in saying "look, Tock is certifiable. We have a board that was certified"
 - Tyler: Or IPC expose OpenThread, which could give really cool isolation guarantees. I think there's a really compelling story right now about Tock isolating OpenThread stuff. Could be valuable for downstream users. I'm convinced that we're actually pretty close to being certifiable
 - Branden: My big thoughts for you: first, we'd need someone in charge of the push, not just the working group. That could be Tyler if he wanted to. Second, we'd need details on what the memberships would look like, what they would require, what they would give us. Also, we'd really want to low-stakes explore if Tock certification is possible and if Thread Group is even interested in that since we're kind-of weird.
 - Tyler: We do have some contacts for people involved here we could pull on
 - Branden: What about low-stakes communication first?
 - Tyler: Thread group is difficult to communicate with if you're not a member. We could join as a non-profit and attend some meetings to start. I think its more like a standards body like that. There's also separately OpenThread which is google's implementation of Thread, which is more open.
 - Branden: And to clear it up for me, this discussion is on Thread Group certification, and not OpenThread certification.
 - Tyler: Yes. Thread Group is the non-profit standards body in charge of the specification. OpenThread has their own test suite too, which I think is partially rolled into Thread certification. But you don't have to use OpenThread to be Thread certified, it's just easier since it's a known good quantity.
 - Rohan: Some details here: https://threadgroup.org/thread-group
 - Branden: We definitely would want to make sure we know what we're signing up for before doing anything. We want to make sure we'd get actual value out of any membership fee. We do have a contact at an IoT company that works with Thread Group, so we could talk to them about it.


