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

#ifndef HAL_ADC_INT_H
#define HAL_ADC_INT_H

#ifdef __cplusplus
extern "C" {
#endif

#include <inttypes.h>
#include <bsp/bsp_sysid.h>


struct hal_adc;

/* These functions make up the driver API for ADC devices.  All
 * ADC devices with Mynewt support implement this interface
 */
struct hal_adc_funcs {
    int (*hadc_read)            (struct hal_adc *padc);
    int (*hadc_get_bits)         (struct hal_adc *padc);
    int (*hadc_get_ref_mv)       (struct hal_adc *padc);
};

/* This is the internal device representation for a hal_adc device.
 *
 * Its main goal is to wrap the const drivers in a non-const structure.
 * Thus these can be made on the stack and wrapped with other non-const
 * structures.
 *
 * For example, if you are creating a adc driver you can use
 *
 * struct my_adc_driver {
 *     struct hal_adc_s parent;
 *     int              my_stuff 1;
 *     char            *mybuff;
 * };
 *
 * See the native MCU and BSP for examples
 */
struct hal_adc {
    const struct hal_adc_funcs  *driver_api;
};

/* The  BSP must implement this factory to get devices for the
 * application.  */
extern struct hal_adc *
bsp_get_hal_adc(enum system_device_id sysid);

#ifdef __cplusplus
}
#endif

#endif /* HAL_ADC_INT_H */
