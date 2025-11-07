// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use super::State as BusState;
use super::{CYW4343xBus, CYW4343xBusClient, Function, RegLen, Type};
use crate::bus::common;
use crate::utils;
use crate::{backplane_window_bits, reset_and_restore_bufs};
use core::cell::Cell;
use kernel::hil::spi::{SpiMasterClient, SpiMasterDevice};
use kernel::hil::time::ConvertTicks;
use kernel::hil::{gpio, time};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::registers::register_bitfields;
use kernel::{utilities::cells::OptionalCell, ErrorCode};
use task::GspiTask;

/// Max packet size is max payload size + 1 command/status word
pub const MAX_PACKET_SIZE: usize = MAX_PAYLOAD_SIZE + CMD_SIZE;

/// Maximum payload size
///
/// For SPI, 2048 is the maximum due to the length encoding
/// in the command as 11 bits (0 decoded as 2048)
const MAX_PAYLOAD_SIZE: usize = 2048;

/// Word size used for address/registers
pub const WORD_SIZE: usize = 4;
/// Command size is 1 word
const CMD_SIZE: usize = WORD_SIZE;
/// Status size is 1 word
const STATUS_SIZE: usize = WORD_SIZE;

// Bus command encoding
register_bitfields![u32,
    CYW43SPI_CMD [
        COMMAND OFFSET(31) NUMBITS(1) [],
        ACCESS OFFSET(30) NUMBITS(1) [
            IncAddr = 1,
        ],
        FUNCTION_NUM OFFSET(28) NUMBITS(2) [],
        ADDRESS OFFSET(11) NUMBITS(17) [],
        LENGTH OFFSET(0) NUMBITS(11) [],
    ]
];

/// 32-bit LE bus command
const fn cmd32(command: Type, function: Function, address: u32, length: u32) -> u32 {
    CYW43SPI_CMD::COMMAND.val(command as u32).value
        | CYW43SPI_CMD::ACCESS::IncAddr.value
        | CYW43SPI_CMD::FUNCTION_NUM.val(function as u32).value
        | CYW43SPI_CMD::ADDRESS.val(address).value
        | CYW43SPI_CMD::LENGTH.val(length).value
}

/// 16-bit LE bus command
const fn cmd16(command: Type, function: Function, address: u32, length: u32) -> u32 {
    cmd32(command, function, address, length).rotate_left(16)
}

#[derive(Clone, Copy, Debug, Default)]
enum State {
    NotInit,
    Init(u8),
    Irq,
    Write,
    Read,
    #[default]
    Idle,
}

struct Backplane {
    /// The backplane window address for the current backplane rx/tx operation
    curr_window: Cell<u32>,
    /// Backplane window that is in process to be set
    pending_window: OptionalCell<(u32, u8)>,
    /// The maximum packet size for F1 is small (64 bytes), so we need
    /// to store the current offset of the buffer we want to transmit
    transfer_offset: Cell<usize>,
    /// This is true if there is at least one more chunk of the current buffer to be written
    pending: Cell<bool>,
}

impl Backplane {
    fn new() -> Self {
        Self {
            curr_window: Cell::new(0xAAAA_AAAA),
            pending_window: OptionalCell::empty(),
            transfer_offset: Cell::new(0),
            pending: Cell::new(false),
        }
    }
}

/// gSPI interface implementation for the CYW43xx Bus
///
/// The protocol is explained in chapter 4.2 of the datasheet
pub struct CYW4343xSpiBus<'a, S: SpiMasterDevice<'a>, A: kernel::hil::time::Alarm<'a>> {
    gspi: &'a S,
    alarm: &'a A,
    client: OptionalCell<&'a dyn CYW4343xBusClient>,
    /// Backplane (F1) state
    backplane: Backplane,
    /// Inner bus state (initialising, idle or transfering bytes)
    inner_state: Cell<State>,
    /// Bus state from trait
    state: Cell<BusState>,
    /// If there was a register read operation this field stores the jump function and
    /// the starting index for the register
    read: OptionalCell<(fn(u32, &mut u8), u8)>,
    /// Command buffer for reads/status buffer for writes
    extra: OptionalCell<SubSliceMut<'static, u8>>,
    /// Data buffer for read/writes
    data: OptionalCell<SubSliceMut<'static, u8>>,
    /// WLAN buffer
    wlan: OptionalCell<SubSliceMut<'static, u8>>,
    /// Firmware
    fw: &'static [u8],
    /// WLAN read packet length
    len: OptionalCell<usize>,
    /// NVRAM fw blob reference, backplane (F1) address and magic number
    nvram: (&'static [u8], u32, u32),
    /// An interrupt has fired while busy
    irq_fired: Cell<bool>,
}

