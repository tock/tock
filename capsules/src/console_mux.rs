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


use core::cell::Cell;
use core::cmp;
use core::str;
use kernel::capabilities::ProcessManagementCapability;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::{List, ListLink, ListNode};
use kernel::debug;
use kernel::hil::uart;
use kernel::introspection::KernelInfo;
use kernel::Kernel;
use kernel::ReturnCode;

// Static buffer for transmitting data.
pub static mut WRITE_BUF: [u8; 512] = [0; 512];

// Static buffer for receiving data.
pub static mut READ_BUF: [u8; 512] = [0; 512];

// Buffer for handling commands sent to the `ConsoleMux` itself. These will likely
// only be short commands.
pub static mut COMMAND_BUF: [u8; 32] = [0; 32];



/// Main interface trait that consoles use to send and receive messages. The
/// buffers provided must not have any console mux header bytes.
pub trait Console {
	/// Function for a console to be able to send a message. It uses the
	/// standard buffer and length. The buffer should be only the
	/// console-specific data and should not contain any header information.
	///
	/// The last parameter is an optional application ID that should only be
	/// used by the app console because the app console is actually forwarding a
	/// message on behalf of an application. All other consoles should set this
	/// parameter to `None`.
	///
	/// The transmitter should not call this multiple times until the
	/// `transmitted_message()` callback has occurred.
    fn transmit_message(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
        app_id: Option<u8>,
    ) -> (ReturnCode, Option<&'static mut [u8]>);

    /// Setup a receive buffer for this particular console. Since there will be
    /// many consoles, this buffer will be held by the mux until a received
    /// message comes in for the particular console.
    fn receive_message(&self, rx_buffer: &'static mut [u8]) -> (ReturnCode, Option<&'static mut [u8]>);

    /// Provide a reference to the console client that will be called when
    /// messages come in or when transmissions have finished.
    fn set_client(&self, client: &'a ConsoleClient);
}

/// Callback interface for consoles. This is how consoles are signaled of new
/// messages and when transmissions are finished.
pub trait ConsoleClient {
	/// Called when a message has been sent for the particular client. This will
	/// return the static buffer back to the console.
	fn transmitted_message(&self, message: &'static mut [u8], tx_len: usize, rcode: ReturnCode);

	/// Called when a incoming message has been received for the particular
	/// client.
    fn received_message(
        &self,
        read_buf: &'static mut [u8],
        rx_len: usize,
        rcode: ReturnCode,
        error: uart::Error,
    );
}

/// State for each attached console to this `ConsoleMux`.
pub struct ConsoleMuxClient<'a> {
	/// A reference to the actual mux structure which is needed for certain
	/// operations in the implementation.
	mux: &'a ConsoleMux<'a>,

	/// The `id` is a simple identifier for this client console. It will be used
	/// when sending message to identify the sender, and used when receiving
	/// messages to route messages to the correct client.
    id: Cell<u8>,

    /// The reference to the actual client capsule.
    client: OptionalCell<&'a ConsoleClient<'a>>,

    /// Stored buffer for receiving messages. This will get passed in from the
    /// console and saved here until a message arrives for the user destined for
    /// that console.
    rx_buffer: TakeCell<'static, [u8]>,

    /// Place to hold a transmit buffer from this console. This is likely not
    /// necessary, but if multiple consoles transmit at the same time then we
    /// need somewhere to buffer the outgoing message.
    tx_buffer: TakeCell<'static, [u8]>,
    /// The length of the outgoing message.
    tx_buffer_len: Cell<usize>,
    /// The `tx_subid` is an additional identifier needed for the application console
    /// that corresponds to
    tx_subid: OptionalCell<u8>,
}

/// The base mux that enables sharing an underlying UART among multiple
/// consoles.
pub struct ConsoleMux<'a> {
	/// The underlying UART hardware for the communication channel.
    uart: &'a uart::UartData<'a>,

    /// List of all attached consoles. There is one special console which will
    /// have an id of 128 which is the console that manages all of the
    /// applications.
    consoles: List<'a, ConsoleMuxClient<'a>>,

    /// Current operating state of this console mux. This is mostly on the RX
    /// side.
    state: Cell<State>,

    /// Flag to mark the transmitter as busy, and to keep track of which buffer
    /// should be returned to which console. If this is `None`, then nothing is
    /// transmitting.
    active_transmitter: OptionalCell<u8>,

    /// Saved TX buffer that is actually passed to the UART.
    tx_buffer: TakeCell<'static, [u8]>,

    /// Saved RX buffer that most of the time is being held by the UART driver
    /// waiting for incoming messages.
    rx_buffer: TakeCell<'static, [u8]>,

    /// Saved command buffer that is populated when a message comes in for the
    /// `ConsoleMux` itself.
    command_buffer: TakeCell<'static, [u8]>,
}

/// The state of the mux, mostly handles transitioning in the receive case.
enum State {
	/// Haven't started, not currently sending or transmitting.
	Idle,

