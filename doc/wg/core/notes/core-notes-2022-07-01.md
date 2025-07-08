# Tock Core Notes 2022-06-10

Attendees:
- Hudson Ayers
- Alyssa Haroldsen
- Alexandru Radovici
- Vadim Sukhomlinov
- Johnathan Van Why
- Brad Campbell
- Leon Schuermann

# Updates

- Alyssa: Found a bug that generated 3k of size savings for us! All we had to do
  was change the representation of ErrorCode to be u32 instead of u16.

- Alyssa: The problem is that the Rust ABI for risc-v makes the callee
  responsible for ensuring the top bits of a register are cleared. So the
  compiler inserts assembly instructions to set the top bits of a u16 to be 0 in
  any function that accepts a type which is repr(u16).

- Johnathan: I also wrote some inefficient code for converting error codes in
  libtock platform.

- Alyssa: I think the problem I described is the larger one. Here is a godbolt
  sample: https://godbolt.org/z/rxKqjqWfd 

- Alyssa: This is theoretically a breaking change for libtock-rs but it should
  lead to compiler errors for any users.

- Johnathan: I think that change should be fine to submit upstream for the size
  savings. Libtock-rs is not technically stable. I understand some theoretical
  concern about unsafe uses of the type but in practice I do not expect
  problems. But keep in mind that no Tock kernel version has been released which
  libtock-rs targets

- Alyssa: Does that mean Ti50 will have trouble updating libtock-rs?

- Johnathan: Only if you are not on a kernel after the PR that added the allow
  swapping protections.

- Hudson: That is a good reason that we should attempt to release 2.1 sooner
  rather than later, since libtock-rs is pretty usable now.

- Brad: I have a student who is making progress on dynamic app loading. Current
  status is he has an app which has a binary of the blink tbf inside it, the app
  passes it to the kernel and the kernel loads it into the process array and
  runs it without resetting the kernel. Hope is a PR soon.

- Leon: That is awesome, I know Dorota would love to use that.

- Hudson: Recommend people book hotels for TockWorld.

- Brad: Alistair has booked his trip for TockWorld!

# AppID PRs

- Hudson: I think we should all take a look at Phil's AppID PRs to the Tock
  kernel and elf2tab / tockloader. Let's take 10 minutes and read through the PR
  description and look at the general layout of the code and see if we have any
  high level comments for Phil. I think he would love detailed reviews when
  people get the chance.

- all: 10 minutes to read

- Hudson: One thing I concerned about is this breaking change for updating the
  value of the `init_fn_offset` field to reflect what the TBF documentation says
  it should mean. I don't think we have a great way to support that type of
  breaking change in a way that is not gonna be pretty poor from a user
  experience perspective. I was wondering whether we could just change the
  documentation instead.

- Alyssa: Most of my concerns surround code size / RAM overhead. Specifically
  the impact per-app. We verify our entire firmware image at once so for us
  validating apps dynamically is not a business need. Wondering whether this
  could be an optional feature.

- Johnathan: Well this does not require any cryptographic implementation, it is
  optional for the kernel

- Johnathan: Either way you need an ID mechanism for apps for security, even if
  you verify images externally. How would you do IPC securely?

- Alyssa: Well we use a capsule for that where apps subscribe to dispatcher
  channels, and name targets to send to.

- Leon: As far as I know if you do not include footers and do not supply a
  verifier this should be pretty minimal overhead. This mechanism is optional!

- Alyssa: The Process struct adds fields, it is over 1kB in size already,
  per-process.

- Leon: This adds just two words to the struct. Those are references to the
  location in memory of those footers.

- Alyssa: Oh so those fields are just raw pointers?

- Leon: They are actually references but I think they should be pointers looking
  from a safety perspective.

- Alyssa: Ok so long as it is not a significant code size or RAM impact I am
  happy but I think we should have a test bed set of applications to test things
  like code size impact. Do we have that?

- Hudson: Not completely -- for most boards the kernel is built with an
  assumption of 4 processes and we have github workflows that track the size
  impact of a given change. And for this particular change I don't think there
  should be any change to the size of apps themselves if they choose not to add
  these footers.

- Alyssa: So what are the results of that workflow for this PR?

- Leon: It looks like Phil has updated all boards to actually use this mechanism
  now, so we could not easily see what the overhead is for someone opting out.

- Alyssa: How do you turn this on/off? Is it a cargo feature?

- Leon: You use `load_processes` instead of `load_and_check_processes`.

- Hudson: I think this is a fair concern, we definitely need to benchmark this
  before we merge it, and we should probably have some of the example boards
  upstream not use the feature so we can ensure that option continues to work.

- Hudson: It looks like this feature adds about 2.5 kB for all of the upstream
  boards now, so that should be an upper bound on the overhead. That is true for
  both Hail which has 20 apps and Imix which has 4.

- Brad: Why would the number of processes in the process array matter?

- Hudson: I guess it shouldn't

- Alyssa: It can, in isolated cases.

- Johnathan: 2600 bytes is a pretty large increase. Does this include a crypto
  implementation?

- Hudson: Yes, I think so

- Johnathan: Oh, then that makes sense.

- Hudson: Yeah we need to be sure if you don't use this you don't pay the cost
  of the SHA implementation.

- Brad: Can we turn off the feature for a board real quick and see the overhead?

- Hudson: (tried to do this, did not have luck as it is a little more involved
  than I thought).

- Leon: I think we need to get rid of some dynamic dispatch in this
  implementation in favor of generics so that dead code elimination can work
  properly for this code.

- Alyssa: Maybe I should try to upstream something like our current dispatcher
  implementation -- it does provide pretty simple and efficient IPC

- Hudson: We are definitely interested in improved IPC

- Brad: Seconded

- Leon: Improved and safe IPC!

- Alyssa: I do not think ours even includes any unsafe

- Johnathan: If we want to add IPC that does not use AppID we will need to
  change the threat model. So that apps cannot impersonate each other. TicKV
  also is not compliant with this threat model because there is no verification
  that apps are who they say they are.

- Alyssa: Yeah I see. If you want to establish trust on top of IPC I understand
  that. You could have a list in the kernel of what channels apps can subscribe
  to.

- Johnathan: I think we should meet one-on-one and discuss this.

- Alyssa: yeah

- Leon: I think the purpose of this PR is a one-stop-shop for identifying and
  authenticating an app within the Tock ecosystem. Ideally this would be a
  unified thing.

- Johnathan: Well, storage is not included, but we decided that intentionally.
  At least for dynamically loaded apps.