impl<'a, S: SpiMasterDevice<'a>, A: kernel::hil::time::Alarm<'a>> CYW4343xSpiBus<'a, S, A> {
    pub fn new(
        gspi: &'a S,
        alarm: &'a A,
        extra: &'static mut [u8; CMD_SIZE],
        buffer: &'static mut [u8; MAX_PACKET_SIZE],
        fw: &'static [u8],
        nvram: &'static [u8],
    ) -> Self {
        let nvram_len = nvram.len().div_ceil(4) * 4;
        let nvram_words = nvram_len / 4;
        let nvram_magic = (!nvram_words << 16) | nvram_words;
        Self {
            gspi,
            alarm,
            backplane: Backplane::new(),
            extra: OptionalCell::new(SubSliceMut::new(extra)),
            data: OptionalCell::new(SubSliceMut::new(buffer)),
            client: OptionalCell::empty(),
            inner_state: Cell::new(State::NotInit),
            state: Cell::new(BusState::Idle),
            read: OptionalCell::empty(),
            wlan: OptionalCell::empty(),
            nvram: (
                nvram,
                utils::NVRAM_END - (nvram_len as u32),
                nvram_magic as u32,
            ),
            fw,
            irq_fired: Cell::new(false),
            len: OptionalCell::empty(),
        }
    }
}

impl<'a, S: SpiMasterDevice<'a>, A: kernel::hil::time::Alarm<'a>> CYW4343xSpiBus<'a, S, A> {
    /// Execute an initialisation task (read/write bytes on the bus or set a delay)
    fn do_task(&self, task: GspiTask) -> Result<(), ErrorCode> {
        match task {
            GspiTask::ReadBackplane(cmd, addr, jmp) => {
                self.update_bp_window_or_else(backplane_window_bits!(addr), || {
                    self.read.insert(jmp.map(|jmp| (jmp, WORD_SIZE as _)));
                    self.read(Function::Backplane, cmd)
                })
            }
            GspiTask::WriteBackplane(cmd, addr, val) => {
                self.update_bp_window_or_else(backplane_window_bits!(addr), || self.write(cmd, val))
            }
            GspiTask::Read(fun, cmd, jmp) => {
                let pos = if matches!(fun, Function::Backplane) {
                    // for backplane transfers there is an additional read
                    // so the actual register value is the second word from the buffer
                    4
                } else {
                    0
                };

                self.read.insert(jmp.map(|jmp| (jmp, pos)));
                self.read(fun, cmd)
            }
            GspiTask::Write(cmd, val) => self.write(cmd, val),
            GspiTask::Fw => self.write_bp(utils::RAM_BASE_ADDR, self.fw),
            GspiTask::Nvram => self.write_bp(self.nvram.1, self.nvram.0),
            GspiTask::NvramMagic => self
                .update_bp_window_or_else(backplane_window_bits!(utils::NVRAM_END), || {
                    self.write(NVRAM_MAGIC_CMD, self.nvram.2)
                }),
            GspiTask::WaitMs(ms) => {
                self.alarm
                    .set_alarm(self.alarm.now(), self.alarm.ticks_from_ms(ms));
                Ok(())
            }
        }
    }
}

