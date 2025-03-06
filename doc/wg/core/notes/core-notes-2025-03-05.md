# Tock Meeting Notes 2025-03-05

## Attendees

- Branden Ghena
- Brad Campbell
- Pat Pannuto
- Johnathan Van Why
- Amit Levy
- Viswajith
- Alexandru Radovici
- Kat Fox
- Leon Schuermann
- Hudson Ayers
- Tyler Potyondy

## Updates

- None

## Dynamic Process Loading

- TRD: https://github.com/tock/tock/pull/4338
- Adding comments in PR
  - Should dynamic store capsule match trait name?
  - Where are resources allocated, e.g. from app_loader capsule?
- Need to maintain the linked list structure of processes.
  - Need to call: setup(), then write() multiple times, then either abort() or
    load()
  - Should there be a "finalize" step that writes the total_length field to make
    the app valid?
  - What are the guarantees for maintaining the linked list and how is that
    enforced?
  - The requirements for linked list maintenance leaks into the interfaces.
    - We don't know how to build better abstractions right now that hide the
      linked list details.
  - A "finalize" operation might make it simpler to reason to about the state of
    flash.
- The store() and load() are separate traits, yet seem very linked.
  - They are very much linked. Expectation is that there is one app in flight
    being loaded.
  - Perhaps clearer documentation about setup() and load() is needed.
- Why is setup() separate from set_storage_client()?
  - The set_client() is just like any other set_client() in Tock. Multiple apps
    would need a virtualizer layer to handle multiple app-level clients.
- Why are these kernel capsules? Would capablities be a better solution?
  - Maybe. But where to store a process is quite sensitive.
  - The intent of `DynamicBinaryStore` and `DynamicProcessLoad` is to be generic
    to different backing storage mechanisms.
