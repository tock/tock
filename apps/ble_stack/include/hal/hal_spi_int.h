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

#ifndef H_HAL_SPI_INT_
#define H_HAL_SPI_INT_

#ifdef __cplusplus
extern "C" {
#endif

#include <bsp/bsp_sysid.h>

struct hal_spi;

/* configure the spi */
int
hal_spi_config(struct hal_spi *pspi, struct hal_spi_settings *psettings);

/* do a blocking master spi transfer */
int
hal_spi_master_transfer(struct hal_spi *psdi, uint16_t tx);

/* These functions make up the driver API for DAC devices.  All
 * DAC devices with Mynewt support implement this interface
 */
struct hal_spi_funcs {
    int (*hspi_config)           (struct hal_spi *pspi, struct hal_spi_settings *psettings);
    int (*hspi_master_transfer)  (struct hal_spi *psdi, uint16_t tx);
};

/* This is the internal device representation for a hal_spi device.
 *
 * Its main goal is to wrap the const drivers in a non-const structure.
 * Thus these can be made on the stack and wrapped with other non-const
 * structures.
 *
 * For example, if you are creating a spi driver you can use
 *
 * struct my_spi_driver {
 *     struct hal_spi   parent;
 *     int              my_stuff 1;
 *     char            *mybuff;
 * };
 *
 * See the native MCU and BSP for examples
 */
struct hal_spi {
    const struct hal_spi_funcs  *driver_api;
};

/* The  BSP must implement this factory to get devices for the
 * application.
 */
extern struct hal_spi *
bsp_get_hal_spi(enum system_device_id sysid);


#ifdef __cplusplus
}
#endif

#endif /* H_HAL_SPI_INT_ */
