// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! HC-SR04 Ultrasonic Distance Sensor.
//!
//! Product Link: [HC-SR04 Product Page](https://www.sparkfun.com/products/15569)
//! Datasheet: [HC-SR04 Datasheet](https://www.handsontec.com/dataspecs/HC-SR04-Ultrasonic.pdf)
//!
//! HC-SR04 ultrasonic sensor provides a very low-cost and easy method of distance measurement. It measures distance using sonar,
//! an ultrasonic (well above human hearing) pulse (~40KHz) is transmitted from the unit and distance-to-target is determined by
//! measuring the time required for the echo return. This sensor offers excellent range accuracy and stable readings in an easy-to-use
//! package.

use core::cell::Cell;

use kernel::hil::gpio;
use kernel::hil::sensors::{self, Distance, DistanceClient};
use kernel::hil::time::Alarm;
use kernel::hil::time::{AlarmClient, ConvertTicks};
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

#[derive(Copy, Clone, PartialEq)]
/// Status of the sensor.
pub enum Status {
    /// Sensor is idle.
    Idle,

    /// Sending ultrasonic pulse.
    TriggerPulse,

    /// Interrupt on the rising edge.
    EchoStart,

    /// Interrupt on the falling edge.
    EchoEnd,
}

/// HC-SR04 Ultrasonic Distance Sensor Driver
pub struct HcSr04<'a, A: Alarm<'a>> {
    trig: &'a dyn gpio::Pin,
    echo: &'a dyn gpio::InterruptPin<'a>,
    alarm: &'a A,
    start_time: Cell<u64>,
    state: Cell<Status>,
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
            state: Cell::new(Status::Idle),
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
        if self.state.get() == Status::Idle {
            self.state.set(Status::TriggerPulse);
            self.trig.set();

            // Setting the alarm to send the trigger pulse.
            // According to the HC-SR04 datasheet, a 10 µs pulse should be sufficient
            // to trigger the measurement. However, in practical tests, using this
            // 10 µs value led to inaccurate measurements.
            // We have chosen to use a 1 ms pulse instead because it provides stable
            // operation and accurate measurements, even though it is slightly longer
            // than the datasheet recommendation. While this adds a small delay to the
            // triggering process, it does not significantly affect the overall performance
            // of the sensor.
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
        match self.state.get() {
            Status::TriggerPulse => {
                self.state.set(Status::EchoStart); // Update status to waiting for echo.
                self.echo.enable_interrupts(gpio::InterruptEdge::RisingEdge); // Enable rising edge interrupt on echo pin.
                self.trig.clear(); // Clear the trigger pulse.
                self.alarm.set_alarm(
                    self.alarm.now(),
                    self.alarm.ticks_from_ms(MAX_ECHO_DELAY_MS),
                ); // Set alarm for maximum echo delay.
            }
            // Timeout for echo pulse.
            Status::EchoStart => {
                self.state.set(Status::Idle); // Update status to idle.
                if let Some(distance_client) = self.distance_client.get() {
                    // NOACK indicates that no echo was received within the expected time.
                    distance_client.callback(Err(ErrorCode::NOACK));
                }
            }
            _ => {}
        }
    }
}

impl<'a, A: Alarm<'a>> gpio::Client for HcSr04<'a, A> {
    /// Handle the GPIO interrupt.
    fn fired(&self) {
        // Convert current ticks to microseconds using `ticks_to_us`,
        // which handles the conversion based on the timer frequency.
        let time = self.alarm.ticks_to_us(self.alarm.now()) as u64;
        match self.state.get() {
            Status::EchoStart => {
                let _ = self.alarm.disarm(); // Disarm the alarm.
                self.state.set(Status::EchoEnd); // Update status to waiting for echo end.
                self.echo
                    .enable_interrupts(gpio::InterruptEdge::FallingEdge); // Enable falling edge interrupt on echo pin.
                self.start_time.set(time); // Record start time when echo received.
            }
            Status::EchoEnd => {
                let end_time = time; // Use a local variable for the end time.
                self.state.set(Status::Idle); // Update status to idle.
                let duration = end_time.wrapping_sub(self.start_time.get()) as u32; // Calculate pulse duration.
                if duration > MAX_ECHO_DELAY_MS * 1000 {
                    // If the duration exceeds the maximum distance, return an error indicating invalid measurement.
                    // This means that the object is out of range or no valid echo was received.
                    if let Some(distance_client) = self.distance_client.get() {
                        distance_client.callback(Err(ErrorCode::INVAL));
                    }
                } else {
                    // Calculate distance in millimeters based on the duration of the echo.
                    // The formula for calculating distance is:
                    // Distance = (duration (µs) * SPEED_OF_SOUND (mm/s)) / (2 * 1_000_000), where
                    // - `duration` is the time taken for the echo to travel to the object and back, in microseconds,
                    // - SPEED_OF_SOUND is the speed of sound in air, in millimeters per second.
                    // We divide by 2 because `duration` includes the round-trip time (to the object and back),
                    // and we divide by 1,000,000 to convert from microseconds to seconds.
                    //
                    // To avoid using 64-bit arithmetic (u64), we restructure this equation as:
                    // ((SPEED_OF_SOUND / 1000) * duration) / (2 * 1000).
                    // This rearrangement reduces the scale of intermediate values, keeping them within u32 limits:
                    // - SPEED_OF_SOUND is divided by 1000, reducing it to 343 (in mm/ms), and
                    // - duration remains in microseconds (µs).
                    // The final division by 2000 adjusts for the round trip and scales to the correct unit.
                    //
                    // This form is less intuitive, but it ensures all calculations stay within 32-bit size (u32).
                    // Given the HC-SR04 sensor's maximum `duration` of ~23,000 µs (datasheet limit), this u32 approach
                    // is sufficient for accurate distance calculations without risking overflow.
                    let distance = ((SPEED_OF_SOUND / 1000) * duration) / (2 * 1000);
                    if let Some(distance_client) = self.distance_client.get() {
                        distance_client.callback(Ok(distance));
                    }
                }
            }
            _ => {}
        }
    }
}
