# Tock Network WG Meeting Notes

- **Date:** September 14, 2026
- **Participants:**
    - Branden Ghena
    - Leon Schuermann
- **Agenda:**
    1. Updates
    2. IPC
- **References:**
    - [IPC Reference Document](https://github.com/tock/tock/pull/5016)


## Updates
- Various discussions on Treadmill and Tock Release

## IPC
* https://github.com/tock/tock/pull/5016#discussion_r3969495277
* https://github.com/tock/tock/pull/5016#discussion_r3960789951
* Branden: Background: Brad took a look at the IPC documentation and was getting confused on Services versus Processes, which is totally a great insight. We've been mixing them up, which is a fixable documentation problem. But also we haven't put thought into how to handle this, which is an architecture/design question we should address
* Branden: Network analogy: IPC is like IP. It gives the address of a process, but services are a higher-level thing analogous to Ports. In this analogy, Registration is like DNS, it connects plaintext names to IpcIds.
* Branden: So Port could be layered on top of IPC communication channels
* Leon: IPC is too ill-defined for us to anticipate all use cases
* Leon: Announcing multiple services from a single process sounds like something that should be outside of scope of things the kernel is aware
* Branden: Two problems discovery and filtering
* Leon: I think these are the same problem. Discovery has multiple identifiers and then filters can allow/deny each of these individually
* Leon: From a system integration perspective, clients talking to services implies there's a high-level design where a developer of a system knows that there are clients/services available. They may or may not known which application is which service/client. But there should be understanding of which services possibly exist. So maybe the list of possible services exists in the board file.
* Branden: I think that's totally reasonable. Possible to do as a separate capsule.
* Leon: There's a use case where we don't care which service names exist, and those strings could be stored dynamically (ignore implementation for now). Clients can discover them.
* Leon: But if we're doing filtering at all, then the board should know which service names exist at all. It's static.
* Branden: I think you can combine both worlds by just installing both registries on the board.
* Branden: So, how do we do registry(s) that enable multiple services per process:
  * Leon: TBF-header array-based registry, rather than package name itself.
  * Branden: Fixed-list provided by board that Leon mentioned. Which associates strings with IpcIds upon registration
  * Branden: String Name with const generic fixed number of strings per process
  * Branden: String Name with dynamic grant allocation of strings per process
* Branden: Okay, how important is implementing some of these?
* Leon: Most important thing is what an interface looks like from userspace. As far as real use cases and complexity goes, static strings version is most useful.
* Leon: Board would provide array of tuples, first element is the string and second element is some opaque, default-able type, which could be passed back into the filter for it to make a decision.

