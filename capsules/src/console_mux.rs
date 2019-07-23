//! `ConsoleMux` enables multiple interactive consoles to share a single UART.
//!
//! ## Overview
//!
//! Interactive consoles can be very useful, for example for allowing developers
//! to inspect the state of the system (like using `ProcessConsole`) or for
//! users to interact with userspace applications (like the number guessing
//! game). However, sharing a single UART channel among multiple of these
//! interactive consoles leads to some confusing results since each console is
//! unaware of the others.
//!
//! The general structure at a high level looks like the following, where a user
//! wants to interact with a Tock board (likely connected over USB), and that
//! board may be running multiple console applications:
//!
//! ```
//!                      +---------------------+
//!                      | Tock Board Consoles |
//!                      |                     |
//! +------+             | +---------------+   |
//! |      |             | |Process Control|   |
//! | User +-------------+ +---------------+   |
//! |      |  UART over  | +---------------+   |
//! +------+     USB     | |802.15.4 Status|   |
//! tockloader           | +---------------+   |
//!                      | +------+            |
//!                      | | Apps |            |
//!                      | +------+            |
//!                      +---------------------+
//!```
//!
//! `ConsoleMux` defines and uses an explicit structure to enable sharing a
//! single UART channel among multiple consoles in a (hopefully) user friendly
//! way. The basic idea is that instead of sending raw messages over the UART
//! channel, each message is prepended with a known header. This enables both
//! sides (i.e. the Tock kernel and tockloader) to understand which console
//! messages originate from or are destined to. `ConsoleMux` is responsible for
//! understanding and prepending the headers for the Tock kernel. All messages
//! sent and received by the Tock board go through `ConsoleMux` which adds the
//! header or dispatches messages appropriately.
//!
//! ## Console Subsystem Structure
//!
//! The expected structure looks like the following:
//!
//! ```
//! +----------------+     +---------+
//! |                |     |         |
//! | ProcessConsole |     | Console |
//! |                |     |         |
//! +-------+--------+     +----+----+
//!         |                   |
//!         +-------+    +------+ Console
//!                 |    |
//!             +---+----+----
//!             |            |
//!             | ConsoleMux |
//!             |            |
//!             +------+-----+
//!                    |
//!                    | UartData
//!                    |
//!                +---+---+
//!                |       |
//!                | UART  |
//!                |       |
//!                +-------+
//! ```
//!
//! The `ConsoleMux` sits above the UART and uses the `UartData` interface from
//! the UART HIL. Above that each console uses the `Console` interface to
//! interact through the `ConsoleMux` to the underlying communication channel.
//! The `Console` interface abstracts low-level details that may be present in
//! the UART stream (like users hitting the backspace key) and provides
//! individual commands to all of the consoles. The `Console` interface also
//! allows consoles to send information back to the channel to display to users.
//!
//! ## Console Packet Structure
//!
//! To enable multiple consoles to share the same UART channel a packet
//! structure provides information about the source or destination of each UART
//! message. Specifically, the user side when sending a message must specify
//! which receiver should handle the message, and the Tock side when sending a
//! message must specify which console is transmitting the message. Since we
//! assume only one entity (the user) exists on the user side only one message
//! identifier is needed.
//!
//! The specific packet structure is as follows:
//!
//! ```
//! 0    2  3
//! +----------------------------------------------+
//! |len |id|data...                               |
//! +----------------------------------------------+
//! ```
//!
//! The first two bytes are the big endian length of the remainder of the
//! message. The `id` is a one byte identifier that specifies the source or
//! destination of the message, based on which way it is going over the UART
//! channel (if user->tock then it is the destination, if tock->user it is the
//! source id). After that is up to 65534 bytes of data payload (note that the
//! console mux or specific console being interacted with might have a shorter
//! maximum payload length).
//!
//! ## `id` Allocation
//!
//! The `id` field determines which console sent a message or which console the
//! message is intended for. In general, the `ConsoleMux` is free to assign
//! ids however it wants, but there are some reserved ids and rules to make
//! this protocol easier to use for tockloader (or other user-facing tools).
//!
//! ```
//! id #    | Reserved For
//! ------------------------------------------------------------------------
//! 0       | Control messages between the user facing tool and `ConsoleMux`
//! 128-255 | Applications
//! ```
//!
//! The ID of 0 is reserved to allow the user-facing tool (like tockloader) to
//! communicate directly with the `ConsoleMux` on the Tock board. This may be
//! used for a range of things, but a primary use case is to allow `ConsoleMux`
//! to provide a mapping of IDs to actual consoles so that the user-facing tool
//! can help the user select which console to interact with.
//!
//! ## Endpoint Responsibilities
//!
//! The user-facing tool must handle user input and prepend the correct header
//! structure on all messages sent over the UART channel to `ConsoleMux`. The
//! tool must select which ID to send to. Likely this would happen by asking the
//! user to select, but console-specific tools are also possible. When receiving
//! a message, the tool must process the header and remove the header bytes
//! before displaying the message to the user.
//!
//! `ConsoleMux` must prepend the correct header on all messages transmitted
//! from the various consoles. Upon receiving messages, it must inspect the
//! header and dispatch the message to the correct console.
//!
//! Because an unknown number of applications may be running on the board, and
//! the number of applications may change as the kernel executes, IDs greater
//! than 127 are reserved for applications. Applications interact with
//! `ConsoleMux` through the `Console` capsule, the `Console` capsule is
//! considered special and is automatically allocated half of the valid IDs.
//!
//!
