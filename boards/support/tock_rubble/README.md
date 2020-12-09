Tock-Rubble Interface
=====================

This crate provides the glue logic between the Tock kernel and the external
[Rubble BLE stack](https://github.com/jonas-schievink/rubble).

Since Tock generally prohibits external dependencies, we do not include Rubble
directly in the Tock kernel. Instead, Tock includes the necessary interfaces
(specified in `/kernel/hil/rubble`) to use Rubble, but the dependency is only
included in this crate. This enables boards to selectively use Rubble (and hence
include the dependency), while not including the dependency for boards that do
not use Rubble.
