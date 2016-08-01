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

#ifndef H_HAL_ADC_
#define H_HAL_ADC_

#ifdef __cplusplus
extern "C" {
#endif

/* for the pin descriptor enum */
#include <bsp/bsp_sysid.h>

/* This is the device for an ADC. The application using the ADC device
 * does not need to know the definition of this device and can operate
 * with a pointer to this device.  you can get/build device pointers in the
 * BSP
 */
struct hal_adc;

/* initialize the ADC on the corresponding BSP Pin. Returns a pointer
 * to the adc object to use for the methods below. Returns NULL on
 * error
 */
 struct hal_adc *hal_adc_init(enum system_device_id sysid);

/* read the ADC corresponding to sysid in your system.  Returns
 * the  adc value read or negative on error, See
 * hal_adc_get_resolution to check the range of the return value
 */
int hal_adc_read(struct hal_adc *padc);

/* returns the number of bit of resolution in this ADC.
 * For example if the system has an 8-bit ADC reporting
 * values from 0= to 255 (2^8-1), this function would return
 * the value 8. returns negative or zero on error
 */
int hal_adc_get_bits(struct hal_adc *padc);

/* Returns the positive reference voltage for a maximum ADC reading.
 * This API assumes the negative reference voltage is zero volt.
 * Returns negative or zero on error.
 */
int hal_adc_get_ref_mv(struct hal_adc *padc);

/* Converts and ADC value to millivolts. This is a helper function
 * but does call the ADC to get the reference voltage and
 * resolution
 */
int hal_adc_to_mv(struct hal_adc *padc, int val);


#ifdef __cplusplus
}
#endif

#endif /* H_HAL_ADC_ */
