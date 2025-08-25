Some notes for Leon on `PacketBuffer`.

General notes:
* What is the "true" external PacketBuffer interface? I guess it's the `PacketBufferMut` impl?? It would be really nice if this file made clear what the externally-usable calls are and what the internal crap is. I guess that's the `pub` keyword? But the impl of `PacketSliceMut` is `pub` too...
* This file will make clear what the internal implementation of PacketBuffer is and how that works. It doesn't, however, contain justification for _why_ it's like that. I think a lot of comments for functions and structs that explain the "why" would be valuable.
* This file also doesn't have documentation on how to use it practically, especially with the generic parameters. Maybe a separate file with toy example that uses it would be valuable there so people can see the generics in action but without all the other stuff that comes with, say the Console redesign. Something minimal would go down through at least two layers and then talk to some chip driver at the bottom. Then go back up through those same two layers.

Starting in `PacketBufferDyn`:
* For `reclaim_headroom`, the comment could be more clear here. I think it's taking space from the buffer and giving it to the headroom? Maybe a quick diagram. So false would mean that there wasn't enough buffer space?
* For `reclaim_tailroom`, it will presumably not move past the headroom marker.
* For `reset`, what does it do to tailroom? It's unclear to me how reset is different from `reclaim_headroom`
* For `copy_from_slice_or_err` and `append_from_slice_max`, those apply to the buffer data itself, right? They could use comments. I don't really understand what they do from the names alone. I'm also not sure why these two functions exist? What makes them the "proper" interface?
* Why do we even need a `prepend_unchecked` operation? I guess the idea is that the checks are already happening at compile time before calling this?

In `impl PacketBufferMut`:
* For `reset`, that's a runtime assert, right? Is that necessary?

For `PacketSliceMut`:
* A diagram here would be really nice. What values are prepended, and how do headroom, payload, and tailroom work?
* Also a comment that they're stored with native endianness, whatever that may be
* I don't at all understand why `_inner` is a `u8` here. And the comment above it does nothing to help that. It seems to never be used because we just refer to `self` instead. I kind of thought it would be `[u8]` since it's a transmuted array of data...?
* In `new`, the comments don't seem to match the code about starting with zero headroom. I think it starts with payload length, headroom bytes of headroom, and length - headroom bytes of tailroom.
* Have you looked at the assembly for `get_inner_slice_length()`? It seems like it should be pretty optimized, but I'm wondering if it actually happens in practice. Also that the panic isn't there.
* I don't like that `data_slice` and `data_slice_mut` have slightly different constructions

In `impl PacketBufferDyn for PacketSliceMut`:
* Is the implementation of `len()` correct? The comment there looks plausible, but the implementation looks wrong.
* Why does `copy_from_slice_or_err` eat up tailroom? I assumed it would write into the payload and error if it didn't fit in the payload (slice.len() - headroom - tailroom).