	/// We are waiting for the user side to send a valid message, and we are
	/// only listening for the header bytes of the message.
	WaitingHeader,

	/// The console mux has received the first three bytes of the message which
	/// is the header including the message length and the destination id. The
	/// `ConsoleMux` is now trying to receive the remainder of the message.
	ReceivedHeader { length: u16, id: u8 },
}

impl<'a> ConsoleMux<'a> {
    pub fn new(
        uart: &'a uart::UartData<'a>,
        tx_buffer: &'static mut [u8],
        rx_buffer: &'static mut [u8],
        cmd_buffer: &'static mut [u8]
    ) -> ConsoleMux<'a> {
        ConsoleMux {
            uart: uart,
            consoles: List::new(),
            state: Cell::new(State::Idle),
            active_transmitter: OptionalCell::empty(),
            tx_buffer: TakeCell::new(tx_buffer),
            rx_buffer: TakeCell::new(rx_buffer),
            command_buffer: TakeCell::new(cmd_buffer),
        }
    }

    /// Start the console mux by passing a receive buffer to the underlying UART
    /// device.
    pub fn start(&self) -> ReturnCode {
        if self.state.get() == State::Idle {
            self.rx_buffer.take().map(|buffer| {
                // self.rx_in_progress.set(true);
                self.uart.receive_buffer(buffer, 3);
                // self.running.set(true);
                self.state.set(State::WaitingHeader);
            });
        }
        ReturnCode::SUCCESS
    }

    /// Add a console client to the mux. This is for in-kernel consoles.
    crate fn register(&self, client: usize) {
    	// Determine the ID for this console.
    	let mut count = 1; // Start at 1 because 0 is a reserved index.
    	self.consoles.iter().for_each(|_| {
    	    count += 1;
    	});

    	client.id.set(count);
    	self.consoles.push_head(client);
    }

    /// Add a console client to the mux. This is for an app console.
    crate fn register_app_console(&self, client: usize) {
    	client.id.set(128);
    	self.consoles.push_head(client);
    }

    /// Process messages sent to the `ConsoleMux` itself.
    fn handle_internal_command(&self, length: usize) {
    	self.command_buffer.map(|command| {
            let cmd_str = str::from_utf8(&command[0..length]);
            match cmd_str {
                Ok(s) => {
                    let clean_str = s.trim();
                    if clean_str.starts_with("list") {
                        debug!("Consoles:");
                        debug!("console 1");
                    }
                }
                Err(_e) => debug!("Invalid command: {:?}", command),
            }
        });
    }

    /// Check if there are any consoles trying to send messages. If not, just
    /// return and this will get called again when a console tries to send.
    fn transmit(&self) {
    	if self.active_transmitter.is_none() {
    		self.tx_buffer.take().map(|console_mux_tx_buffer| {

		    	self.consoles.iter().for_each(|client| {
		    		let sent = client.tx_buffer.map_or(false, |tx_buffer| {
		    			// Get the length to send, and add one for the ID byte.
		    			let len = tx_buffer.length() as u16 + 1;
		    			console_mux_tx_buffer[0] = len >> 8;
		    			console_mux_tx_buffer[1] = len & 0xFF;

		    			// Set the sender id in the message. We have to use the
		    			// app id if one is set.
		    			match client.tx_subid.get() {
		    				Some(id) => console_mux_tx_buffer[2] = id,
		    				None => console_mux_tx_buffer[2] = client.id,
		    			}

		    			// Copy the payload into the outgoing buffer.
		    			for (a, b) in console_mux_tx_buffer.iter_mut().skip(3).zip(tx_buffer) {
		    			    *a = *b;
		    			}
		    			self.uart.transmit(console_mux_tx_buffer);
		    			self.active_transmitter.set(client.id.get());

		    			// Return that we transmitted something.
		    			true
		    		});
		    		if sent {
		    			break;
		    		}
				});

		    });
	    }
    }

