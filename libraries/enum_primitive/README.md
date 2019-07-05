enum_primitive
==============

This library is a lightly modified clone of
[enum_primitive-rs](https://github.com/andersk/enum_primitive-rs), adapted to
support `no_std`.

For complete documentation please visit
https://andersk.github.io/enum_primitive-rs/enum_primitive/


Status / Why a Copy?
--------------------

Due to auditability concerns, Tock currently has a policy against linking in
external code. For a simpler library, the approach taken here was to audit a
snapshot and include it directly.
