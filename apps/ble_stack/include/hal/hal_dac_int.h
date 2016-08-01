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

#ifndef H_HAL_DAC_INT_
#define H_HAL_DAC_INT_

#ifdef __cplusplus
extern "C" {
#endif

#include <bsp/bsp_sysid.h>


struct hal_dac;

/* These functions make up the driver API for DAC devices.  All
 * DAC devices with Mynewt support implement this interface
 */
struct hal_dac_funcs {
    int (*hdac_write)            (struct hal_dac *pdac, int val);
    int (*hdac_current)          (struct hal_dac *pdac);
    int (*hdac_disable)          (struct hal_dac *pdac);
    int (*hdac_get_bits)         (struct hal_dac *pdac);
    int (*hdac_get_ref_mv)       (struct hal_dac *pdac);
};

/* This is the internal device representation for a hal_dac device.
 *
 * Its main goal is to wrap the const drivers in a non-const structure.
 * Thus these can be made on the stack and wrapped with other non-const
 * structures.
 *
 * For example, if you are creating a dac driver you can use
 *
 * struct my_dac_driver {
 *     struct hal_dac   parent;
 *     int              my_stuff 1;
 *     char            *mybuff;
 * };
 *
 * See the native MCU and BSP for examples
 */
struct hal_dac {
    const struct hal_dac_funcs  *driver_api;
};

/* The  BSP must implement this factory to get devices for the
 * application.
 */
extern struct hal_dac *
bsp_get_hal_dac(enum system_device_id sysid);


#ifdef __cplusplus
}
#endif

#endif /* HAL_DAC_INT_H */