impl<'a, S: SpiMasterDevice<'a>, A: kernel::hil::time::Alarm<'a>> CYW4343xSpiBus<'a, S, A> {
    /// Write bytes to a backplane memory region from a base address and base buffer.
    /// As this is likely to be done in chunks of MAX_SPI_BP_CHUNK_SIZE bytes,
    /// an inner offset will be kept to be added to the base_address.
    fn write_bp(&self, base_address: u32, base_buffer: &[u8]) -> Result<(), ErrorCode> {
        let offset = self.backplane.transfer_offset.get();
        let address = base_address + offset as u32;

        // Either update the backplane window or write a chunk of the buffer
        self.update_bp_window_or_else(backplane_window_bits!(address), || {
            let Some((mut extra, mut buf)) = Option::zip(self.extra.take(), self.data.take())
            else {
                return Err(ErrorCode::NOMEM);
            };

            let window_offset = address & utils::BACKPLANE_ADDRESS_MASK;
            let window_remaining = (utils::BACKPLANE_WINDOW_SIZE - window_offset) as usize;

            // chunk size is the min of:
            // - the remaining length of packet
            // - 64 bytes
            // - the remaining length of the current window (so it doesnt wrap to the start of
            // window)
            let len = (base_buffer.len() - offset)
                .min(utils::MAX_SPI_BP_CHUNK_SIZE)
                .min(window_remaining);

            let cmd = cmd32(Type::Write, Function::Backplane, window_offset, len as u32);

            // We want to write cmd + len bytes?
            buf.slice(0..CMD_SIZE + len.div_ceil(4) * 4);
            extra.slice(0..STATUS_SIZE);

            buf.as_mut_slice()[0..CMD_SIZE].copy_from_slice(&cmd.to_le_bytes());
            buf.as_mut_slice()[CMD_SIZE..][..len].copy_from_slice(&base_buffer[offset..][..len]);

            self.gspi
                .read_write_bytes(buf, Some(extra))
                .map_err(|(err, mut data, extra)| {
                    let mut extra = extra.unwrap();
                    reset_and_restore_bufs!(self, extra, data);
                    err
                })?;

            // This is not the last chunk to be written
            // NOTE: Using `last` instead of `pending` could be more readable
            // let last = len + offset >= base_buffer.len()
            let pending = len + offset < base_buffer.len();

            self.backplane.pending.set(pending);
            self.backplane
                .transfer_offset
                .set(if pending { offset + len } else { 0 });
            Ok(())
        })
    }

    /// Read a word
    fn read(&self, fun: Function, cmd: u32) -> Result<(), ErrorCode> {
        let Some((mut extra, mut data)) = Option::zip(self.extra.take(), self.data.take()) else {
            return Err(ErrorCode::NOMEM);
        };

        extra.slice(..CMD_SIZE);
        let mut size = STATUS_SIZE + WORD_SIZE;
        if let Function::Backplane = fun {
            // For backplane functions we read an extra word
            size += WORD_SIZE;
        }
        data.slice(0..size);
        extra.as_mut_slice()[0..CMD_SIZE].copy_from_slice(&cmd.to_le_bytes());

        self.gspi
            .read_write_bytes(extra, Some(data))
            .map_err(|(err, mut extra, data)| {
                let mut data = data.unwrap();
                reset_and_restore_bufs!(self, extra, data);
                err
            })
    }

    /// Write a word
    fn write(&self, cmd: u32, val: u32) -> Result<(), ErrorCode> {
        let Some((mut extra, mut buffer)) = Option::zip(self.extra.take(), self.data.take()) else {
            return Err(ErrorCode::NOMEM);
        };

        buffer.slice(..CMD_SIZE + WORD_SIZE);
        extra.slice(..STATUS_SIZE);

        buffer.as_mut_slice()[..CMD_SIZE].copy_from_slice(&cmd.to_le_bytes());
        buffer.as_mut_slice()[CMD_SIZE..][..WORD_SIZE].copy_from_slice(&val.to_le_bytes());

        self.gspi
            .read_write_bytes(buffer, Some(extra))
            .map_err(|(err, mut data, extra)| {
                let mut extra = extra.unwrap();
                reset_and_restore_bufs!(self, extra, data);
                err
            })
    }

