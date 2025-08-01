sha256sum
=========

Tool for computing a SHA-256 hash of a file.

In general we use the host's `sha256sum`, but if one doesn't exist we fallback
to this to avoid an error and another dependency.
