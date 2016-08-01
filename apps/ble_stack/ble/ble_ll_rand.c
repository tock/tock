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
#include "nimble/nimble_opt.h"
#include "controller/ble_hw.h"
#include "controller/ble_ll.h"

/* This is a simple circular buffer for holding N samples of random data */
struct ble_ll_rnum_data
{
    uint8_t *rnd_in;
    uint8_t *rnd_out;
    uint8_t rnd_size;
};

struct ble_ll_rnum_data g_ble_ll_rnum_data;
uint8_t g_ble_ll_rnum_buf[NIMBLE_OPT_LL_RNG_BUFSIZE];

#define IS_RNUM_BUF_END(x)  (x == &g_ble_ll_rnum_buf[NIMBLE_OPT_LL_RNG_BUFSIZE])

void
ble_ll_rand_sample(uint8_t rnum)
{
    os_sr_t sr;

    OS_ENTER_CRITICAL(sr);
    if (g_ble_ll_rnum_data.rnd_size < NIMBLE_OPT_LL_RNG_BUFSIZE) {
        ++g_ble_ll_rnum_data.rnd_size;
        g_ble_ll_rnum_data.rnd_in[0] = rnum;
        if (IS_RNUM_BUF_END(g_ble_ll_rnum_data.rnd_in)) {
            g_ble_ll_rnum_data.rnd_in = g_ble_ll_rnum_buf;
        } else {
            ++g_ble_ll_rnum_data.rnd_in;
        }
    } else {
        /* Stop generating random numbers as we are full */
        ble_hw_rng_stop();
    }
    OS_EXIT_CRITICAL(sr);
}

/* Get 'len' bytes of random data */
int
ble_ll_rand_data_get(uint8_t *buf, uint8_t len)
{
    uint8_t rnums;
    os_sr_t sr;

    while (len != 0) {
        OS_ENTER_CRITICAL(sr);
        rnums = g_ble_ll_rnum_data.rnd_size;
        if (rnums > len) {
            rnums = len;
        }
        len -= rnums;
        g_ble_ll_rnum_data.rnd_size -= rnums;
        while (rnums) {
            buf[0] = g_ble_ll_rnum_data.rnd_out[0];
            if (IS_RNUM_BUF_END(g_ble_ll_rnum_data.rnd_out)) {
                g_ble_ll_rnum_data.rnd_out = g_ble_ll_rnum_buf;
            } else {
                ++g_ble_ll_rnum_data.rnd_out;
            }
            ++buf;
            --rnums;
        }
        OS_EXIT_CRITICAL(sr);

        /* Make sure rng is started! */
        ble_hw_rng_start();

        /* Wait till bytes are in buffer. */
        if (len) {
            while ((g_ble_ll_rnum_data.rnd_size < len) &&
                   (g_ble_ll_rnum_data.rnd_size < NIMBLE_OPT_LL_RNG_BUFSIZE)) {
                /* Spin here */
            }
        }
    }

    return BLE_ERR_SUCCESS;
}

/**
 * Start the generation of random numbers
 *
 * @return int
 */
int
ble_ll_rand_start(void)
{
    /* Start the generation of numbers if we are not full */
    if (g_ble_ll_rnum_data.rnd_size < NIMBLE_OPT_LL_RNG_BUFSIZE) {
        ble_hw_rng_start();
    }
    return 0;
}

/**
 * Initialize LL random number generation. Should be called only once on
 * initialization.
 *
 * @return int
 */
int
ble_ll_rand_init(void)
{
    g_ble_ll_rnum_data.rnd_in = g_ble_ll_rnum_buf;
    g_ble_ll_rnum_data.rnd_out = g_ble_ll_rnum_buf;
    ble_hw_rng_init(ble_ll_rand_sample, 1);
    return 0;
}