    /// Update the backplane window
    fn update_bp_window_or_else(
        &self,
        window: u32,
        f: impl FnOnce() -> Result<(), ErrorCode>,
    ) -> Result<(), ErrorCode> {
        // FIXME: I think this is very hard to read
        let update_byte = |idx: u8| {
            let extract = |val: u32| -> u8 { (val >> ((idx as u32) * 8)) as u8 };
            let extr = extract(window);
            let curr = self.backplane.curr_window.get();
            let extr_curr = extract(curr);
            (extr != extr_curr).then(|| {
                (
                    cmd32(
                        Type::Write,
                        Function::Backplane,
                        utils::REG_BACKPLANE_BACKPLANE_ADDRESS_LOW + idx as u32,
                        RegLen::Byte as _,
                    ),
                    extr,
                )
            })
        };

        // If we were updating a byte from the window, start from that - 1
        let byte = self.backplane.pending_window.take().map(|(_, b)| b - 1);
        // If not, start from the highest byte
        let byte = byte.unwrap_or(2) as _;

        for b in (0..=byte).rev() {
            if let Some((cmd, val)) = update_byte(b) {
                self.backplane.pending_window.set((window, b));
                return self.write(cmd, val as u32);
            }
        }

        self.backplane.curr_window.set(window);
        f()
    }
}

impl<'a, S: SpiMasterDevice<'a>, A: kernel::hil::time::Alarm<'a>> CYW4343xBus<'a>
    for CYW4343xSpiBus<'a, S, A>
{
    fn set_client(&self, client: &'a dyn CYW4343xBusClient) {
        self.client.set(client);
    }

    fn state(&self) -> Result<BusState, kernel::ErrorCode> {
        if let State::NotInit = self.inner_state.get() {
            return Err(ErrorCode::BUSY);
        }
        Ok(self.state.get())
    }

    fn init(&self) -> Result<(), ErrorCode> {
        let State::NotInit = self.inner_state.get() else {
            return Err(ErrorCode::ALREADY);
        };

        const CMD: u32 = cmd16(
            Type::Read,
            Function::Bus,
            utils::REG_BUS_TEST_RO,
            RegLen::Word as _,
        );

        self.read(Function::Bus, CMD)?;
        self.inner_state.set(State::Init(0));
        self.read.set((common::eq::<0xBEADFEED, 1, 0>, 0));
        Ok(())
    }

    fn write_bytes(
        &self,
        mut buffer: SubSliceMut<'static, u8>,
    ) -> Result<(), (kernel::ErrorCode, SubSliceMut<'static, u8>)> {
        let Some((mut data, mut extra)) = Option::zip(self.data.take(), self.extra.take()) else {
            return Err((ErrorCode::NOMEM, buffer));
        };

        let mut total_len = (buffer.len() + 3) & !3;
        if total_len > MAX_PAYLOAD_SIZE {
            return Err((ErrorCode::NOMEM, buffer));
        } else if total_len == MAX_PACKET_SIZE {
            total_len = 0;
        }

        // Construct command
        let cmd = cmd32(Type::Write, Function::Wlan, 0x0, total_len as _);

        let slice = data.as_mut_slice();
        slice[..CMD_SIZE].copy_from_slice(&cmd.to_le_bytes());
        slice[CMD_SIZE..][..buffer.len()].copy_from_slice(buffer.as_mut_slice());

        extra.slice(..STATUS_SIZE);
        data.slice(..CMD_SIZE + total_len);

        if let Err((err, mut data, extra)) = self.gspi.read_write_bytes(data, Some(extra)) {
            let mut extra = extra.unwrap();
            reset_and_restore_bufs!(self, data, extra);
            Err((err, buffer))
        } else {
            self.inner_state.set(State::Write);
            self.wlan.set(buffer);
            Ok(())
        }
    }

    fn read_bytes(
        &self,
        mut buffer: SubSliceMut<'static, u8>,
        len: usize,
    ) -> Result<(), (kernel::ErrorCode, SubSliceMut<'static, u8>)> {
        let Some((mut extra, mut data)) = Option::zip(self.extra.take(), self.data.take()) else {
            return Err((ErrorCode::NOMEM, buffer));
        };

        let total_len = (len + 3) & !3;
        if total_len > MAX_PACKET_SIZE {
            return Err((ErrorCode::NOMEM, buffer));
        }

        let cmd = cmd32(
            Type::Read,
            Function::Wlan,
            0,
            if total_len == MAX_PAYLOAD_SIZE {
                0
            } else {
                total_len as _
            },
        );

        extra.slice(..CMD_SIZE);
        extra[..CMD_SIZE].copy_from_slice(&cmd.to_le_bytes());
        data.slice(0..total_len + STATUS_SIZE);

        if let Err((err, mut extra, data)) = self.gspi.read_write_bytes(extra, Some(data)) {
            let mut data = data.unwrap();
            reset_and_restore_bufs!(self, data, extra);
            Err((err, buffer))
        } else {
            buffer.slice(0..len);
            self.wlan.set(buffer);

            self.len.set(len);
            self.inner_state.set(State::Read);
            Ok(())
        }
    }
}

