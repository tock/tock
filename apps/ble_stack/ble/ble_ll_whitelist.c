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
#include "controller/ble_ll_whitelist.h"
#include "controller/ble_ll_hci.h"
#include "controller/ble_ll_adv.h"
#include "controller/ble_ll_scan.h"
#include "controller/ble_hw.h"

#ifndef BLE_USES_HW_WHITELIST
struct ble_ll_whitelist_entry
{
    uint8_t wl_valid;
    uint8_t wl_addr_type;
    uint8_t wl_dev_addr[BLE_DEV_ADDR_LEN];
};

struct ble_ll_whitelist_entry g_ble_ll_whitelist[NIMBLE_OPT_LL_WHITELIST_SIZE];
#endif

static int
ble_ll_whitelist_chg_allowed(void)
{
    int rc;

    /*
     * This command is not allowed if:
     *  -> advertising uses the whitelist and we are currently advertising.
     *  -> scanning uses the whitelist and is enabled.
     *  -> initiating uses whitelist and a LE create connection command is in
     *     progress
     */
    rc = 1;
    if (!ble_ll_adv_can_chg_whitelist() || !ble_ll_scan_can_chg_whitelist()) {
        rc = 0;
    }
    return rc;
}

/**
 * Clear the whitelist.
 *
 * @return int 0: success, BLE error code otherwise
 */
int
ble_ll_whitelist_clear(void)
{

    /* Check proper state */
    if (!ble_ll_whitelist_chg_allowed()) {
        return BLE_ERR_CMD_DISALLOWED;
    }

#ifdef BLE_USES_HW_WHITELIST
    ble_hw_whitelist_clear();
#else
    int i;
    struct ble_ll_whitelist_entry *wl;

    /* Set the number of entries to 0 */
    wl = &g_ble_ll_whitelist[0];
    for (i = 0; i < NIMBLE_OPT_LL_WHITELIST_SIZE; ++i) {
        wl->wl_valid = 0;
        ++wl;
    }
#endif

    return BLE_ERR_SUCCESS;
}

/**
 * Read the size of the whitelist. This is the total number of whitelist
 * entries allowed by the controller.
 *
 * @param rspbuf Pointer to response buffer
 *
 * @return int 0: success.
 */
int
ble_ll_whitelist_read_size(uint8_t *rspbuf, uint8_t *rsplen)
{
#ifdef BLE_USES_HW_WHITELIST
    rspbuf[0] = ble_hw_whitelist_size();
#else
    rspbuf[0] = NIMBLE_OPT_LL_WHITELIST_SIZE;
#endif
    *rsplen = 1;
    return BLE_ERR_SUCCESS;
}

#ifndef BLE_USES_HW_WHITELIST
/**
 * Used to determine if the device is on the whitelist.
 *
 * @param addr
 * @param addr_type Public address (0) or random address (1)
 *
 * @return int 0: device is not on whitelist; otherwise the return value
 * is the 'position' of the device in the whitelist (the index of the element
 * plus 1).
 */
static int
ble_ll_is_on_whitelist(uint8_t *addr, uint8_t addr_type)
{
    int i;
    struct ble_ll_whitelist_entry *wl;

    wl = &g_ble_ll_whitelist[0];
    for (i = 0; i < NIMBLE_OPT_LL_WHITELIST_SIZE; ++i) {
        if ((wl->wl_valid) && (wl->wl_addr_type == addr_type) &&
            (!memcmp(&wl->wl_dev_addr[0], addr, BLE_DEV_ADDR_LEN))) {
            return i + 1;
        }
        ++wl;
    }

    return 0;
}
#endif

/**
 * Is there a match between the device and a device on the whitelist
 *
 * @param addr
 * @param addr_type Public address (0) or random address (1)
 *
 * @return int
 */

#pragma GCC diagnostic ignored "-Wunused-parameter"
int
ble_ll_whitelist_match(uint8_t *addr, uint8_t addr_type)
{
    int rc;
#ifdef BLE_USES_HW_WHITELIST
    rc = ble_hw_whitelist_match();
#else
    rc = ble_ll_is_on_whitelist(addr, addr_type);
#endif
    return rc;
}

/**
 * Add a device to the whitelist
 *
 * @return int
 */
int
ble_ll_whitelist_add(uint8_t *addr, uint8_t addr_type)
{
    int rc;

    /* Must be in proper state */
    if (!ble_ll_whitelist_chg_allowed()) {
        return BLE_ERR_CMD_DISALLOWED;
    }

    /* Check if we have any open entries */
#ifdef BLE_USES_HW_WHITELIST
    rc = ble_hw_whitelist_add(addr, addr_type);
#else
    int i;
    struct ble_ll_whitelist_entry *wl;

    rc = BLE_ERR_SUCCESS;
    if (!ble_ll_is_on_whitelist(addr, addr_type)) {
        wl = &g_ble_ll_whitelist[0];
        for (i = 0; i < NIMBLE_OPT_LL_WHITELIST_SIZE; ++i) {
            if (wl->wl_valid == 0) {
                memcpy(&wl->wl_dev_addr[0], addr, BLE_DEV_ADDR_LEN);
                wl->wl_addr_type = addr_type;
                wl->wl_valid = 1;
                break;
            }
            ++wl;
        }

        if (i == NIMBLE_OPT_LL_WHITELIST_SIZE) {
            rc = BLE_ERR_MEM_CAPACITY;
        }
    }
#endif

    return rc;
}

/**
 * Remove a device from the whitelist
 *
 * @param cmdbuf
 *
 * @return int 0: success, BLE error code otherwise
 */
int
ble_ll_whitelist_rmv(uint8_t *addr, uint8_t addr_type)
{
    /* Must be in proper state */
    if (!ble_ll_whitelist_chg_allowed()) {
        return BLE_ERR_CMD_DISALLOWED;
    }

#ifdef BLE_USES_HW_WHITELIST
    ble_hw_whitelist_rmv(addr, addr_type);
#else
    int position;

    position = ble_ll_is_on_whitelist(addr, addr_type);
    if (position) {
        g_ble_ll_whitelist[position - 1].wl_valid = 0;
    }
#endif

    return BLE_ERR_SUCCESS;
}

/**
 * Enable whitelisting.
 *
 * Note: This function has no effect if we are not using HW whitelisting
 */
void
ble_ll_whitelist_enable(void)
{
#ifdef BLE_USES_HW_WHITELIST
    ble_hw_whitelist_enable();
#endif
}

/**
 * Disable whitelisting.
 *
 * Note: This function has no effect if we are not using HW whitelisting
 */
void
ble_ll_whitelist_disable(void)
{
#ifdef BLE_USES_HW_WHITELIST
    ble_hw_whitelist_disable();
#endif
}


