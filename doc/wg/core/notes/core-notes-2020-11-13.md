# Tock Core Notes 2020-11-13

## Attending
 * Branden Ghena
 * Alistair
 * Leon Schuermann
 * Amit Levy
 * Johnathan Van Why
 * Philip Levis
 * Brad Campbell
 * Pat Pannuto
 * Vadim Sukhomlinov
 * Hudson Ayers

## Updates

- Brad: Merging Hudson's switch to board-based instantiations.
- Hudson: Only MSP432 and new board to do.

- Johnathan: Discussed with Alistair about AppID, may be a simpler
  way to implement it. Need to verify it would work with OpenTitan
  use case.

- Hudson: Board based chip instantiation significant reduce static mut in
  kernel.



## Anti DOS Storage

- Johnathan: Transferring storage between apps can difficult and require
  specific order for app update and installs.
- https://mailman.stanford.edu/pipermail/helena-project/Week-of-Mon-20201102/001116.html

- Amit: Is this filesystem-specific, or a more general interface design case?
- Johnathan: This could come up if an app wanted to transfer its permissions to
  a new app. But seems most likely to be an issue with storage.

- Hudson: Does this require an app to be loaded without restarting?
- Johnathan: No, I was thinking the new app could be loaded and then a restart
  could happen.

- Amit: Filesystem fills up: resource allocation problem.
- Stealing file names: unix solves with directories.
- Transferring files notion is...odd.
- Johnathan: Transferring files comes from the appid discussion, I don't know
  the exact use case.
- Alistair: Transferring files allows for splitting apps, or upgrading apps.

- Alistair: having different read IDs and write IDs and enforcing access like
  that would work.

- Phil: Idea is over time want flexibility in software. Might want to use
  security credentials in old app in a new one.

- Leon: Flexibility could be mapped on to existing AppID proposal. ACLs could be
  updated to allow an app to access any file.
- Alistair: That would require the kernel to be updated?
- Leon: ACLs could be in TBF headers.
- Alistair: TBF headers aren't trusted.
- Leon: Apps can't change their TBFs.
- Alistair: If an app could get loaded and set its own ACLs.

- Amit: What scenario makes this difficult?
- Johnathan: If all new apps are created, difficult to transfer files between
  old versions of the app and new versions of the app.
- Amit: Could new app have permission to access old files?
- Johnathan: In simple cases probably ok, gets complicated to keep this straight
  if apps change multiple times.

- Pat: This migration process doesn't happen often, can we just have this be a
  complex migration tool that we run very rarely?
- Johnathan: Agreed.

- Amit: Is this solved by groups in unix?
- Leon: I think the model of an app has access to a folder based on its group
  permissions.
- Johnathan: Can do this with signatures, but those get long.
- Amit: can put apps in groups, permissions based on groups.
- Have a groups file.
- Johnathan: Like the permission list model Alistair proposed.

- Leon: Do we need so much flexibility? Why not allow apps to have access to
  specific file names. New apps could have access to the same file names.
- Johnathan: Would need to trust this access list.
- Leon: I'm signing apps, and kernel won't load apps if signature not trusted.
- Johnathan: Won't work for OT.

- Phil: Alternative idea: use IPC to transfer data between apps, new app
  receives data via IPC.
- Leon: Problem with long files.

- Amit: Need more concrete threats and use cases. See if this has been solved
  before.


## 2.0

- https://mailman.stanford.edu/pipermail/helena-project/Week-of-Mon-20201102/001118.html

- Leon: Unallow semantics. App cannot allow an alias of existing buffer
  (duplicate references). How to enforce?
- Idea: have static table in kernel per app that records each allow. Can check
  this table to verify allow.
- However, static issues: could run out of slots.
- Alternative: multi layer structure.

- Branden: Is it ok if we have aliased buffers since the memory is in apps?
- Leon: Still UB.

- Brad: What about using grant-like mechanism?
- Phil: Fragmentation issues.

- Branden: Use a dynamic linked list.
- Leon: Still fragmentation issue.

- Amit: We can try different options. Try more exotic approach and fallback to
  static table.


## PR #2156

- K,V storage system.

- Brad: Is it still blocking?
- Alistair: Yes
- Phil: Can't block because it could be off-chip.
