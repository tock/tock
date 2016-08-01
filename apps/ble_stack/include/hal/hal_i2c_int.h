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


#ifndef H_HAL_I2C_INT_
#define H_HAL_I2C_INT_

#ifdef __cplusplus
extern "C" {
#endif

#include <hal/hal_i2c.h>
#include <inttypes.h>

struct hal_i2c;

struct hal_i2c_funcs {
    int (*hi2cm_write_data) (struct hal_i2c *pi2c, struct hal_i2c_master_data *ppkt);
    int (*hi2cm_read_data)  (struct hal_i2c *pi2c, struct hal_i2c_master_data *ppkt);
    int (*hi2cm_probe)      (struct hal_i2c *pi2c, uint8_t address);
    int (*hi2cm_start)      (struct hal_i2c *pi2c);
    int (*hi2cm_stop)       (struct hal_i2c *pi2c);
};

struct hal_i2c {
    const struct hal_i2c_funcs *driver_api;
};

struct hal_i2c *
bsp_get_hal_i2c_driver(enum system_device_id sysid);

#ifdef __cplusplus
}
#endif

#endif /* H_HAL_I2C_INT_ */