    // // Process the command in the command buffer and clear the buffer.
    // fn read_command(&self) {
    //     self.command_buffer.map(|command| {
    //         let mut terminator = 0;
    //         let len = command.len();
    //         for i in 0..len {
    //             if command[i] == 0 {
    //                 terminator = i;
    //                 break;
    //             }
    //         }
    //         //debug!("Command: {}-{} {:?}", start, terminator, command);
    //         // A command is valid only if it starts inside the buffer,
    //         // ends before the beginning of the buffer, and ends after
    //         // it starts.
    //         if terminator > 0 {
    //             let cmd_str = str::from_utf8(&command[0..terminator]);
    //             match cmd_str {
    //                 Ok(s) => {
    //                     let clean_str = s.trim();
    //                     if clean_str.starts_with("help") {
    //                         debug!("Welcome to the process console.");
    //                         debug!("Valid commands are: help status list stop start");
    //                     } else if clean_str.starts_with("start") {
    //                         let argument = clean_str.split_whitespace().nth(1);
    //                         argument.map(|name| {
    //                             self.kernel.process_each_capability(
    //                                 &self.capability,
    //                                 |_i, proc| {
    //                                     let proc_name = proc.get_process_name();
    //                                     if proc_name == name {
    //                                         proc.resume();
    //                                         debug!("Process {} resumed.", name);
    //                                     }
    //                                 },
    //                             );
    //                         });
    //                     } else if clean_str.starts_with("stop") {
    //                         let argument = clean_str.split_whitespace().nth(1);
    //                         argument.map(|name| {
    //                             self.kernel.process_each_capability(
    //                                 &self.capability,
    //                                 |_i, proc| {
    //                                     let proc_name = proc.get_process_name();
    //                                     if proc_name == name {
    //                                         proc.stop();
    //                                         debug!("Process {} stopped", proc_name);
    //                                     }
    //                                 },
    //                             );
    //                         });
    //                     } else if clean_str.starts_with("fault") {
    //                         let argument = clean_str.split_whitespace().nth(1);
    //                         argument.map(|name| {
    //                             self.kernel.process_each_capability(
    //                                 &self.capability,
    //                                 |_i, proc| {
    //                                     let proc_name = proc.get_process_name();
    //                                     if proc_name == name {
    //                                         proc.set_fault_state();
    //                                         debug!("Process {} now faulted", proc_name);
    //                                     }
    //                                 },
    //                             );
    //                         });
    //                     } else if clean_str.starts_with("list") {
    //                         debug!(" PID    Name                Quanta  Syscalls  Dropped Callbacks    State");
    //                         self.kernel
    //                             .process_each_capability(&self.capability, |i, proc| {
    //                                 let pname = proc.get_process_name();
    //                                 debug!(
    //                                     "  {:02}\t{:<20}{:6}{:10}{:19}  {:?}",
    //                                     i,
    //                                     pname,
    //                                     proc.debug_timeslice_expiration_count(),
    //                                     proc.debug_syscall_count(),
    //                                     proc.debug_dropped_callback_count(),
    //                                     proc.get_state()
    //                                 );
    //                             });
    //                     } else if clean_str.starts_with("status") {
    //                         let info: KernelInfo = KernelInfo::new(self.kernel);
    //                         debug!(
    //                             "Total processes: {}",
    //                             info.number_loaded_processes(&self.capability)
    //                         );
    //                         debug!(
    //                             "Active processes: {}",
    //                             info.number_active_processes(&self.capability)
    //                         );
    //                         debug!(
    //                             "Timeslice expirations: {}",
    //                             info.timeslice_expirations(&self.capability)
    //                         );
    //                     } else {
    //                         debug!("Valid commands are: help status list stop start fault");
    //                     }
    //                 }
    //                 Err(_e) => debug!("Invalid command: {:?}", command),
    //             }
    //         }
    //     });
    //     self.command_buffer.map(|command| {
    //         command[0] = 0;
    //     });
    //     self.command_index.set(0);
    // }

