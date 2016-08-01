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

#ifndef H_HAL_I2C_
#define H_HAL_I2C_

#include <inttypes.h>
#include <bsp/bsp_sysid.h>

#ifdef __cplusplus
extern "C" {
#endif

/* This is the API for an i2c bus.  Currently, this is a master API
 * allowing the mynewt device to function as an I2C master.
 *
 * A slave API is pending for future release
 *
 * Typical usage of this API is as follows:
 *
 * Initialize an i2c device with
 *      hal_i2c_init()
 *
 * When you with to perform an i2c transaction, issue
 *      hal_i2c_master_begin()l
 * followed by the transaction.  For example, in an I2C memory access access
 * you might write and address and then read back data
 *      hal_i2c_write(); -- write amemory ddress to device
 *      hal_i2c_read(); --- read back data
 * then end the transaction
 *      hal_i2c_end();
 */

struct hal_i2c;

/* when sending a packet, use this structure to pass the arguments */
struct hal_i2c_master_data {
    uint8_t  address;   /* destination address */
            /* a I2C address has 7 bits. In the protocol these
             * 7 bits are combined with a 1 bit R/W bit to specify read
             * or write operation in an 8-bit address field sent to
             * the remote device .  This API accepts the 7-bit
             * address as its argument in the 7 LSBs of the
             * address field above.  For example if I2C was
             * writing a 0x81 in its protocol, you would pass
             * only the top 7-bits to this function as 0x40 */
    uint16_t len;       /* number of buffer bytes to transmit or receive */
    uint8_t *buffer;    /* buffer space to hold the transmit or receive */
};

/* Initialize a new i2c device with the given system id.
 * Returns a pointer to the i2c device or NULL on error
 */
struct hal_i2c*
hal_i2c_init(enum system_device_id sysid);

/* Sends a start condition and writes <len> bytes of data on the i2c.
 * This API assumes that you have already called hal_i2c_master_begin
 *  It will fail if you have not. This API does NOT issue a stop condition.
 * You must stop the bus after successful or unsuccessful write attempts.
 * This API is blocking until an error or NaK occurs. Timeout is platform
 * dependent
 * Returns 0 on success, negative on failure
 */
int
hal_i2c_master_write(struct hal_i2c*, struct hal_i2c_master_data *pdata);

/* Sends a start condition and reads <len> bytes of data on the i2c.
 * This API assumes that you have already called hal_i2c_master_begin
 *  It will fail if you have not. This API does NOT issue a stop condition.
 * You must stop the bus after successful or unsuccessful write attempts.
 * This API is blocking until an error or NaK occurs. Timeout is platform
 * dependent
 * Returns 0 on success, negative on failure
 */
int
hal_i2c_master_read(struct hal_i2c*, struct hal_i2c_master_data *pdata);

/*
 * Starts an I2C transaction with the driver. This API does not send
 * anything over the bus itself
 */
int
hal_i2c_master_begin(struct hal_i2c*);

/* issues a stop condition on the bus and ends the I2C transaction.
 * You must call i2c_master_end for every hal_i2c_master_begin
 * API call that succeeds  */
int
hal_i2c_master_end(struct hal_i2c*);

/* Probes the i2c bus for a device with this address.  THIS API
 * issues a start condition, probes the address using a read
 * command and issues a stop condition.   There is no need to call
 * hal_i2c_master_begin/end with this method
 */
int
hal_i2c_master_probe(struct hal_i2c*, uint8_t address);

#ifdef __cplusplus
}
#endif

#endif /* H_HAL_I2C_ */