/// Command for reading the interrupt register
pub(super) const IRQ_CAUSE_CMD: u32 = cmd32(
    Type::Read,
    Function::Bus,
    utils::REG_BUS_INTERRUPT,
    RegLen::HalfWord as _,
);

/// Command for writing the NVRAM magic number
pub(super) const NVRAM_MAGIC_CMD: u32 = cmd32(
    Type::Write,
    Function::Backplane,
    utils::NVRAM_END & utils::BACKPLANE_ADDRESS_MASK | utils::BACKPLANE_WINDOW_SIZE,
    RegLen::Word as _,
);

impl<'a, S: SpiMasterDevice<'a>, A: kernel::hil::time::Alarm<'a>> SpiMasterClient
    for CYW4343xSpiBus<'a, S, A>
{
    fn read_write_done(
        &self,
        write_buffer: SubSliceMut<'static, u8>,
        read_buffer: Option<SubSliceMut<'static, u8>>,
        rval: Result<usize, ErrorCode>,
    ) {
        let rval = rval.map(|_| ());
        let read_buffer = read_buffer.unwrap();

        let (packet_available, packet_len) = {
            let end = &read_buffer[read_buffer.len() - STATUS_SIZE..];
            let status = u32::from_le_bytes([end[0], end[1], end[2], end[3]]);

            // The status word indicates whether a F2 (WLAN) packet is available
            // + contains the length of the available packet
            (
                status & utils::STATUS_F2_PKT_AVAILABLE != 0,
                (status & utils::STATUS_F2_PKT_LEN_MASK) >> utils::STATUS_F2_PKT_LEN_SHIFT,
            )
        };

        let irq_fired = self.irq_fired.take() && !packet_available;

        self.state.set(if packet_available {
            BusState::Available(packet_len as _)
        } else if irq_fired {
            BusState::Incoming
        } else {
            BusState::Idle
        });

        // Separate command/status and data buffer
        let (mut extra, mut data) = if write_buffer.len() == CMD_SIZE {
            (write_buffer, read_buffer)
        } else {
            (read_buffer, write_buffer)
        };

        match self.inner_state.take() {
            State::Init(mut idx) => {
                if let Err(err) = rval {
                    reset_and_restore_bufs!(self, extra, data);
                    self.client.map(|client| client.init_done(Err(err)));
                    self.inner_state.set(State::NotInit);
                    return;
                }

                if let Some((window, byte)) = self.backplane.pending_window.get() {
                    if byte == 0 {
                        self.backplane.curr_window.set(window);
                        self.backplane.pending_window.clear();
                    }
                } else if !self.backplane.pending.take() {
                    if let Some((jmp, pos)) = self.read.take() {
                        let val = &data[pos as usize..];
                        let val = u32::from_le_bytes([val[0], val[1], val[2], val[3]]);
                        jmp(val, &mut idx)
                    } else {
                        idx += 1
                    }
                }

                reset_and_restore_bufs!(self, extra, data);
                if (idx as usize) < init::OPS.len() {
                    self.do_task(init::OPS[idx as usize]).map_or_else(
                        |err| {
                            self.inner_state.set(State::NotInit);
                            self.client.map(|client| client.init_done(Err(err)));
                        },
                        |()| self.inner_state.set(State::Init(idx)),
                    )
                } else {
                    self.client.map(|client| {
                        client.init_done(Ok(()));
                    });
                }
            }
            State::Write => {
                let Some(wlan) = self.wlan.take() else {
                    return;
                };

                reset_and_restore_bufs!(self, extra, data);
                self.client.map(|client| {
                    if irq_fired && self.read(Function::Bus, IRQ_CAUSE_CMD).is_ok() {
                        self.inner_state.set(State::Irq);
                    }
                    client.write_bytes_done(wlan, rval);
                });
            }
            State::Read => {
                let Some(mut wlan) = self.wlan.take() else {
                    return;
                };
                let len = wlan.len();
                wlan.as_mut_slice()
                    .copy_from_slice(&data.as_mut_slice()[..len]);

                reset_and_restore_bufs!(self, extra, data);
                self.client.map(|client| {
                    if irq_fired && self.read(Function::Bus, IRQ_CAUSE_CMD).is_ok() {
                        self.inner_state.set(State::Irq);
                    }
                    client.read_bytes_done(wlan, rval);
                });
            }
            State::Irq => {
                let irq = u16::from_le_bytes([data[0], data[1]]);
                let pending = irq & utils::IRQ_F2_PACKET_AVAILABLE as u16 != 0;

                reset_and_restore_bufs!(self, extra, data);
                self.client.map(|client| {
                    client.packet_available(if pending || packet_available {
                        packet_len as _
                    } else {
                        0
                    })
                });
            }
            _ => unreachable!(),
        }
    }
}

