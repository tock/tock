//! Implements a text console over the UART that allows
//! a terminal to inspect and control userspace processes.
//!
//! For a more in-depth documentation check /doc/Process_Console.md
use core::cell::Cell;
use core::cmp;
use core::fmt;
use core::fmt::write;
use core::str;
use kernel::capabilities::ProcessManagementCapability;
use kernel::hil::time::ConvertTicks;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::ProcessId;

use kernel::debug;
use kernel::hil::time::{Alarm, AlarmClient};
use kernel::hil::uart;
use kernel::introspection::KernelInfo;
use kernel::process::{ProcessPrinter, ProcessPrinterContext, State};
use kernel::utilities::binary_write::BinaryWrite;
use kernel::ErrorCode;
use kernel::Kernel;

/// Buffer to hold outgoing data that is passed to the UART hardware.
pub const WRITE_BUF_LEN: usize = 500;
/// Buffer responses are initially held in until copied to the TX buffer and
/// transmitted.
pub const QUEUE_BUF_LEN: usize = 300;
/// Since reads are byte-by-byte, to properly echo what's typed,
/// we can use a very small read buffer.
pub const READ_BUF_LEN: usize = 4;
/// Commands can be up to 32 bytes long: since commands themselves are 4-5
/// characters, limiting arguments to 25 bytes or so seems fine for now.
pub const COMMAND_BUF_LEN: usize = 32;
/// Default size for the history command.
pub const DEFAULT_COMMAND_HISTORY_LEN: usize = 10;

/// List of valid commands for printing help. Consolidated as these are
/// displayed in a few different cases.
const VALID_COMMANDS_STR: &[u8] =
    b"help status list stop start fault boot terminate process kernel panic\r\n";

/// Escape character for ANSI escape sequences.
const ESC: u8 = '\x1B' as u8;

/// End of line character.
const EOL: u8 = '\0' as u8;

/// States used for state machine to allow printing large strings asynchronously
/// across multiple calls. This reduces the size of the buffer needed to print
/// each section of the debug message.
#[derive(PartialEq, Eq, Copy, Clone)]
enum WriterState {
    Empty,
    KernelStart,
    KernelBss,
    KernelInit,
    KernelStack,
    KernelRoData,
    KernelText,
    ProcessPrint {
        process_id: ProcessId,
        context: Option<ProcessPrinterContext>,
    },
    List {
        index: isize,
        total: isize,
    },
}

impl Default for WriterState {
    fn default() -> Self {
        WriterState::Empty
    }
}

/// Data structure to hold addresses about how the kernel is stored in memory on
/// the chip.
///
/// All "end" addresses are the memory addresses immediately following the end
/// of the memory region.
pub struct KernelAddresses {
    pub stack_start: *const u8,
    pub stack_end: *const u8,
    pub text_start: *const u8,
    pub text_end: *const u8,
    pub read_only_data_start: *const u8,
    pub relocations_start: *const u8,
    pub relocations_end: *const u8,
    pub bss_start: *const u8,
    pub bss_end: *const u8,
}

pub struct ProcessConsole<
    'a,
    const COMMAND_HISTORY_LEN: usize,
    A: Alarm<'a>,
    C: ProcessManagementCapability,
> {
    uart: &'a dyn uart::UartData<'a>,
    alarm: &'a A,
    process_printer: &'a dyn ProcessPrinter,
    tx_in_progress: Cell<bool>,
    tx_buffer: TakeCell<'static, [u8]>,
    queue_buffer: TakeCell<'static, [u8]>,
    queue_size: Cell<usize>,
    writer_state: Cell<WriterState>,
    rx_in_progress: Cell<bool>,
    rx_buffer: TakeCell<'static, [u8]>,
    command_buffer: TakeCell<'static, [u8]>,
    command_index: Cell<usize>,

    /// Keep a history of inserted commands
    command_history: TakeCell<'static, [Command; COMMAND_HISTORY_LEN]>,

    control_seq_in_progress: Cell<bool>,
    modified_in_history: Cell<bool>,
    modified_byte: Cell<u8>,

    /// Index of the last copied command in the history
    command_history_index: OptionalCell<usize>,

    /// Keep the previously read byte to consider \r\n sequences
    /// as a single \n.
    previous_byte: Cell<u8>,

    /// Flag to mark that the process console is active and has called receive
    /// from the underlying UART.
    running: Cell<bool>,

    /// Internal flag that the process console should parse the command it just
    /// received after finishing echoing the last newline character.
    execute: Cell<bool>,

    /// Reference to the kernel object so we can access process state.
    kernel: &'static Kernel,

    /// Memory addresses of where the kernel is placed in memory on chip.
    kernel_addresses: KernelAddresses,

    /// This capsule needs to use potentially dangerous APIs related to
    /// processes, and requires a capability to access those APIs.
    capability: C,
}

