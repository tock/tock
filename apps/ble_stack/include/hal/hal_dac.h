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

#ifndef HAL_DAC_H_
#define HAL_DAC_H_

#ifdef __cplusplus
extern "C" {
#endif

/* for the pin descriptor enum */
#include <bsp/bsp_sysid.h>

/* This is the device for a Digital to Analog Converter (DAC).
 * The application using the DAC device
 * does not need to know the definition of this device and can operate
 * with a pointer to this device.  you can get/build device pointers in the
 * BSP
 *
 * NOTE: You can also use PWM devices to simulate analog output.
 * These are defined in hal_pwm.h
 */
struct hal_dac;

/* initialize the DAC on the corresponding BSP device. Returns a pointer
 * to the DAC object to use for the methods below. Returns NULL on
 * error
 */
struct hal_dac *
hal_dac_init(enum system_device_id sysid);

/*
 * write the DAC corresponding to sysid in your system
 * and enables the DAC.  Return 0 on success negative on failures. If you
 * write a value larger than the DAC size, it will get truncated to the
 * maximum DAC value but the write will succeed.
 */
int
hal_dac_write(struct hal_dac *pdac, int val);

/*
 * Gets the current value that is output on the DAC .
 * Return the current value on success negative on failures.
 */
int
hal_dac_get_current(struct hal_dac *pdac);

/*
 * Returns the number of bit of resolution in this DAC.
 * For example if the system has an 8-bit DAC reporting
 * values from 0= to 255 (2^8-1), this function would return
 * the value 8. returns negative or zero on error */
int
hal_dac_get_bits(struct hal_dac *pdac);

/*
 * Returns the positive reference voltage for a maximum DAC reading.
 * This API assumes the negative reference voltage is zero volt.
 * Returns negative or zero on error.
 */
int
hal_dac_get_ref_mv(struct hal_dac *pdac);

/* turns the DAC off.  Re-enable with hal_dac_write */
int
hal_dac_disable(struct hal_dac *pdac);


/* Converts a value in millivolts to a DAC value for this DAC */
int
hal_dac_to_val(struct hal_dac *pdac, int mvolts);


#ifdef __cplusplus
}
#endif

#endif /* H_HAL_DAC_ */