impl<'a, S: SpiMasterDevice<'a>, A: kernel::hil::time::Alarm<'a>> time::AlarmClient
    for CYW4343xSpiBus<'a, S, A>
{
    fn alarm(&self) {
        let State::Init(mut idx) = self.inner_state.get() else {
            return;
        };

        idx += 1;

        self.do_task(init::OPS[idx as usize]).map_or_else(
            |err| {
                self.client.map(|client| client.init_done(Err(err)));
                self.inner_state.set(State::NotInit);
            },
            |()| self.inner_state.set(State::Init(idx)),
        )
    }
}

// Implementation of the `gpio` Client trait.
// The WiFi chip should signal on the interrupt line when a WLAN (F2) packet is ready to be read.
impl<'a, S: SpiMasterDevice<'a>, A: kernel::hil::time::Alarm<'a>> gpio::Client
    for CYW4343xSpiBus<'a, S, A>
{
    fn fired(&self) {
        if let State::Init(_) | State::NotInit = self.inner_state.get() {
            return;
        }

        if let State::Idle = self.inner_state.get() {
            if self.read(Function::Bus, IRQ_CAUSE_CMD).is_ok() {
                self.inner_state.set(State::Irq);
                self.state.set(BusState::Incoming);
            }
        } else {
            self.irq_fired.set(true)
        }
    }
}

mod init {
    use crate::bus;
    use crate::utils;
    use bus::common;
    use bus::spi::{bus_init, wakeup};
    use bus::RegLen::Word;

    macro_rules! copy_from_arr {
        (task => $curr:expr, $to:expr, $from:expr) => {
            let mut __idx = 0;
            while __idx < $from.len() {
                $to[$curr + __idx] = GspiTask::from($from[__idx]);
                __idx += 1;
            }
            $curr += __idx;
        };

        ($curr:expr, $to:expr, $from:expr) => {
            let mut __idx = 0;
            while __idx < $from.len() {
                $to[$curr + __idx] = $from[__idx];
                __idx += 1;
            }
            $curr += __idx;
        };
    }
    use super::task::GspiTask;
    pub(crate) use copy_from_arr;

    pub(crate) static OPS: [GspiTask; 55] = const {
        let mut curr = 0;
        let mut bytes = [GspiTask::WaitMs(0);
            bus_init::OPS.len()
                + WLAN_DISABLE.len()
                + SOCRAM_DISABLE.len()
                + SOCRAM_RESET.len()
                + 5
                + WLAN_RESET.len()
                + wakeup::OPS.len()];
        // 1. Bus init
        copy_from_arr!(curr, bytes, bus_init::OPS); // 0..13

        // 2. Wlan core disable
        const WLAN_DISABLE: [common::BackplaneTask; 7] =
            common::core_disable::ops::<{ utils::WLAN_ARM_CORE_BASE_ADDR }>();
        copy_from_arr!(task => curr, bytes, WLAN_DISABLE); // 13..20

        // 3. SoC RAM core disable
        const SOCRAM_DISABLE: [common::BackplaneTask; 7] =
            common::core_disable::ops::<{ utils::SOCRAM_CORE_BASE_ADDR }>();
        copy_from_arr!(task => curr, bytes, SOCRAM_DISABLE); // 20..27

        // 4. SoC RAM core reset
        const SOCRAM_RESET: [common::BackplaneTask; 7] =
            common::core_reset::ops::<{ utils::SOCRAM_CORE_BASE_ADDR }>();
        copy_from_arr!(task => curr, bytes, SOCRAM_RESET); // 27..34

        // 5. Disable remap
        bytes[curr] = GspiTask::write_bp(0x18004000 + 0x10, Word, 3); // 34
        bytes[curr + 1] = GspiTask::write_bp(0x18004000 + 0x44, Word, 0); // 35

        // 6. Load firmware, load nvram, load nvram magic
        bytes[curr + 2] = GspiTask::Fw; // 36
        bytes[curr + 3] = GspiTask::Nvram; // 37
        bytes[curr + 4] = GspiTask::NvramMagic; // 38
        curr += 5;

        // 7. WLAN core reset
        const WLAN_RESET: [common::BackplaneTask; 7] =
            common::core_reset::ops::<{ utils::WLAN_ARM_CORE_BASE_ADDR }>();
        copy_from_arr!(task => curr, bytes, WLAN_RESET); // 39..46

        // 8. Wakeup sequence
        copy_from_arr!(curr, bytes, wakeup::OPS); // 46..55
        let _ = curr;

        bytes
    };
}