#[derive(Copy, Clone)]
pub struct Command {
    buf: [u8; COMMAND_BUF_LEN],
    len: usize,
}

impl Command {
    /// Write the buffer with the provided data.
    /// If the provided data's length is smaller than the buffer length,
    /// the left over bytes are not modified due to '\0' termination.
    fn write(&mut self, buf: &[u8; COMMAND_BUF_LEN]) {
        self.len = buf
            .iter()
            .position(|a| *a == EOL)
            .unwrap_or(COMMAND_BUF_LEN);

        (&mut self.buf).copy_from_slice(buf);
    }

    fn insert_byte(&mut self, byte: u8) {
        if let Some(buf_byte) = self.buf.get_mut(self.len) {
            *buf_byte = byte;
            self.len = self.len + 1;
        }
    }

    fn delete_last_byte(&mut self) {
        if let Some(buf_byte) = self.buf.get_mut(self.len - 1) {
            *buf_byte = EOL;
            self.len = self.len - 1;
        }
    }

    fn clear(&mut self) {
        self.buf.iter_mut().for_each(|x| *x = EOL);
        self.len = 0;
    }
}

impl Default for Command {
    fn default() -> Self {
        Command {
            buf: [EOL; COMMAND_BUF_LEN],
            len: 0,
        }
    }
}

impl PartialEq<[u8; COMMAND_BUF_LEN]> for Command {
    fn eq(&self, other_buf: &[u8; COMMAND_BUF_LEN]) -> bool {
        self.buf
            .iter()
            .zip(other_buf.iter())
            .take_while(|(a, b)| **a != EOL || **b != EOL)
            .all(|(a, b)| *a == *b)
    }
}

pub struct ConsoleWriter {
    buf: [u8; 500],
    size: usize,
}
impl ConsoleWriter {
    pub fn new() -> ConsoleWriter {
        ConsoleWriter {
            buf: [0; 500],
            size: 0,
        }
    }
    pub fn clear(&mut self) {
        self.size = 0;
    }
}
impl fmt::Write for ConsoleWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let curr = (s).as_bytes().len();
        self.buf[self.size..self.size + curr].copy_from_slice(&(s).as_bytes()[..]);
        self.size += curr;
        Ok(())
    }
}

impl BinaryWrite for ConsoleWriter {
    fn write_buffer(&mut self, buffer: &[u8]) -> Result<usize, ()> {
        let start = self.size;
        let remaining = self.buf.len() - start;
        let to_send = core::cmp::min(buffer.len(), remaining);
        self.buf[start..start + to_send].copy_from_slice(&buffer[..to_send]);
        self.size += to_send;
        Ok(to_send)
    }
}

