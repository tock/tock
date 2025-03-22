# Unsafe Rationale

Segger specifies the in-memory representation of its communication channels,
which is comprised of a raw pointer and a length for buffers. These buffers
also must be treated as volatile, since the debugger may arbitrarily write
into the (down) channel.

While this could avoid `unsafe` using an array of `VolatileCells`, where the
driver also holds the original slice of `VolatileCells`, this now requires
reasoning about two views of memory as well as comparatively inefficient
memory access.

Instead, this uses `unsafe` to create a slice-like abstraction that adheres to
the externally imposed Segger requirements.
