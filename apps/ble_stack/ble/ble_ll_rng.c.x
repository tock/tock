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

#include <stdint.h>
#include <assert.h>
#include <string.h>
#include "os/os.h"
#include "nimble/ble.h"
#include "controller/ble_hw.h"
#include "controller/ble_ll.h"

/* This is a simple circular buffer for holding N samples of random data */
struct ble_ll_rnum_data
{
    uint8_t rnd_in;
    uint8_t rnd_out;
    uint8_t rnd_size;
    uint8_t _pad;
};

#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
int
ble_ll_rng_init(void)
{
    return 0;
}

/* Get 'len' bytes of random data */
int
ble_ll_rand_data_get(uint8_t *buf, uint8_t len)
{
    os_sr_t sr;

    while (len != 0) {
        OS_ENTER_CRITICAL(sr);

        OS_EXIT_CRITICAL(sr);
        --len;
    }

    return 0;
}
#endif