impl<'a, const COMMAND_HISTORY_LEN: usize, A: Alarm<'a>, C: ProcessManagementCapability>
    ProcessConsole<'a, COMMAND_HISTORY_LEN, A, C>
{
    pub fn new(
        uart: &'a dyn uart::UartData<'a>,
        alarm: &'a A,
        process_printer: &'a dyn ProcessPrinter,
        tx_buffer: &'static mut [u8],
        rx_buffer: &'static mut [u8],
        queue_buffer: &'static mut [u8],
        cmd_buffer: &'static mut [u8],
        cmd_history_buffer: &'static mut [Command; COMMAND_HISTORY_LEN],
        kernel: &'static Kernel,
        kernel_addresses: KernelAddresses,
        capability: C,
    ) -> ProcessConsole<'a, COMMAND_HISTORY_LEN, A, C> {
        ProcessConsole {
            uart: uart,
            alarm: alarm,
            process_printer,
            tx_in_progress: Cell::new(false),
            tx_buffer: TakeCell::new(tx_buffer),
            queue_buffer: TakeCell::new(queue_buffer),
            queue_size: Cell::new(0),
            writer_state: Cell::new(WriterState::Empty),
            rx_in_progress: Cell::new(false),
            rx_buffer: TakeCell::new(rx_buffer),
            command_buffer: TakeCell::new(cmd_buffer),
            command_index: Cell::new(0),

            control_seq_in_progress: Cell::new(false),
            modified_byte: Cell::new(EOL),
            modified_in_history: Cell::new(false),
            command_history: TakeCell::new(cmd_history_buffer),
            command_history_index: OptionalCell::empty(),

            previous_byte: Cell::new(0),

            running: Cell::new(false),
            execute: Cell::new(false),
            kernel: kernel,
            kernel_addresses: kernel_addresses,
            capability: capability,
        }
    }

    /// Start the process console listening for user commands.
    pub fn start(&self) -> Result<(), ErrorCode> {
        if self.running.get() == false {
            self.alarm
                .set_alarm(self.alarm.now(), self.alarm.ticks_from_ms(100));
            self.running.set(true);
        }
        Ok(())
    }

    /// Print base information about the kernel version installed and the help
    /// message.
    pub fn display_welcome(&self) {
        // Start if not already started.
        if self.running.get() == false {
            self.rx_buffer.take().map(|buffer| {
                self.rx_in_progress.set(true);
                let _ = self.uart.receive_buffer(buffer, 1);
                self.running.set(true);
            });
        }

        // Display pconsole info.
        let mut console_writer = ConsoleWriter::new();
        let _ = write(
            &mut console_writer,
            format_args!(
                "Kernel version: {}.{} (build {})\r\n",
                kernel::KERNEL_MAJOR_VERSION,
                kernel::KERNEL_MINOR_VERSION,
                option_env!("TOCK_KERNEL_VERSION").unwrap_or("unknown"),
            ),
        );
        let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);

        let _ = self.write_bytes(b"Welcome to the process console.\r\n");
        let _ = self.write_bytes(b"Valid commands are: ");
        let _ = self.write_bytes(VALID_COMMANDS_STR);
        self.prompt();
    }

    /// Simple state machine helper function that identifies the next state for
    /// printing log debug messages.
    fn next_state(&self, state: WriterState) -> WriterState {
        match state {
            WriterState::KernelStart => WriterState::KernelBss,
            WriterState::KernelBss => WriterState::KernelInit,
            WriterState::KernelInit => WriterState::KernelStack,
            WriterState::KernelStack => WriterState::KernelRoData,
            WriterState::KernelRoData => WriterState::KernelText,
            WriterState::KernelText => WriterState::Empty,
            WriterState::ProcessPrint {
                process_id,
                context,
            } => WriterState::ProcessPrint {
                process_id,
                context,
            },
            WriterState::List { index, total } => {
                // Next state just increments index, unless we are at end in
                // which next state is just the empty state.
                if index + 1 == total {
                    WriterState::Empty
                } else {
                    WriterState::List {
                        index: index + 1,
                        total,
                    }
                }
            }
            WriterState::Empty => WriterState::Empty,
        }
    }

    /// Create the debug message for each state in the state machine.
    fn create_state_buffer(&self, state: WriterState) {
        match state {
            WriterState::KernelBss => {
                let mut console_writer = ConsoleWriter::new();

                let bss_start = self.kernel_addresses.bss_start as usize;
                let bss_end = self.kernel_addresses.bss_end as usize;
                let bss_size = bss_end - bss_start;

                let _ = write(
                    &mut console_writer,
                    format_args!(
                        "\r\n ╔═══════════╤══════════════════════════════╗\
                    \r\n ║  Address  │ Region Name    Used (bytes)  ║\
                    \r\n ╚{:#010X}═╪══════════════════════════════╝\
                    \r\n             │   BSS        {:6}",
                        bss_end, bss_size
                    ),
                );

                let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
            }
            WriterState::KernelInit => {
                let mut console_writer = ConsoleWriter::new();

                let relocate_start = self.kernel_addresses.relocations_start as usize;
                let relocate_end = self.kernel_addresses.relocations_end as usize;
                let relocate_size = relocate_end - relocate_start;

                let _ = write(
                    &mut console_writer,
                    format_args!(
                        "\
                    \r\n  {:#010X} ┼─────────────────────────────── S\
                    \r\n             │   Relocate   {:6}            R",
                        relocate_end, relocate_size
                    ),
                );
                let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
            }
            WriterState::KernelStack => {
                let mut console_writer = ConsoleWriter::new();

                let stack_start = self.kernel_addresses.stack_start as usize;
                let stack_end = self.kernel_addresses.stack_end as usize;
                let stack_size = stack_end - stack_start;

                let _ = write(
                    &mut console_writer,
                    format_args!(
                        "\
                    \r\n  {:#010X} ┼─────────────────────────────── A\
                    \r\n             │ ▼ Stack      {:6}            M\
                    \r\n  {:#010X} ┼───────────────────────────────",
                        stack_end, stack_size, stack_start
                    ),
                );
                let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
            }
            WriterState::KernelRoData => {
                let mut console_writer = ConsoleWriter::new();

                let rodata_start = self.kernel_addresses.read_only_data_start as usize;
                let text_end = self.kernel_addresses.text_end as usize;
                let rodata_size = text_end - rodata_start;

                let _ = write(
                    &mut console_writer,
                    format_args!(
                        "\
                        \r\n             .....\
                     \r\n  {:#010X} ┼─────────────────────────────── F\
                     \r\n             │   RoData     {:6}            L",
                        text_end, rodata_size
                    ),
                );
                let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
            }
            WriterState::KernelText => {
                let mut console_writer = ConsoleWriter::new();

                let code_start = self.kernel_addresses.text_start as usize;
                let code_end = self.kernel_addresses.read_only_data_start as usize;
                let code_size = code_end - code_start;

                let _ = write(
                    &mut console_writer,
                    format_args!(
                        "\
                     \r\n  {:#010X} ┼─────────────────────────────── A\
                     \r\n             │   Code       {:6}            S\
                     \r\n  {:#010X} ┼─────────────────────────────── H\
                     \r\n",
                        code_end, code_size, code_start
                    ),
                );
                let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
            }
            WriterState::ProcessPrint {
                process_id,
                context,
            } => {
                self.kernel
                    .process_each_capability(&self.capability, |process| {
                        if process_id == process.processid() {
                            let mut console_writer = ConsoleWriter::new();
                            let new_context = self.process_printer.print_overview(
                                process,
                                &mut console_writer,
                                context,
                            );

                            let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);

                            if new_context.is_some() {
                                self.writer_state.replace(WriterState::ProcessPrint {
                                    process_id: process_id,
                                    context: new_context,
                                });
                            } else {
                                self.writer_state.replace(WriterState::Empty);
                                // As setting the next state here to Empty does not
                                // go through this match again before reading a new command,
                                // we have to print the prompt here.
                                self.prompt();
                            }
                        }
                    });
            }
            WriterState::List { index, total: _ } => {
                let mut local_index = -1;
                self.kernel
                    .process_each_capability(&self.capability, |process| {
                        local_index += 1;
                        if local_index == index {
                            let info: KernelInfo = KernelInfo::new(self.kernel);

                            let pname = process.get_process_name();
                            let process_id = process.processid();
                            let (grants_used, grants_total) =
                                info.number_app_grant_uses(process_id, &self.capability);
                            let mut console_writer = ConsoleWriter::new();
                            let _ = write(
                                &mut console_writer,
                                format_args!(
                                    " {:<7?}{:<20}{:6}{:10}{:10}  {:2}/{:2}   {:?}\r\n",
                                    process_id,
                                    pname,
                                    process.debug_timeslice_expiration_count(),
                                    process.debug_syscall_count(),
                                    process.get_restart_count(),
                                    grants_used,
                                    grants_total,
                                    process.get_state(),
                                ),
                            );

                            let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
                        }
                    });
            }
            WriterState::Empty => {
                self.prompt();
            }
            _ => {}
        }
    }

    // Process the command in the command buffer and clear the buffer.
    fn read_command(&self) {
        self.command_buffer.map(|command| {
            let mut terminator = 0;
            let len = command.len();
            for i in 0..len {
                if command[i] == 0 {
                    terminator = i;
                    break;
                }
            }

            // A command is valid only if it starts inside the buffer,
            // ends before the beginning of the buffer, and ends after
            // it starts.
            if terminator > 0 {
                let cmd_str = str::from_utf8(&command[0..terminator]);

                match cmd_str {
                    Ok(s) => {
                        let clean_str = s.trim();

                        // Check if the command history is enabled by the user
                        // and check if the command is not full of whitespaces
                        if COMMAND_HISTORY_LEN > 1 {
                            if clean_str.len() > 0 {
                                self.command_history.map(|cmd_arr| {
                                    let mut command_array = [0; COMMAND_BUF_LEN];
                                    command_array.copy_from_slice(command);

                                    if cmd_arr[1] != command_array {
                                        cmd_arr.rotate_right(1);
                                        cmd_arr[1].write(&command_array);
                                    }
                                });
                            }
                        }

                        if clean_str.starts_with("help") {
                            let _ = self.write_bytes(b"Welcome to the process console.\r\n");
                            let _ = self.write_bytes(b"Valid commands are: ");
                            let _ = self.write_bytes(VALID_COMMANDS_STR);
                        } else if clean_str.starts_with("start") {
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                self.kernel
                                    .process_each_capability(&self.capability, |proc| {
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name {
                                            proc.resume();
                                            let mut console_writer = ConsoleWriter::new();
                                            let _ = write(
                                                &mut console_writer,
                                                format_args!("Process {} resumed.\r\n", name),
                                            );

                                            let _ = self.write_bytes(
                                                &(console_writer.buf)[..console_writer.size],
                                            );
                                        }
                                    });
                            });
                        } else if clean_str.starts_with("stop") {
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                self.kernel
                                    .process_each_capability(&self.capability, |proc| {
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name {
                                            proc.stop();
                                            let mut console_writer = ConsoleWriter::new();
                                            let _ = write(
                                                &mut console_writer,
                                                format_args!("Process {} stopped\r\n", proc_name),
                                            );

                                            let _ = self.write_bytes(
                                                &(console_writer.buf)[..console_writer.size],
                                            );
                                        }
                                    });
                            });
                        } else if clean_str.starts_with("fault") {
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                self.kernel
                                    .process_each_capability(&self.capability, |proc| {
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name {
                                            proc.set_fault_state();
                                            let mut console_writer = ConsoleWriter::new();
                                            let _ = write(
                                                &mut console_writer,
                                                format_args!(
                                                    "Process {} now faulted\r\n",
                                                    proc_name
                                                ),
                                            );

                                            let _ = self.write_bytes(
                                                &(console_writer.buf)[..console_writer.size],
                                            );
                                        }
                                    });
                            });
                        } else if clean_str.starts_with("terminate") {
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                self.kernel
                                    .process_each_capability(&self.capability, |proc| {
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name {
                                            proc.terminate(None);
                                            let mut console_writer = ConsoleWriter::new();
                                            let _ = write(
                                                &mut console_writer,
                                                format_args!("Process {} terminated\n", proc_name),
                                            );

                                            let _ = self.write_bytes(
                                                &(console_writer.buf)[..console_writer.size],
                                            );
                                        }
                                    });
                            });
                        } else if clean_str.starts_with("boot") {
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                self.kernel
                                    .process_each_capability(&self.capability, |proc| {
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name
                                            && proc.get_state() == State::Terminated
                                        {
                                            proc.try_restart(None);
                                        }
                                    });
                            });
                        } else if clean_str.starts_with("list") {
                            let _ = self.write_bytes(b" PID    Name                Quanta  ");
                            let _ = self.write_bytes(b"Syscalls  Restarts  Grants  State\r\n");

                            // Count the number of current processes.
                            let mut count = 0;
                            self.kernel.process_each_capability(&self.capability, |_| {
                                count += 1;
                            });

                            if count > 0 {
                                // Start the state machine to print each separately.
                                self.write_state(WriterState::List {
                                    index: -1,
                                    total: count,
                                });
                            }
                        } else if clean_str.starts_with("status") {
                            let info: KernelInfo = KernelInfo::new(self.kernel);
                            let mut console_writer = ConsoleWriter::new();
                            let _ = write(
                                &mut console_writer,
                                format_args!(
                                    "Total processes: {}\r\n",
                                    info.number_loaded_processes(&self.capability)
                                ),
                            );
                            let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
                            console_writer.clear();
                            let _ = write(
                                &mut console_writer,
                                format_args!(
                                    "Active processes: {}\r\n",
                                    info.number_active_processes(&self.capability)
                                ),
                            );
                            let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
                            console_writer.clear();
                            let _ = write(
                                &mut console_writer,
                                format_args!(
                                    "Timeslice expirations: {}\r\n",
                                    info.timeslice_expirations(&self.capability)
                                ),
                            );
                            let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
                        } else if clean_str.starts_with("process") {
                            let argument = clean_str.split_whitespace().nth(1);
                            argument.map(|name| {
                                // If two processes have the same name, only
                                // print the first one we find.
                                let mut found = false;
                                self.kernel
                                    .process_each_capability(&self.capability, |proc| {
                                        if found {
                                            return;
                                        }
                                        let proc_name = proc.get_process_name();
                                        if proc_name == name {
                                            let mut console_writer = ConsoleWriter::new();
                                            let mut context: Option<ProcessPrinterContext> = None;
                                            context = self.process_printer.print_overview(
                                                proc,
                                                &mut console_writer,
                                                context,
                                            );

                                            let _ = self.write_bytes(
                                                &(console_writer.buf)[..console_writer.size],
                                            );

                                            if context.is_some() {
                                                self.writer_state.replace(
                                                    WriterState::ProcessPrint {
                                                        process_id: proc.processid(),
                                                        context: context,
                                                    },
                                                );
                                            }

                                            found = true;
                                        }
                                    });
                            });
                        } else if clean_str.starts_with("kernel") {
                            let mut console_writer = ConsoleWriter::new();
                            let _ = write(
                                &mut console_writer,
                                format_args!(
                                    "Kernel version: {}.{} (build {})\r\n",
                                    kernel::KERNEL_MAJOR_VERSION,
                                    kernel::KERNEL_MINOR_VERSION,
                                    option_env!("TOCK_KERNEL_VERSION").unwrap_or("unknown")
                                ),
                            );
                            let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
                            console_writer.clear();

                            // Prints kernel memory by moving the writer to the
                            // start state.
                            self.writer_state.replace(WriterState::KernelStart);
                        } else if clean_str.starts_with("panic") {
                            panic!("Process Console forced a kernel panic.");
                        } else {
                            let _ = self.write_bytes(b"Valid commands are: ");
                            let _ = self.write_bytes(VALID_COMMANDS_STR);
                        }
                    }
                    Err(_e) => {
                        let mut console_writer = ConsoleWriter::new();
                        let _ = write(
                            &mut console_writer,
                            format_args!("Invalid command: {:?}", command),
                        );
                        let _ = self.write_bytes(&(console_writer.buf)[..console_writer.size]);
                    }
                }
            }
        });
        self.command_buffer.map(|command| {
            command[0] = 0;
        });
        self.command_index.set(0);
        if self.writer_state.get() == WriterState::Empty {
            self.prompt();
        }
    }

    fn prompt(&self) {
        let _ = self.write_bytes(b"tock$ ");
    }

    /// Start or iterate the state machine for an asynchronous write operation
    /// spread across multiple callback cycles.
    fn write_state(&self, state: WriterState) {
        self.writer_state.replace(self.next_state(state));
        self.create_state_buffer(self.writer_state.get());
    }

    fn write_byte(&self, byte: u8) -> Result<(), ErrorCode> {
        if self.tx_in_progress.get() {
            self.queue_buffer.map(|buf| {
                buf[self.queue_size.get()] = byte;
                self.queue_size.set(self.queue_size.get() + 1);
            });
            Err(ErrorCode::BUSY)
        } else {
            self.tx_in_progress.set(true);
            self.tx_buffer.take().map(|buffer| {
                buffer[0] = byte;
                let _ = self.uart.transmit_buffer(buffer, 1);
            });
            Ok(())
        }
    }

    fn write_bytes(&self, bytes: &[u8]) -> Result<(), ErrorCode> {
        if self.tx_in_progress.get() {
            self.queue_buffer.map(|buf| {
                let size = self.queue_size.get();
                let len = cmp::min(bytes.len(), buf.len() - size);
                (&mut buf[size..size + len]).copy_from_slice(&bytes[..len]);
                self.queue_size.set(size + len);
            });
            Err(ErrorCode::BUSY)
        } else {
            self.tx_in_progress.set(true);
            self.tx_buffer.take().map(|buffer| {
                let len = cmp::min(bytes.len(), buffer.len());
                // Copy elements of `bytes` into `buffer`
                (&mut buffer[..len]).copy_from_slice(&bytes[..len]);
                let _ = self.uart.transmit_buffer(buffer, len);
            });
            Ok(())
        }
    }

    /// If there is anything in the queue, copy it to the TX buffer and send
    /// it to the UART.
    ///
    /// Returns Ok(usize) with the number of bytes sent from the queue. If Ok(0)
    /// is returned, nothing was sent and the UART is free.
    fn handle_queue(&self) -> Result<usize, ErrorCode> {
        if self.tx_in_progress.get() {
            // This shouldn't happen because we should only try to handle the
            // queue when nothing else is happening, but still have the check
            // for safety.
            return Err(ErrorCode::BUSY);
        }

        self.queue_buffer.map_or(Err(ErrorCode::FAIL), |qbuf| {
            let qlen = self.queue_size.get();

            if qlen > 0 {
                self.tx_buffer.take().map_or(Err(ErrorCode::FAIL), |txbuf| {
                    let txlen = cmp::min(qlen, txbuf.len());

                    // Copy elements of the queue into the TX buffer.
                    (&mut txbuf[..txlen]).copy_from_slice(&qbuf[..txlen]);

                    // TODO: If the queue needs to print over multiple TX
                    // buffers, we need to shift the remaining contents of the
                    // queue back to index 0.
                    // if qlen > txlen {
                    //     (&mut qbuf[txlen..qlen]).copy_from_slice(&qbuf[txlen..qlen]);
                    // }

                    // Mark that we sent at least some of the queue.
                    let remaining = qlen - txlen;
                    self.queue_size.set(remaining);

                    self.tx_in_progress.set(true);
                    let _ = self.uart.transmit_buffer(txbuf, txlen);
                    Ok(txlen)
                })
            } else {
                // Queue was empty, nothing to do.
                Ok(0)
            }
        })
    }
}

