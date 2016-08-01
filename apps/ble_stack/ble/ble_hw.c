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
#include "mcu/nrf51_bitfields.h"
#include "controller/ble_hw.h"
#include "bsp/cmsis_nvic.h"

/* Total number of white list elements supported by nrf52 */
#define BLE_HW_WHITE_LIST_SIZE      (8)

/* We use this to keep track of which entries are set to valid addresses */
static uint8_t g_ble_hw_whitelist_mask;

/* Random number generator isr callback */
ble_rng_isr_cb_t g_ble_rng_isr_cb;

/**
 * Clear the whitelist
 * 
 * @return int 
 */
void
ble_hw_whitelist_clear(void)
{
    NRF_RADIO->DACNF = 0;
    g_ble_hw_whitelist_mask = 0;
}

/**
 * Add a device to the hw whitelist 
 * 
 * @param addr 
 * @param addr_type 
 * 
 * @return int 0: success, BLE error code otherwise
 */
int
ble_hw_whitelist_add(uint8_t *addr, uint8_t addr_type)
{
    int i;
    uint32_t mask;

    /* Find first ununsed device address match element */
    mask = 0x01;
    for (i = 0; i < BLE_HW_WHITE_LIST_SIZE; ++i) {
        if ((mask & g_ble_hw_whitelist_mask) == 0) {
            NRF_RADIO->DAB[i] = le32toh(addr);
            NRF_RADIO->DAP[i] = le16toh(addr + 4);
            if (addr_type == BLE_ADDR_TYPE_RANDOM) {
                NRF_RADIO->DACNF |= (mask << 8);
            }
            g_ble_hw_whitelist_mask |= mask;
            return BLE_ERR_SUCCESS;
        }
        mask <<= 1;
    }

    return BLE_ERR_MEM_CAPACITY;
}

/**
 * Remove a device from the hw whitelist 
 * 
 * @param addr 
 * @param addr_type 
 * 
 */
void
ble_hw_whitelist_rmv(uint8_t *addr, uint8_t addr_type)
{
    int i;
    uint8_t cfg_addr;
    uint16_t dap;
    uint16_t txadd;
    uint32_t dab;
    uint32_t mask;

    /* Find first ununsed device address match element */
    dab = le32toh(addr);
    dap = le16toh(addr + 4);
    txadd = NRF_RADIO->DACNF >> 8;
    mask = 0x01;
    for (i = 0; i < BLE_HW_WHITE_LIST_SIZE; ++i) {
        if (mask & g_ble_hw_whitelist_mask) {
            if ((dab == NRF_RADIO->DAB[i]) && (dap == NRF_RADIO->DAP[i])) {
                cfg_addr = txadd & mask;
                if (addr_type == BLE_ADDR_TYPE_RANDOM) {
                    if (cfg_addr != 0) {
                        break;
                    }
                } else {
                    if (cfg_addr == 0) {
                        break;
                    }
                }
            }
        }
        mask <<= 1;
    }

    if (i < BLE_HW_WHITE_LIST_SIZE) {
        g_ble_hw_whitelist_mask &= ~mask;
        NRF_RADIO->DACNF &= ~mask;
    }
}

/**
 * Returns the size of the whitelist in HW 
 * 
 * @return int Number of devices allowed in whitelist
 */
uint8_t
ble_hw_whitelist_size(void)
{
    return BLE_HW_WHITE_LIST_SIZE;
}

/**
 * Enable the whitelisted devices 
 */
void
ble_hw_whitelist_enable(void)
{
    /* Enable the configured device addresses */
    NRF_RADIO->DACNF |= g_ble_hw_whitelist_mask;
}

/**
 * Disables the whitelisted devices 
 */
void
ble_hw_whitelist_disable(void)
{
    /* Disable all whitelist devices */
    NRF_RADIO->DACNF &= 0x0000ff00;
}

/**
 * Boolean function which returns true ('1') if there is a match on the 
 * whitelist. 
 * 
 * @return int 
 */
