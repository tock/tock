// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::cell::Cell;

use kernel::hil::gpio::{Client, InterruptEdge};
use kernel::hil::sensors::{self, Distance, DistanceClient};
use kernel::hil::time::{Alarm, Ticks, Time};
use kernel::hil::time::{AlarmClient, ConvertTicks, Frequency};
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

/// Maximum duration for the echo pulse to be measured in milliseconds.
// As specified in the datasheet:
// https://www.handsontec.com/dataspecs/HC-SR04-Ultrasonic.pdf,
// the maximum time for the echo pulse to return is around 23 milliseconds
// for a maximum distance of approximately 4 meters under standard temperature
// and pressure conditions, but we use 38 milliseconds to account for variations
// in real-world conditions. We use a slightly higher the value to account for
// possible variations in measurement.
pub const MAX_ECHO_DELAY_MS: u32 = 50;

/// Speed of sound in air in mm/s.
// The speed of sound is approximately 343 meters per second, which
// translates to 343,000 millimeters per second. This value is used
// to calculate the distance based on the time it takes for the echo
// to return.
pub const SPEED_OF_SOUND: u32 = 343000;

/// Syscall driver number.
use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::Distance as usize;

#[derive(Copy, Clone, PartialEq)]
/// Status of the sensor.
pub enum Status {
    Idle,         // Sensor is idle.
    TriggerPulse, // Sending ultrasonic pulse.
    EchoStart,    // Interrupt on the rising edge.
    EchoEnd,      // Interrupt on the falling edge.
}

/// HC-SR04 Ultrasonic Distance Sensor Driver
pub struct HcSr04<'a, A: Alarm<'a>> {
    trig: &'a dyn kernel::hil::gpio::Pin,
    echo: &'a dyn kernel::hil::gpio::InterruptPin<'a>,
    alarm: &'a A,
    start_time: Cell<u64>,
    end_time: Cell<u64>,
    status: Cell<Status>,
    distance_client: OptionalCell<&'a dyn sensors::DistanceClient>,
}

impl<'a, A: Alarm<'a>> HcSr04<'a, A> {
    /// Create a new HC-SR04 driver.
    pub fn new(
        trig: &'a dyn kernel::hil::gpio::Pin,
        echo: &'a dyn kernel::hil::gpio::InterruptPin<'a>,
        alarm: &'a A,
    ) -> HcSr04<'a, A> {
        // Setup and return struct.
        HcSr04 {
            trig,
            echo,
            alarm,
            start_time: Cell::new(0),
            end_time: Cell::new(0),
            status: Cell::new(Status::Idle),
            distance_client: OptionalCell::empty(),
        }
    }
}

impl<'a, A: Alarm<'a>> Distance<'a> for HcSr04<'a, A> {
    /// Set the client for distance measurement results.
    fn set_client(&self, distance_client: &'a dyn DistanceClient) {
        self.distance_client.set(distance_client);
    }
    /// Start a distance measurement.
    fn read_distance(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.status.set(Status::TriggerPulse);
            self.trig.set(); // Send the trigger pulse.
            self.alarm
                .set_alarm(self.alarm.now(), self.alarm.ticks_from_ms(1));
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }
    /// Get the maximum distance the sensor can measure in mm
    fn get_maximum_distance(&self) -> u32 {
        // The maximum distance is determined by the maximum pulse width the sensor can detect.
        // As specified in the datasheet: https://www.handsontec.com/dataspecs/HC-SR04-Ultrasonic.pdf,
        // the maximum measurable distance is approximately 4 meters.
        // Convert this to millimeters.
        4000
    }
    /// Get the minimum distance the sensor can measure in mm.
    fn get_minimum_distance(&self) -> u32 {
        // The minimum distance is determined by the minimum pulse width the sensor can detect.
        // As specified in the datasheet: https://www.handsontec.com/dataspecs/HC-SR04-Ultrasonic.pdf,
        // the minimum measurable distance is approximately 2 cm.
        // Convert this to millimeters.
        20
    }
}

impl<'a, A: Alarm<'a>> AlarmClient for HcSr04<'a, A> {
    /// Handle the alarm event.
    fn alarm(&self) {
        match self.status.get() {
            Status::TriggerPulse => {
                self.status.set(Status::EchoStart); // Update status to waiting for echo.
                self.echo.enable_interrupts(InterruptEdge::RisingEdge); // Enable rising edge interrupt on echo pin.
                self.trig.clear(); // Clear the trigger pulse.
                self.alarm.set_alarm(
                    self.alarm.now(),
                    self.alarm.ticks_from_ms(MAX_ECHO_DELAY_MS),
                ); // Set alarm for maximum echo delay.
            }
            // Timeout for echo pulse.
            Status::EchoStart => {
                self.status.set(Status::Idle); // Update status to idle.
                if let Some(distance_client) = self.distance_client.get() {
                    distance_client.callback(Err(ErrorCode::FAIL));
                }
            }
            _ => {}
        }
    }
}

impl<'a, A: Alarm<'a>> Client for HcSr04<'a, A> {
    /// Handle the GPIO interrupt.
    fn fired(&self) {
        let frequency = <A as Time>::Frequency::frequency();
        let ticks = self.alarm.now().into_u32();
        // Convert ticks to microseconds.
        // Microseconds = (ticks * 1_000_000) / frequency
        let time = (ticks as u64 * 1_000_000) / frequency as u64;
        match self.status.get() {
            Status::EchoStart => {
                let _ = self.alarm.disarm(); // Disarm the alarm.
                self.status.set(Status::EchoEnd); // Update status to waiting for echo end.
                self.echo.enable_interrupts(InterruptEdge::FallingEdge); // Enable falling edge interrupt on echo pin.
                self.start_time.set(time); // Record start time when echo received.
            }
            Status::EchoEnd => {
                self.end_time.set(time); // Record end time when echo ends.
                self.status.set(Status::Idle); // Update status to idle.
                let duration = self.end_time.get().wrapping_sub(self.start_time.get()) as u32; // Calculate pulse duration.
                if duration > self.alarm.ticks_from_ms(MAX_ECHO_DELAY_MS).into_u32() {
                    // If duration exceeds the maximum distance, return an error.
                    if let Some(distance_client) = self.distance_client.get() {
                        distance_client.callback(Err(ErrorCode::INVAL));
                    }
                } else {
                    // Calculate distance in milimeters based on the duration of the echo.
                    // The formula for calculating distance is:
                    // Distance = (duration (µs) * SPEED_OF_SOUND (mm/s)) / (2 * 1_000_000), where
                    // duration is the time taken for the echo to travel to the object and back, in microseconds,
                    // SPEED_OF_SOUND is the speed of sound in air, in millimeters per second.
                    // We divide by 2 because the duration includes the round trip time (to the object and back) and
                    // we divide by 1_000_000 to convert the duration from microseconds to seconds.
                    let distance = duration as u32 * SPEED_OF_SOUND as u32 / (2 * 1_000_000) as u32;
                    if let Some(distance_client) = self.distance_client.get() {
                        distance_client.callback(Ok(distance));
                    }
                }
            }
            _ => {}
        }
    }
}