impl<'a, const COMMAND_HISTORY_LEN: usize, A: Alarm<'a>, C: ProcessManagementCapability> AlarmClient
    for ProcessConsole<'a, COMMAND_HISTORY_LEN, A, C>
{
    fn alarm(&self) {
        self.prompt();
        self.rx_buffer.take().map(|buffer| {
            self.rx_in_progress.set(true);
            let _ = self.uart.receive_buffer(buffer, 1);
        });
    }
}

impl<'a, const COMMAND_HISTORY_LEN: usize, A: Alarm<'a>, C: ProcessManagementCapability>
    uart::TransmitClient for ProcessConsole<'a, COMMAND_HISTORY_LEN, A, C>
{
    fn transmitted_buffer(
        &self,
        buffer: &'static mut [u8],
        _tx_len: usize,
        _rcode: Result<(), ErrorCode>,
    ) {
        // Reset state now that we no longer have an active transmission on the
        // UART.
        self.tx_buffer.replace(buffer);
        self.tx_in_progress.set(false);

        // Check if we have anything queued up. If we do, let the queue
        // empty.
        let ret = self.handle_queue();
        if ret.ok() == Some(0) || ret.is_err() {
            // The queue was empty or we couldn't print the queue.

            let current_state = self.writer_state.get();
            if current_state != WriterState::Empty {
                self.write_state(current_state);
                return;
            }

            // Check if we just received and echoed a newline character, and
            // therefore need to process the received message.
            if self.execute.get() {
                self.execute.set(false);
                self.read_command();
            }
        }
    }
}

