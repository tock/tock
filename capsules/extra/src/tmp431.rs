// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! SyscallDriver for the Texas Instruments TMP431 Thermometer.
//!
//! SMBus Interface
//!
//! Usage
//! -----
//!
//! ```rust
//! let tmp431 = components::tmp431::Tmp431SMBusComponent::new(
//!     thermometer, alarm_mux, board_kernel, driver_num
//! )
//! .finalize(components::tmp431_component_static!(Thermometer, Alarm));
//!
//! let temp = components::temperature::TemperatureComponent::new(
//!    board_kernel,
//!    capsules_extra::temperature::DRIVER_NUM,
//!    tmp431,
//! )
//! .finalize(components::temperature_component_static!(
//!    capsules_extra::tmp431::Tmp431SMBus<Thermometer, VirtualMuxAlarm<Alarm>
//! ));
//! ```
//!

use kernel::{
    grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount},
    hil::{
        i2c, sensors,
        time::{Alarm, AlarmClient, ConvertTicks},
    },
    syscall::{CommandReturn, SyscallDriver},
    utilities::cells::{OptionalCell, TakeCell},
    ErrorCode, ProcessId,
};

#[derive(Clone, Copy, PartialEq, Debug)]
enum CmdScheduled {
    Config,
    OneShot,
    ReadTemperature,
}

/// 8.4 Device Functional Modes
///
/// 8.4.1 Shutdown Mode (SD)
/// The TMP43x shutdown mode allows the user to save maximum power by shutting down all device circuitry other
/// than the serial interface, reducing current consumption to typically less than 3 µA; see Figure 6. Shutdown mode
/// is enabled when the SD bit of the Configuration Register 1 is high; the device shuts down immediately, aborting
/// the current conversion. When SD is low, the device maintains a continuous conversion state.
///
/// 8.4.2 One-Shot Mode
/// When the TMP43x are in shutdown mode (SD = 1 in the Configuration Register 1), a single conversion on both
/// channels is started by writing any value to the One-Shot Start Register, pointer address 0Fh. This write operation
/// starts one conversion; the TMP43x return to shutdown mode when that conversion completes. The value of the
/// data sent in the write command is irrelevant and is not stored by the TMP43x. When the TMP43x are in
/// shutdown mode, an initial 200 ps is required before a one-shot command can be given. (Note: When a shutdown
/// command is issued, the TMP43x shut down immediately, aborting the current conversion.) This wait time only
/// applies to the 200 ps immediately following shutdown. One-shot commands can be issued without delay
/// thereafter.
struct CmdBuf {
    inner: TakeCell<'static, [u8; 2]>,
}

impl CmdBuf {
    const TEMP_LOCAL_READ: u8 = 0x00;
    const CONFIG1_WRITE: u8 = 0x09;
    const ONE_SHOT: u8 = 0x0f;

    const CONFIG1_SD_MASK: u8 = 1 << 6;
    const CONFIG1_RANGE_MASK: u8 = 1 << 2;

    fn new(buf: &'static mut [u8; 2]) -> Self {
        Self {
            inner: TakeCell::new(buf),
        }
    }

    fn replace(&self, buf: &'static mut [u8; 2]) -> Option<&'static mut [u8; 2]> {
        self.inner.replace(buf)
    }

    fn take(&self) -> Option<&'static mut [u8; 2]> {
        self.inner.take()
    }

    fn write_config_and_shutdown(&self) {
        let config = Self::CONFIG1_RANGE_MASK | Self::CONFIG1_SD_MASK;
        self.inner
            .map(|buf| buf.copy_from_slice(&[Self::CONFIG1_WRITE, config]));
    }

    fn write_one_shot(&self) {
        self.inner
            .map(|buf| buf.copy_from_slice(&[Self::ONE_SHOT, 0 /*Value ignored*/]));
    }

    fn write_read_temperature(&self) {
        self.inner
            .map(|buf| buf.copy_from_slice(&[Self::TEMP_LOCAL_READ, 0 /*Value ignored*/]));
    }

    fn read_as_temperature(&self) -> Option<i32> {
        self.inner.map(|temp| {
            let ms_byte = temp[0] as i16 - 64; /* 0x00 is -64 C and 0xff is 191 C */
            let ls_byte = temp[1] as i16;
            let temperature = ((ms_byte << 4) + (ls_byte >> 4)) as i32; /* in range [-64 * 16 + 0; 191 * 16 + 15] = [-1024; 3071] */
            let deci_celc = temperature * 100; /* in range [-102400; 307100] */
            let deci_celc = deci_celc >> 4; /* in range [-102400 / 16; 307100 / 16] = [-6400; 19190] */

            deci_celc
        })
    }
}