mod bus_init {
    use super::task::GspiTask;
    use crate::{bus, utils};
    use bus::common::{eq, mask};
    use bus::Function::{Backplane as Bp, Bus};
    use bus::RegLen::{Byte, Word};

    pub(super) static OPS: [GspiTask; 12] = [
        // First test
        GspiTask::read16(
            Bus,
            utils::REG_BUS_TEST_RO,
            Word,
            Some(eq::<0xBEADFEED, 1, 0>),
        ),
        GspiTask::write16(Bus, utils::REG_BUS_TEST_RW, Word, 0x12345678),
        GspiTask::read16(
            Bus,
            utils::REG_BUS_TEST_RW,
            Word,
            Some(eq::<0x56781234, 1, 0>),
        ),
        // Configure bus
        GspiTask::write16(Bus, utils::REG_BUS_CTRL, Word, utils::CONFIG_DATA),
        // Second test
        GspiTask::read32(
            Bus,
            utils::REG_BUS_TEST_RO,
            Word,
            Some(eq::<0xFEEDBEAD, 1, 0>),
        ),
        // Interrupts
        GspiTask::write32(
            Bus,
            utils::REG_BUS_INTERRUPT,
            Byte,
            utils::INTR_STATUS_RESET,
        ),
        GspiTask::write32(
            Bus,
            utils::REG_BUS_INTERRUPT_ENABLE,
            Byte,
            utils::INTR_ENABLE_RESET,
        ),
        GspiTask::write32(
            Bp,
            utils::REG_BACKPLANE_CHIP_CLOCK_CSR,
            Byte,
            utils::BACKPLANE_ALP_AVAIL_REQ as u32,
        ),
        GspiTask::write32(Bp, utils::REG_BACKPLANE_FUNCTION2_WATERMARK, Byte, 0x10),
        GspiTask::read32(Bp, utils::REG_BACKPLANE_FUNCTION2_WATERMARK, Byte, None),
        GspiTask::read32(
            Bp,
            utils::REG_BACKPLANE_CHIP_CLOCK_CSR,
            Byte,
            Some(mask::<{ utils::BACKPLANE_ALP_AVAIL }, 0, 1>),
        ),
        GspiTask::write32(Bp, utils::REG_BACKPLANE_CHIP_CLOCK_CSR, Byte, 0x0),
    ];
}

mod wakeup {
    use super::task::GspiTask;
    use crate::{bus, utils};
    use bus::common::mask;
    use bus::Function::{Backplane as Bp, Bus};
    use bus::RegLen::{Byte, HalfWord, Word};