    // fn write_byte(&self, byte: u8) -> ReturnCode {
    //     if self.tx_in_progress.get() {
    //         ReturnCode::EBUSY
    //     } else {
    //         self.tx_in_progress.set(true);
    //         self.tx_buffer.take().map(|buffer| {
    //             buffer[0] = byte;
    //             self.uart.transmit_buffer(buffer, 1);
    //         });
    //         ReturnCode::SUCCESS
    //     }
    // }

    // fn write_bytes(&self, bytes: &[u8]) -> ReturnCode {
    //     if self.tx_in_progress.get() {
    //         ReturnCode::EBUSY
    //     } else {
    //         self.tx_in_progress.set(true);
    //         self.tx_buffer.take().map(|buffer| {
    //             let len = cmp::min(bytes.len(), buffer.len());
    //             for i in 0..len {
    //                 buffer[i] = bytes[i];
    //             }
    //             self.uart.transmit_buffer(buffer, len);
    //         });
    //         ReturnCode::SUCCESS
    //     }
    // }
}

impl<'a> ConsoleMuxClient<'a> {
    pub fn new(
        mux: &'a ConsoleMux<'a>,
    ) -> ConsoleMuxClient<'a> {
        ConsoleMuxClient {
            mux: mux,
            id: Cell::new(0),
            client: OptionalCell::empty(),
            rx_buffer: TakeCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_buffer_len: Cell::new(0),
            tx_subid: OptionalCell::empty(),
        }
    }

    /// Must be called right after `static_init!()`.
    pub fn setup(&'a self) {
        self.mux.register(self);
    }

    /// Setup this `ConsoleMuxClient` as the app_console designed to handle
    /// console messages to and from applications. Must be called right after
    /// `static_init!()`.
    pub fn setup_as_app_console(&'a self) {
        self.mux.register_app_console(self);
    }
}

impl<'a> ListNode<'a, ConsoleMuxClient<'a>> for ConsoleMuxClient<'a> {
    fn next(&'a self) -> &'a ListLink<'a, ConsoleMuxClient<'a>> {
        &self.next
    }
}

impl<'a> Console<'a> for ConsoleMuxClient<'a> {
    fn transmit_message(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
        app_id: Option<u8>,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
    	// Save the buffer for the console client.
    	self.tx_buffer.set(tx_buffer);
    	self.tx_buffer_len.set(tx_len);

    	// Save the app id if this comes from the app console.
    	match app_id {
    		Some(id) => self.tx_subid.set(id),
    		None => self.tx_subid.clear()
    	}

    	// Try to send the buffer, no guarantee that it will go out right now.
    	self.mux.transmit();

    	(ReturnCode::SUCCESS, None)
    }

    // Just have to save the rx buffer in case a command comes in for this
    // particular console.
    fn receive_message(&self, rx_buffer: &'static mut [u8]) -> (ReturnCode, Option<&'static mut [u8]>) {
    	self.rx_buffer.set(rx_buffer);
    	(ReturnCode::SUCCESS, None)
    }

    fn set_client(&self, client: &'a ConsoleClient) {
    	self.client.set(client);
    }
}

impl<'a> uart::TransmitClient for ConsoleMux<'a> {
    fn transmitted_buffer(&self, buffer: &'static mut [u8], tx_len: usize, rcode: ReturnCode) {
    	// Replace the `ConsoleMux` tx buffer since that is what we actually
    	// passed to the UART.
    	self.tx_buffer.replace(buffer);

    	// Now we need to pass the tx buffer for the console back to the console
    	// so it can transmit again.
    	match self.active_transmitter.get() {
    		Some(id) => {
    			self.consoles.iter().for_each(|client| {
    			    if id == client.id || (id >= 128 && client.id == 128) {
    			        client.tx_buffer.take().map(|tx_buffer| {
    			        	client.client.transmitted_command(tx_buffer, tx_len, rcode);
    			        });
    			    }
    			});
    		}
    		None => {}
    	}

    	// Mark that there is no transmitter.
    	self.active_transmitter.clear();

    	// See if there is more to transmit. This will just do nothing if there
    	// are no consoles trying to send data.
    	self.transmit();
    }
}

