Tock Components
===============

Components in Tock are helper files that simplify initializing a board with
various capsules and other resources.

Motivation
----------

Initializing a board mainly consists of three steps:

1. Setting any MCU-specific configurations necessary for the MCU to operate
   correctly.
2. Statically declaring memory for various kernel resources (i.e. capsules) and
   configuring the capsules correctly.
3. Loading processes, configuring the core kernel, and starting the kernel.

Components are designed to simplify the second step (configuring capsules) while
also reducing the chance for misconfiguration or other setup errors.

Configuring capsules can be tricky because i) the exact type needs to be
specified, and types can be complicated and tedious to define correctly, ii)
capsules can be complex and require several arguments or setup steps, and iii)
capsules often require `set_client()` to be called to setup in-kernel callback
chains, and these can be easy to forget. Components allow most of a capsule's
configuration to be written just once in a component, and then various boards
can use the component to reduce the complexity of their setup code and reduce
the chance for errors. Components also reduce the burden when changes are made
to capsules, as the change can likely be reflected in the single component and
not in every board's main.rs file.

Adding Components
-----------------

We are always happy to merge new components for various capsules or different
configurations. Generally copying an existing component is the best place to
start when creating a new component.
