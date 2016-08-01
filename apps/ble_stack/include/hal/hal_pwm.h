/**
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *  http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

#ifndef H_HAL_HAL_PWM_
#define H_HAL_HAL_PWM_

#ifdef __cplusplus
extern "C" {
#endif

#include <inttypes.h>
#include <bsp/bsp_sysid.h>

/* This is an abstract hardware API to Pulse Width Modulators.
 * A Pulse width module produces an output pulse stream with
 * a specified period, and duty cycle.
 */
struct hal_pwm;

/* Initialize a new PWM device with the given system id.
 * Returns negative on error, 0 on success.
 */
struct hal_pwm*
hal_pwm_init(enum system_device_id sysid);

/* gets the underlying clock driving the PWM output. Return value
 * is in Hz. Returns negative on error
 */
int
hal_pwm_get_source_clock_freq(struct hal_pwm *ppwm);

/* gets the resolution of the PWM in bits.  An N-bit PWM can have
 * on values between 0 and 2^bits - 1. Returns negative on error
 */
int
hal_pwm_get_resolution_bits(struct hal_pwm *ppwm);

/* turns off the PWM channel */
int
hal_pwm_disable(struct hal_pwm *ppwm);

/* enables the PWM with duty cycle specified. This duty cycle is
 * a fractional duty cycle where 0 == off, 65535=on, and
 * any value in between is on for fraction clocks and off
 * for 65535-fraction clocks.
 */
int
hal_pwm_enable_duty_cycle(struct hal_pwm *ppwm, uint16_t fraction);

/*
 * This frequency must be between 1/2 the clock frequency and
 * the clock divided by the resolution. NOTE: This may affect
 * other PWM channels.
 */
int
hal_pwm_set_frequency(struct hal_pwm *ppwm, uint32_t freq_hz);

/* NOTE: If you know the resolution and clock frequency, you can
 * compute the period of the PWM Its 2^resolution/clock_freq
 */


#ifdef __cplusplus
}
#endif


#endif /* H_HAL_HAL_PWM_ */