int
ble_hw_whitelist_match(void)
{
    return (int)NRF_RADIO->EVENTS_DEVMATCH;
}

/* Encrypt data */
int
ble_hw_encrypt_block(struct ble_encryption_block *ecb)
{
    int rc;

    /* Stop ECB */
    NRF_ECB->TASKS_STOPECB = 1;
    NRF_ECB->EVENTS_ENDECB = 0;
    NRF_ECB->EVENTS_ERRORECB = 0;
    NRF_ECB->ECBDATAPTR = (uint32_t)ecb;

    /* Start ECB */
    NRF_ECB->TASKS_STARTECB = 1;

    /* Wait till error or done */
    while (1) {
        if (NRF_ECB->EVENTS_ENDECB != 0) {
            rc = 0;
            break;
        }

        if (NRF_ECB->EVENTS_ERRORECB != 0) {
            rc = -1;
            break;
        }
    }

    return rc;
}

/**
 * Random number generator ISR.
 */
static void
ble_rng_isr(void)
{
    uint8_t rnum;

    /* No callback? Clear and disable interrupts */
    if (g_ble_rng_isr_cb == NULL) {
        NRF_RNG->INTENCLR = 1;
        NRF_RNG->EVENTS_VALRDY = 0;
        (void)NRF_RNG->SHORTS;
        return;
    }

    /* If there is a value ready grab it */
    if (NRF_RNG->EVENTS_VALRDY) {
        NRF_RNG->EVENTS_VALRDY = 0;
        rnum = (uint8_t)NRF_RNG->VALUE;
        (*g_ble_rng_isr_cb)(rnum);
    }
}

/**
 * Initialize the random number generator
 * 
 * @param cb 
 * @param bias 
 * 
 * @return int 
 */
int
ble_hw_rng_init(ble_rng_isr_cb_t cb, int bias)
{
    /* Set bias */
    if (bias) {
        NRF_RNG->CONFIG = 1;
    } else {
        NRF_RNG->CONFIG = 0;
    }

    /* If we were passed a function pointer we need to enable the interrupt */
    if (cb != NULL) {
        NVIC_SetPriority(RNG_IRQn, (1 << __NVIC_PRIO_BITS) - 1);
        NVIC_SetVector(RNG_IRQn, (uint32_t)ble_rng_isr);
        NVIC_EnableIRQ(RNG_IRQn);
        g_ble_rng_isr_cb = cb;
    }

    return 0;
}

/**
 * Start the random number generator
 * 
 * @return int 
 */
int
ble_hw_rng_start(void)
{
    os_sr_t sr;

    /* No need for interrupt if there is no callback */
    OS_ENTER_CRITICAL(sr);
    if (NRF_RNG->TASKS_START == 0) {
        NRF_RNG->EVENTS_VALRDY = 0;
        if (g_ble_rng_isr_cb) {
            NRF_RNG->INTENSET = 1;
        }
        NRF_RNG->TASKS_START = 1;
    }
    OS_EXIT_CRITICAL(sr);

    return 0;
}

/**
 * Stop the random generator
 * 
 * @return int 
 */
int
ble_hw_rng_stop(void)
{
    os_sr_t sr;

    /* No need for interrupt if there is no callback */
    OS_ENTER_CRITICAL(sr);
    NRF_RNG->INTENCLR = 1;
    NRF_RNG->TASKS_STOP = 1;
    NRF_RNG->EVENTS_VALRDY = 0;
    OS_EXIT_CRITICAL(sr);

    return 0;
}

/**
 * Read the random number generator.
 * 
 * @return uint8_t 
 */
uint8_t
ble_hw_rng_read(void)
{
    uint8_t rnum;

    /* Wait for a sample */
    while (NRF_RNG->EVENTS_VALRDY == 0) {
    }

    NRF_RNG->EVENTS_VALRDY = 0;
    rnum = (uint8_t)NRF_RNG->VALUE;

    return rnum;
}