#[derive(Default)]
pub struct App {}

pub struct Tmp431SMBus<'a, S: i2c::SMBusDevice, A: Alarm<'a>> {
    smbus_temp: &'a S,
    temperature_client: OptionalCell<&'a dyn sensors::TemperatureClient>,
    buffer: CmdBuf,
    cmd_scheduled: OptionalCell<CmdScheduled>,
    grant: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
    owning_process: OptionalCell<ProcessId>,
    delay: &'a A,
}

impl<'a, S: i2c::SMBusDevice, A: Alarm<'a>> Tmp431SMBus<'a, S, A> {
    #[allow(dead_code)]
    const ENABLE_DELAY_MS: u32 = 17;
    const SETTLING_DELAY_MS: u32 = 17;

    pub fn new(
        smbus_temp: &'a S,
        delay: &'a A,
        grant: Grant<App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>,
        buf: &'static mut [u8; 2],
    ) -> Self {
        Self {
            smbus_temp,
            temperature_client: OptionalCell::empty(),
            buffer: CmdBuf::new(buf),
            cmd_scheduled: OptionalCell::empty(),
            grant,
            owning_process: OptionalCell::empty(),
            delay,
        }
    }

    pub fn initialize(&'a self) {
        self.delay.set_alarm_client(self);
    }

    fn trigger_write_read(&self) -> Result<(), kernel::hil::i2c::Error> {
        // kernel::debug!("TMP431: triggering write-read");
        let res = self
            .buffer
            .take()
            .map_or(Ok(()), |buf| self.smbus_temp.smbus_write_read(buf, 1, 2));
        res.map_err(|(err, buf)| {
            self.buffer.replace(buf.try_into().unwrap());
            err
        })
    }

    fn trigger_write(&self) -> Result<(), kernel::hil::i2c::Error> {
        // kernel::debug!("TMP431: triggering write");
        let res = self
            .buffer
            .take()
            .map_or(Ok(()), |buf| self.smbus_temp.smbus_write(buf, 2));
        res.map_err(|(err, buf)| {
            self.buffer.replace(buf.try_into().unwrap());
            err
        })
    }

    #[allow(dead_code)]
    fn trigger_read(&self) -> Result<(), kernel::hil::i2c::Error> {
        // kernel::debug!("TMP431: triggering read");
        let res = self
            .buffer
            .take()
            .map_or(Ok(()), |buf| self.smbus_temp.smbus_read(buf, 1));
        res.map_err(|(err, buf)| {
            self.buffer.replace(buf.try_into().unwrap());
            err
        })
    }

    /* whip6 algorithm for reading temperature goes as follows (SYNC version):
     * 1. Turn I2C on.
     * 2. EnableDelay (17 ms).
     * 3. Write config (range and shutdown).
     * 4. Write one shot (this triggers one conversion).
     * 5. SettlingDelay (17 ms).
     * 6. Write "read temperature" cmd.
     * 7. Read temperature.
     *
     * So the event-driver version is as follows:
     * - upon delay done:
     *      - EnableDelay -> write config
     *      - SettlingDelay -> write "read temperature" cmd
     * - upon write done:
     *      - config written -> write one shot
     *      - one shot written -> set SettlingDelay
     *      - "write temperature" cmd -> read temperature
     */

    fn read_object_temperature(&self) -> Result<(), ErrorCode> {
        // kernel::debug!("TMP431: requested temperature read");
        self.buffer.write_config_and_shutdown();
        let res = self.trigger_write();
        match res {
            Ok(()) => {
                self.cmd_scheduled.set(CmdScheduled::Config);
            }
            Err(err) => {
                self.temperature_client.map(|client| {
                    client.callback(Err(err.into()));
                });
                self.cmd_scheduled.insert(None);
            }
        };
        res.map_err(ErrorCode::from)
    }
}

