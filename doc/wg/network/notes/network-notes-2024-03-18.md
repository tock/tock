# Tock Network WG Meeting Notes

- **Date:** March 18, 2024
- **Participants:**
    - Alex Radovici
    - Tyler Potyondy
    - Branden Ghena
    - Amit Levy
- **Agenda**
    1. Updates
    2. OpenThread
- **References:**
    - [3923](https://github.com/tock/tock/issues/3923)


## Updates
- Alex: Leon and Amalia working on buffer management. Writing and re-writing code. Constant expressions would be really really helpful here. Overall, we're still figuring out how to best use the buffer management system, and also how to design it. It is progressing!


## Tutorial
- Tyler: Has there been movement on tutorial involvement?
- Alex: It's moving forward with information soon.
- Tyler: I need names by March 30th for the conference


## OpenThread
### Channel switching design
- Tyler: For channel switching, are we okay to just allow any process to change the channel?
- Branden: If you loaded multiple processes, 15.4 stuff would break anyways, right?
- Tyler: Likely? Not entirely sure, but I wouldn't trust it
- Branden: Then having just one process able to touch the radio seems fine. This could be handled maturely with syscall filtering on a per-app basis.
### Other stuff
- Tyler: Channel switching and RSSI PRs will be coming soon.
- Tyler: There's also a 6lowpan bug that I'm working on. https://github.com/tock/tock/issues/3923
- Tyler: Libtock-c PR coming soon as well. There's a big change to the build system to support OpenThread. Currently it fails CI and it's unclear why.
- Tyler: All other PRs are taken care of right now
### Tutorial planning
- Tyler: Discussions will be coming in the Slack channel. Not yet at the point where people should test anything, but I expect that this will come at some point.
### Future 15.4 changes
- Tyler: I do think there are some capsule changes we should make. Encryption, for example, the process seems strange right now. There are more stream-lined ways to do this. And I think we could focus on doing stuff in userland, which I think researchers would want.
- Tyler: It would be great for the capsule to have some features, and then userland to be able to make choices and implementations that can modify how things actually work.
- Tyler: 15.4 has gone through several implementations already. I think some of the work just needs to get rid of deprecated functionality too. Streamlining for usability and readability.
- Branden: I do think that a focus on users is good. Who are they and what do they actually want.