impl<'a> uart::ReceiveClient for ConsoleMux<'a> {
    fn received_buffer(
        &self,
        read_buf: &'static mut [u8],
        rx_len: usize,
        _rcode: ReturnCode,
        error: uart::Error,
    ) {
        // let mut execute = false;

        if error == uart::Error::None {
        	match self.state.get() {
        		State::WaitingHeader => {
        			match rx_len {
        				3 => {
        					// We got the expected number of header bytes.
        					let length: u16 = (read_buf[0] as u16 << 8) + (read_buf[1] as u16);
        					let id: u8 = read_buf[2];
        					self.state.set(State::ReceivedHeader{length, id});

        					// Setup the remainder of the read. Since we already
        					// read the id byte, we subtract one from the
        					// length.
        					self.uart.receive_buffer(buffer, length-1);
        				}
        				_ => {
        					debug!("ConsoleMux invalid receive.");
        				}
        			}
        		}

        		State::ReceivedHeader{id, length} => {
        			match rx_len {
        				0 => debug!("ConsoleMux recv 0."),

        				_ => {
        					match id {
        						0 => {
        							// Copy the received bytes into our local
        							// command buffer.

        							// The `ConsoleMux` handles this command.
        							self.handle_internal_command(rx_len);
        						}
        						_ => {
        							// Handle all kernel console messages.

        							// Look through all consoles to find one
        							// that matches.
        							self.consoles.iter().for_each(|client| {
        							    if id == client.id || (id >= 128 && client.id == 128) {
        							        client.rx_buffer.map(|rx_buffer| {
        							        	// Copy the receive bytes to the
        							        	// passed in buffer from the
        							        	// console.

        							        	client.client.received_command(rx_buffer, rx_len, id);
        							        });
        							    }
        							});
        						}
        						// 128..=255 => {
        						// 	// Handle all application console messages.

        						// 	self.app_console.map(|app_console|{
        						// 		app_console.rx_buffer.take().map(|rx_buffer| {
	        					// 			// Copy the receive bytes to the
	        					// 			// passed in buffer from the
	        					// 			// app console.


	        					// 			app_console.client.received_command(rx_buffer, rx_len, id);
	        					// 		});

	        					// 	});
        						// }
        					}
        					self.uart.receive_buffer(buffer, 3);
        				}
        			}
        		}
        	}


            // match rx_len {
            //     0 => debug!("InteractiveConsoleMux had read of 0 bytes"),
            //     1 => {
            //         self.command_buffer.map(|command| {
            //             let index = self.command_index.get() as usize;
            //             if read_buf[0] == ('\n' as u8) || read_buf[0] == ('\r' as u8) {
            //                 execute = true;
            //                 self.write_bytes(&['\r' as u8, '\n' as u8]);
            //             } else if read_buf[0] == ('\x08' as u8) && index > 0 {
            //                 // Backspace, echo and remove last byte
            //                 // Note echo is '\b \b' to erase
            //                 self.write_bytes(&['\x08' as u8, ' ' as u8, '\x08' as u8]);
            //                 command[index - 1] = '\0' as u8;
            //                 self.command_index.set(index - 1);
            //             } else if index < (command.len() - 1) && read_buf[0] < 128 {
            //                 // For some reason, sometimes reads return > 127 but no error,
            //                 // which causes utf-8 decoding failure, so check byte is < 128. -pal

            //                 // Echo the byte and store it
            //                 self.write_byte(read_buf[0]);
            //                 command[index] = read_buf[0];
            //                 self.command_index.set(index + 1);
            //                 command[index + 1] = 0;
            //             }
            //         });
            //     }
            //     _ => debug!(
            //         "ProcessConsole issues reads of 1 byte, but receive_complete was length {}",
            //         rx_len
            //     ),
            // };
        }
        // self.rx_in_progress.set(true);
        // self.uart.receive_buffer(read_buf, 1);

        // if execute {
        //     self.read_command();
        // }
    }
}