impl<'a, S: i2c::SMBusDevice, A: Alarm<'a>> i2c::I2CClient for Tmp431SMBus<'a, S, A> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        // kernel::debug!("TMP431: I2C read completed with buffer {:?} and status: {:?}", buffer, status);
        self.buffer.replace(buffer.try_into().unwrap());
        match self.cmd_scheduled.get() {
            None => panic!("TMP431: BUG: Command completed when no command scheduled!"),
            Some(CmdScheduled::Config) => {
                // write one shot
                self.buffer.write_one_shot();
                match self.trigger_write() {
                    Ok(()) => {
                        self.cmd_scheduled.set(CmdScheduled::OneShot);
                        // kernel::debug!("TMP431: Scheduled OneShot");
                    }
                    Err(err) => {
                        self.temperature_client.map(|client| {
                            client.callback(Err(err.into()));
                        });
                        self.cmd_scheduled.insert(None);
                    }
                }
            }

            Some(CmdScheduled::OneShot) => {
                // set SettlingDelay
                let now = self.delay.now();
                self.delay
                    .set_alarm(now, self.delay.ticks_from_ms(Self::SETTLING_DELAY_MS));
                // kernel::debug!("TMP431: set SettlingDelay");
            }

            Some(CmdScheduled::ReadTemperature) => {
                let values = match status {
                    Ok(()) =>
                    // Convert to centi celsius
                    {
                        self.buffer.read_as_temperature().ok_or(ErrorCode::FAIL)
                        // .inspect(|val| kernel::debug!("TMP431: Read temperature: {}", val))
                    }
                    Err(i2c_error) => Err(i2c_error.into()),
                };
                self.temperature_client.map(|client| {
                    client.callback(values);
                });
                self.owning_process.map(|pid| {
                    let _ = self.grant.enter(pid, |_app, upcalls| {
                        let _ = upcalls.schedule_upcall(0, (values.unwrap_or(0) as usize, 0, 0));
                    });
                });
                self.cmd_scheduled.insert(None);
            }
        }
    }
}

impl<'a, S: i2c::SMBusDevice, A: Alarm<'a>> SyscallDriver for Tmp431SMBus<'a, S, A> {
    fn command(
        &self,
        command_num: usize,
        _data1: usize,
        _data2: usize,
        process_id: ProcessId,
    ) -> CommandReturn {
        if command_num == 0 {
            // Handle this first as it should be returned
            // unconditionally
            return CommandReturn::success();
        }
        // Check if this non-virtualized driver is already in use by
        // some (alive) process
        let match_or_empty_or_nonexistent = self.owning_process.map_or(true, |current_process| {
            self.grant
                .enter(current_process, |_, _| current_process == process_id)
                .unwrap_or(true)
        });
        if match_or_empty_or_nonexistent {
            self.owning_process.set(process_id);
        } else {
            return CommandReturn::failure(ErrorCode::NOMEM);
        }

        match command_num {
            0 => CommandReturn::success(),
            // Check if sensor is correctly connected
            1 => CommandReturn::failure(ErrorCode::NOSUPPORT),
            // Read Ambient Temperature
            2 => CommandReturn::failure(ErrorCode::NOSUPPORT),
            // Read Object Temperature
            3 => {
                if self.cmd_scheduled.get().is_none() {
                    self.read_object_temperature().into()
                } else {
                    CommandReturn::failure(ErrorCode::BUSY)
                }
            }
            // default
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.grant.enter(processid, |_, _| {})
    }
}

impl<'a, S: i2c::SMBusDevice, A: Alarm<'a>> sensors::TemperatureDriver<'a>
    for Tmp431SMBus<'a, S, A>
{
    fn set_client(&self, temperature_client: &'a dyn sensors::TemperatureClient) {
        self.temperature_client.replace(temperature_client);
    }

    fn read_temperature(&self) -> Result<(), ErrorCode> {
        self.read_object_temperature()
    }
}

impl<'a, S: i2c::SMBusDevice, A: Alarm<'a>> AlarmClient for Tmp431SMBus<'a, S, A> {
    fn alarm(&self) {
        // Determine whether it was EnableDelay or SettlingDelay.

        // For now, only SettlingDelay is used.
        // kernel::debug!("TMP431: SettlingDelay fired!");
        self.buffer.write_read_temperature();
        match self.trigger_write_read() {
            Ok(()) => {
                self.cmd_scheduled.set(CmdScheduled::ReadTemperature);
            }
            Err(err) => {
                self.temperature_client.map(|client| {
                    client.callback(Err(err.into()));
                });
                self.cmd_scheduled.insert(None);
            }
        }
    }
}