impl<'a, const COMMAND_HISTORY_LEN: usize, A: Alarm<'a>, C: ProcessManagementCapability>
    uart::ReceiveClient for ProcessConsole<'a, COMMAND_HISTORY_LEN, A, C>
{
    fn received_buffer(
        &self,
        read_buf: &'static mut [u8],
        rx_len: usize,
        _rcode: Result<(), ErrorCode>,
        error: uart::Error,
    ) {
        if error == uart::Error::None {
            match rx_len {
                0 => debug!("ProcessConsole had read of 0 bytes"),
                1 => {
                    self.command_buffer.map(|command| {
                        let previous_byte = self.previous_byte.get();
                        self.previous_byte.set(read_buf[0]);
                        let index = self.command_index.get() as usize;
                        if read_buf[0] == ('\n' as u8) || read_buf[0] == ('\r' as u8) {
                            if (previous_byte == ('\n' as u8) || previous_byte == ('\r' as u8))
                                && previous_byte != read_buf[0]
                            {
                                // Reset the sequence, when \r\n is received
                                self.previous_byte.set(0);

                                if COMMAND_HISTORY_LEN > 1 {
                                    self.command_history_index.insert(None);
                                    self.modified_in_history.set(false);

                                    // Clear the unfinished command if the \r\n is received
                                    self.command_history.map(|cmd_arr| {
                                        cmd_arr[0].clear();
                                    });
                                }
                            } else {
                                self.execute.set(true);
                                let _ = self.write_bytes(&['\r' as u8, '\n' as u8]);
                            }
                        } else if read_buf[0] == ('\x08' as u8) || read_buf[0] == ('\x7F' as u8) {
                            if index > 0 {
                                // Backspace, echo and remove last byte
                                // Note echo is '\b \b' to erase
                                let _ = self.write_bytes(&['\x08' as u8, ' ' as u8, '\x08' as u8]);
                                command[index - 1] = EOL;
                                self.command_index.set(index - 1);

                                // Remove last byte from the command in order
                                // not to permit accumulation of the text
                                if COMMAND_HISTORY_LEN > 1 {
                                    self.command_history.map(|cmd_arr| {
                                        (&mut cmd_arr[0]).delete_last_byte();
                                    });
                                }
                            }
                        } else if (COMMAND_HISTORY_LEN > 1)
                            && (read_buf[0] == ESC || self.control_seq_in_progress.get())
                        {
                            // Catch the Up and Down arrow keys
                            if read_buf[0] == ESC {
                                // Signal that a control sequence has started and capture it
                                self.control_seq_in_progress.set(true);
                                self.modified_byte.set(previous_byte);
                            } else if read_buf[0] != ('[' as u8) {
                                // Fetch the index of the last command added to the history
                                if let Some(index) = match read_buf[0] {
                                    // Up arrow case
                                    b'A' => {
                                        let i = match self.command_history_index.extract() {
                                            Some(i) => i + 1,
                                            None => 1,
                                        };
                                        if i >= COMMAND_HISTORY_LEN {
                                            None
                                        } else {
                                            // Check if any command can be displayed
                                            self.command_history
                                                .map(|cmd_arr| match cmd_arr[i].len == 0 {
                                                    true => None,
                                                    false => {
                                                        // Remove last whitespace byte from the command
                                                        // or register a new unfinshed command
                                                        // upon pressing backspace
                                                        match self.modified_byte.get() {
                                                            b' ' => {
                                                                (&mut cmd_arr[0])
                                                                    .delete_last_byte();
                                                            }
                                                            b'\x08' | b'\x7F' => {
                                                                cmd_arr[0].clear();

                                                                let mut command_array =
                                                                    [0; COMMAND_BUF_LEN];
                                                                command_array
                                                                    .copy_from_slice(command);
                                                                cmd_arr[0].write(&command_array);
                                                            }
                                                            _ => {
                                                                self.modified_byte.set(EOL);
                                                            }
                                                        };
                                                        Some(i)
                                                    }
                                                })
                                                .unwrap()
                                        }
                                    }
                                    // Down arrow case
                                    b'B' => match self.command_history_index.extract() {
                                        Some(i) => match i > 0 {
                                            true => {
                                                // Remove last whitespace byte from the command
                                                // or register a new unfinshed command
                                                // upon pressing backspace
                                                self.command_history.map(|cmd_arr| {
                                                    match self.modified_byte.get() {
                                                        b' ' => {
                                                            (&mut cmd_arr[0]).delete_last_byte();
                                                        }
                                                        b'\x08' | b'\x7F' => {
                                                            cmd_arr[0].clear();

                                                            let mut command_array =
                                                                [0; COMMAND_BUF_LEN];
                                                            command_array.copy_from_slice(command);
                                                            cmd_arr[0].write(&command_array);
                                                        }
                                                        _ => {
                                                            self.modified_byte.set(EOL);
                                                        }
                                                    };
                                                });

                                                Some(i - 1)
                                            }
                                            false => None,
                                        },
                                        None => None,
                                    },
                                    _ => None,
                                } {
                                    self.command_history_index.set(index);
                                    self.command_history.map(|cmd_arr| {
                                        let next_command = cmd_arr[index];
                                        let prev_command_len = self.command_index.get();
                                        let next_command_len = next_command.len;

                                        // Clear the displayed command
                                        for _ in 0..prev_command_len {
                                            let _ = self.write_bytes(&[
                                                '\x08' as u8,
                                                ' ' as u8,
                                                '\x08' as u8,
                                            ]);
                                        }

                                        // Display the new command
                                        for i in 0..next_command_len {
                                            let byte = next_command.buf[i];
                                            let _ = self.write_byte(byte);
                                            command[i] = byte;
                                        }

                                        self.modified_in_history.set(true);
                                        self.command_index.set(next_command_len);
                                        command[next_command_len] = EOL;
                                    });
                                };
                                self.control_seq_in_progress.set(false);
                            }
                        } else if index < (command.len() - 1) && read_buf[0] < 128 {
                            // For some reason, sometimes reads return > 127 but no error,
                            // which causes utf-8 decoding failure, so check byte is < 128. -pal

                            // Echo the byte and store it
                            let _ = self.write_byte(read_buf[0]);
                            command[index] = read_buf[0];
                            self.command_index.set(index + 1);
                            command[index + 1] = 0;

                            if COMMAND_HISTORY_LEN > 1 {
                                self.command_history.map(|cmd_arr| {
                                    if self.modified_in_history.get() {
                                        // Copy the last command into the unfinished command

                                        cmd_arr[0].clear();

                                        let mut command_array = [0; COMMAND_BUF_LEN];
                                        command_array.copy_from_slice(command);
                                        cmd_arr[0].write(&command_array);

                                        self.modified_in_history.set(false);
                                    } else {
                                        // Do not save unnecessary white spaces
                                        // between commands
                                        if read_buf[0] != (' ' as u8)
                                            || previous_byte != (' ' as u8)
                                        {
                                            (&mut cmd_arr[0]).insert_byte(read_buf[0]);
                                        }
                                    }
                                });
                            }
                        }
                    });
                }
                _ => debug!(
                    "ProcessConsole issues reads of 1 byte, but receive_complete was length {}",
                    rx_len
                ),
            };
        }
        self.rx_in_progress.set(true);
        let _ = self.uart.receive_buffer(read_buf, 1);
    }
}
