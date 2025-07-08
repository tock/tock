# Tock Meeting Notes 2024-06-14

## Attendees

- Hudson Ayers
- Phil Levis
- Amit Levy
- Brad Campbell
- Leon Schuermann
- Branden Ghena
- Alyssa Haroldsen
- Johnathan Van Why
- Pat Pannuto


## Updates

- Amit: I have been looking back at IPC because some people are actually using it
- Amit: I am going to try to fix some issues with discovery that are dependent on
  app ordering
- Amit: There are bigger problems with IPC, we are using it for open
thread where it has been problematic, this is a problem for user space services
in general.
- Amit: pretty big design tradeoff space here, depends on what we
might think are the actual meaningful use cases for IPC in Tock. Mostly a heads
up that I will soon start soliciting feedback on this.
- Branden: are you considering message passing?
- Amit: Anything is on the table, but the use case I
am referencing involves sharing 16kB buffers. It is possible there could be two
separate interfaces here (shared memory + message passing)
- Phil: Finished up
class on rusty systems. Asked students for their favorite paper, they said
Hudson’s code size paper.
- Amit: I have been looking through how we are doing in
code size, and we seem to have gotten some good gains from the last few Rust
updates. On the nrf52840dk about 15kB.
- Amit: I realized by accident that
changing some cargo options can save a few additional kB on the same board.
- Amit: We reposted a tweedegolf blog about identifying size bloat in Tock
binaries.
- Amit: The authors of that post are leading the charge in upstream Rust
to reduce the size of core. For example they added a flag to the core library
to push size optimization into the Rust compiler rather than LLVM.
- Pat: TockWorld is 2 weeks away! Less than half the people on the agenda have
registered, please do so.
- Alyssa: I will be attending TockWorld!
- Pat: I will extend the early-bird registration to Monday

## Tock Storage Permissions (PR #4021)
** time spent reading PR + discussion **
- Phil: I think Alistair’s FIDOv1 / FIDOv2 use case
can be solved in other ways that are not yet discussed on the PR.
- Amit: Without essentially applications being able to declare themselves what they can read
and write to, how would storage for FIDOv1 know to have the FIDOv2 app shortID
in the readable permissions set?
- Johnathan: You have to push a new version of
FIDOv1 that includes that permission. Otherwise there is a fundamental threat
model violation
- Amit: I agree
- Brad: This TRD is not about how permissions are assigned
- Brad: I think Alistair would say that Fidov1 can use data that FIDOv2
wrote and it will just work and understand it
- Phil: So Fidov2 would change the keys under FIDOv1?
- Brad: yes
- Phil: that sounds like a terrible idea, FIdov1
didn’t allow it
- Alyssa: The shortID could maybe be mixed with a secret?
- Brad: I have a branch with the implementation of this which informed the TRD. That
hasn’t really changed the policy but has changed how storage permissions are
implemented in the kernel.
- Amit: I think we should vote to either say that we do
not need this TRD, or we approve it as is, or that we say there are specific
things that need to be changed before merging, or that we really do not know
and what specifically would need more discussion.
- Amit: Does anyone have comments before we vote?
- Johnathan: An AppId is how an app gets access to its
secrets. We do not want to assume some trusted registry or secrecy of app
binaries. I don’t think there is a good way to allow access to another app
without an explicit permission without one of these things. To add access after
the fact you should update the original app.
- Johnathan: One thing that remains
unclear to me are the uniqueness guarantees of ShortIDs. My intent was that
ShortIDs must be unique across running processes and consistent across all
running instances of an app on Tock systems.
- Johnathan: 2 years ago we
effectively created a requirement to have nonvolatile storage accessible by any
implementation of AppId / ShortID that meets the requirements
- Amit: I mean,
alternatively a customer could have a registry
- Johnathan: I think if we have
the property that no two AppIds can map to the same ShortID, and shortIDs are
consistent, they are 1:1 bijective with AppIds. Then the policy that assigns
the ShortIDs determines security policy, but that could be a board level thing,
but consumers of ShortIDs do not need to care.
- Pat: What is the scope of these
ShortIDs?
- Johnathan: I think the scope should be specific to the “Tock system”,
which we have to define.
- Johnathan: For example, if you do not have nonvolatile
storage, every reboot defines a new Tock system.
- Brad: The goal of ShortID was
to not make requirements that individual orgs might not want, and instead be an
adaptable framework for use by those organizations.
- Pat: I think I am coming
around here. What might help is being more explicit that StorageIDs are
deriving from AppIDs and the policies around their uniqueness/ownership will
depend on the board configuration. Someone just reading the storageID document
may be missing some of this context about AppIDs than just a link to the other
doc.
- Brad: I see that.
- Brad: Most of these comments are about AppID
- Amit: and the legitimacy of using ShortIDs in this case. My claim is that the assumption
the kernel should be allowed to make about ShortIDs is that they are a 1:1
mapping with AppIDs. That assumption may not be true in some cases and it is
not great if that is the case, in a not so different way than say hash
collisions.
- Johnathan: To legitimize the use of hashes, if you trust no-one is
trying to create hash collisions, that is good enough for security
- Amit: A not bogus way to generate shortIDs would be to take a cryptographic hash of AppIDs
and take the lower 32 bits
- Johnathan: That might make people think we are more
secure than we are.
- Brad: Do we want that policy to be the same one that we use
to create ShortIDs, or do we want another WriteID policy?
- Amit: To my mind, having another kind of ID does not fix this, and there is a significant benefit
to using a single ID instead of many, and I still don’t see a compelling reason
not to do so. This seems like the right design to me.
- Johnathan: StorageIDs have to either be based on AppIds or be as complex, and we can’t afford
duplicates of something so complex in embedded.
- Amit: let’s vote!
- *vote in chat*
- Approved by: Brad, Amit, Branden, Johnathan, Hudson
- Abstained: Alyssa, Leon
- Pat: approve with no changes to policy, but request for explicit
statements to the relation between ShortID and AppId.
- Pat: I will summarize this in a comment on the TRD PR


## Yield WaitFor

- Brad: I updated the PR, renamed some terminology and types , tried to remove
the iterative thing that happened with naming. I also updated the TRD. I think
the implementation is good enough to evaluate this.
- Amit: Lets turn this into a
non-draft PR and properly review it
- Brad: OK, I will do that.
- Brad: The one meaningful change is there was confusion about which yield parameter becomes
the upcall ID, and I smoothed that out. Was mostly moving text around.
- Hudson: Do we want a release before or after Yield WaitFor?
- Brad: Ideally we would do a release right before and right after, but I don’t think we have the
motivation.