    pub(super) static OPS: [GspiTask; 10] = [
        GspiTask::WaitMs(30),
        GspiTask::read32(
            Bp,
            utils::REG_BACKPLANE_CHIP_CLOCK_CSR,
            Byte,
            Some(mask::<0x80, 0, 1>),
        ),
        GspiTask::write_bp(
            utils::SDIOD_CORE_BASE_ADDRESS + utils::SDIO_INT_HOST_MASK,
            Word,
            utils::I_HMB_SW_MASK,
        ),
        GspiTask::write32(
            Bus,
            utils::REG_BUS_INTERRUPT_ENABLE,
            HalfWord,
            utils::IRQ_F2_PACKET_AVAILABLE,
        ),
        GspiTask::write32(
            Bp,
            utils::REG_BACKPLANE_FUNCTION2_WATERMARK,
            Byte,
            utils::SPI_F2_WATERMARK,
        ),
        GspiTask::read32(
            Bus,
            utils::REG_BUS_STATUS,
            Word,
            Some(mask::<{ utils::STATUS_F2_RX_READY }, 0, 1>),
        ),
        GspiTask::write32(Bp, utils::REG_BACKPLANE_PULL_UP, Byte, 0x0),
        GspiTask::read32(Bp, utils::REG_BACKPLANE_PULL_UP, Byte, None),
        GspiTask::write32(Bp, utils::REG_BACKPLANE_CHIP_CLOCK_CSR, Byte, 0x10),
        GspiTask::read32(
            Bp,
            utils::REG_BACKPLANE_CHIP_CLOCK_CSR,
            Byte,
            Some(mask::<0x80, 0, 1>),
        ),
    ];
}

mod task {
    use super::{cmd16, cmd32};
    use crate::{bus, utils};
    use bus::{common, Function, RegLen, Type};

    type Cmd = u32;
    type Addr = u32;

    /// GSPI task
    #[derive(Clone, Copy, Debug)]
    pub(super) enum GspiTask {
        Read(Function, Cmd, Option<fn(u32, &mut u8) -> ()>),
        Write(Cmd, u32),
        ReadBackplane(Cmd, Addr, Option<fn(u32, &mut u8) -> ()>),
        WriteBackplane(Cmd, Addr, u32),
        WaitMs(u32),
        Fw,
        Nvram,
        NvramMagic,
    }

    impl GspiTask {
        pub(super) const fn from(value: common::BackplaneTask) -> Self {
            match value {
                common::BackplaneTask::Read(len, addr, jmp) => Self::read_bp(addr, len, jmp),
                common::BackplaneTask::Write(len, addr, val) => Self::write_bp(addr, len, val),
                common::BackplaneTask::WaitMs(ms) => Self::WaitMs(ms),
            }
        }

        pub(super) const fn read16(
            fun: Function,
            addr: u32,
            len: RegLen,
            jmp: Option<fn(u32, &mut u8) -> ()>,
        ) -> Self {
            Self::Read(fun, cmd16(Type::Read, fun, addr, len as _), jmp)
        }

        pub(super) const fn write16(fun: Function, addr: u32, len: RegLen, mut val: u32) -> Self {
            if let RegLen::Word = len {
                val = val.rotate_left(16);
            }
            Self::Write(cmd16(Type::Write, fun, addr, len as _), val)
        }

        pub(super) const fn read32(
            fun: Function,
            addr: u32,
            len: RegLen,
            jmp: Option<fn(u32, &mut u8)>,
        ) -> Self {
            Self::Read(fun, cmd32(Type::Read, fun, addr, len as _), jmp)
        }

        pub(super) const fn write32(fun: Function, addr: u32, len: RegLen, val: u32) -> Self {
            Self::Write(cmd32(Type::Write, fun, addr, len as _), val)
        }

        pub(super) const fn read_bp(addr: u32, len: RegLen, jmp: Option<fn(u32, &mut u8)>) -> Self {
            let mut cmd_addr = addr & utils::BACKPLANE_ADDRESS_MASK;
            if let RegLen::Word = len {
                cmd_addr |= utils::BACKPLANE_WINDOW_SIZE
            }

            Self::ReadBackplane(
                cmd32(Type::Read, Function::Backplane, cmd_addr, len as _),
                addr,
                jmp,
            )
        }

        pub(super) const fn write_bp(addr: u32, len: RegLen, val: u32) -> Self {
            assert!(addr % 4 == 0);

            let mut cmd_addr = addr & utils::BACKPLANE_ADDRESS_MASK;
            if let RegLen::Word = len {
                cmd_addr |= utils::BACKPLANE_WINDOW_SIZE
            }

            Self::WriteBackplane(
                cmd32(Type::Write, Function::Backplane, cmd_addr, len as _),
                addr,
                val,
            )
        }
    }
}
