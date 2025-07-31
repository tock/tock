# Tock Meeting Notes 2025-06-25

## Attendees

- Amit Levy
- Brad Campbell
- Leon Schuermann
- Johnathan Van Why
- Tyler Potyondy
- Hudson Ayers
- Pat Pannuto


## Updates

- Tyler: Update on AES. Things not working in the stack. Bitrot over time.
  Working to clean it up and make it compatible with rust-crypto.
  - Working on PR by end of next week (July 4, 2025).
  - Amit: would be good to see if these are compatible with other AES users.
    Often other users want APIs that more directly match hardware. But would be
    interesting to see if anything has changed and things could match better.


## PR Discussion and Moderation

- Often relevant to PRs where the merit/need of the PR isn't always clear, _but_
  there is still some value.
- Symptom: discussion on the PR is not productive, eg. dismissive or debate-y.
- Symptom: PR not to the point where we are ready to merge.

- Issue: we don't have a document specifying how to engage or what our
  expectations are.
  - We do have the Rust CoC.
  	- Links to the node.js document on trolls.
  - We have been hesitant to 1) write down what feel like opinions rather than
    objective decisions based on data, 2) haven't wanted to legislate specific
    rules which could lead to nitpicking details.

- Interactions with outside contributors can be more challenging
  - For example, issues with nix community, moderators being dismissive towards
    outside contributors.

- It's going to be hard to write down exactly what we want with respect to
  interaction
  - Maybe having the document and being able to point to it, even if the wording
    is not perfect, would be a way to make it easier to discuss in PRs.

- There are two issue: engagements and expectation for PRs
  - Those could be separate documents
  - We could try to write down our expectations for PR quality given:
    1. Tock's goals
    2. Tock's users
    3. Our experience with different subsystems
    4. Our expectations for different subsystems

- Many projects use bots to close stale PRs
  - Not a person closing the PR
  - Idea: use a PR label as a "warning" for a PR that will be closed soon-ish
    for staleness.
    - Could be called `Changes Requested` denoting that changes are needed for
      the PR to be eventually merged.
      - Might be hard to apply this for our more contentious PRs.

- **Action Items**:
  - Amit to draft two documents


## IPC

- Many discussions about redesigning IPC, both at 2025 workshop and in other
  discussions.
- IPC redesign semi-assigned to the Networking WG.

- This is a task where:
  1. A bunch of people want it
  2. No specific person has the engineering time to work on it

- Questions:
  1. We never actually said the Networking WG formally owns it, should we?
     - Amit: I think yes, as the members are most related to using it.
     - But, is there engineering time?
  2. What is the path for forward progress?
     - Is it sketching out a design, or doing the implementation too?
       - Yes, could be an RFC. Maybe there is a close implementation that could
         be updated to match the RPC.
       - That could help with scavenging various time to work on this.

- There is a project for this: https://github.com/orgs/tock/projects/5
