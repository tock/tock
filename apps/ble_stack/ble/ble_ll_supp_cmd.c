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
#include <string.h>

#include "nimble/ble.h"
#include "nimble/nimble_opt.h"
#include "nimble/hci_common.h"
#include "controller/ble_ll.h"
#include "controller/ble_ll_hci.h"

/* Octet 0 */
#define BLE_SUPP_CMD_DISCONNECT             (1 << 5)
#define BLE_LL_SUPP_CMD_OCTET_0             (BLE_SUPP_CMD_DISCONNECT)

/* Octet 5 */
#define BLE_SUPP_CMD_SET_EVENT_MASK         (1 << 6)
#define BLE_LL_SUPP_CMD_OCTET_5             (BLE_SUPP_CMD_SET_EVENT_MASK)

/* Octet 10 */
#define BLE_SUPP_CMD_RD_TX_PWR              (0 << 2)
#define BLE_LL_SUPP_CMD_OCTET_10            (BLE_SUPP_CMD_RD_TX_PWR)

/* Octet 14 */
#define BLE_SUPP_CMD_RD_LOC_VER             (1 << 3)
#define BLE_SUPP_CMD_RD_LOC_SUPP_FEAT       (1 << 5)
#define BLE_LL_SUPP_CMD_OCTET_14            \
(                                           \
    BLE_SUPP_CMD_RD_LOC_VER         |       \
    BLE_SUPP_CMD_RD_LOC_SUPP_FEAT           \
)

/* Octet 15 */
#define BLE_SUPP_CMD_RD_BD_ADDR             (1 << 1)
#define BLE_SUPP_CMD_RD_RSSI                (1 << 5)

#define BLE_LL_SUPP_CMD_OCTET_15            \
(                                           \
    BLE_SUPP_CMD_RD_BD_ADDR         |       \
    BLE_SUPP_CMD_RD_RSSI                    \
)

/* Octet 25 */
#define BLE_SUPP_CMD_LE_SET_EV_MASK         (1 << 0)
#define BLE_SUPP_CMD_LE_RD_BUF_SIZE         (1 << 1)
#define BLE_SUPP_CMD_LE_RD_LOC_FEAT         (1 << 2)
#define BLE_SUPP_CMD_LE_SET_RAND_ADDR       (1 << 4)
#define BLE_SUPP_CMD_LE_SET_ADV_PARAMS      (1 << 5)
#define BLE_SUPP_CMD_LE_SET_ADV_TX_PWR      (1 << 6)
#define BLE_SUPP_CMD_LE_SET_ADV_DATA        (1 << 7)

#define BLE_LL_SUPP_CMD_OCTET_25            \
(                                           \
    BLE_SUPP_CMD_LE_SET_EV_MASK     |       \
    BLE_SUPP_CMD_LE_RD_BUF_SIZE     |       \
    BLE_SUPP_CMD_LE_RD_LOC_FEAT     |       \
    BLE_SUPP_CMD_LE_SET_RAND_ADDR   |       \
    BLE_SUPP_CMD_LE_SET_ADV_PARAMS  |       \
    BLE_SUPP_CMD_LE_SET_ADV_TX_PWR  |       \
    BLE_SUPP_CMD_LE_SET_ADV_DATA            \
)

/* Octet 26 */
#define BLE_SUPP_CMD_LE_SET_SCAN_RSP_DATA   (1 << 0)
#define BLE_SUPP_CMD_LE_SET_ADV_ENABLE      (1 << 1)
#define BLE_SUPP_CMD_LE_SET_SCAN_PARAMS     (1 << 2)
#define BLE_SUPP_CMD_LE_SET_SCAN_ENABLE     (1 << 3)
#define BLE_SUPP_CMD_LE_CREATE_CONN         (1 << 4)
#define BLE_SUPP_CMD_LE_CREATE_CONN_CANCEL  (1 << 5)
#define BLE_SUPP_CMD_LE_RD_WHITELIST_SIZE   (1 << 6)
#define BLE_SUPP_CMD_LE_CLR_WHITELIST       (1 << 7)

#define BLE_LL_SUPP_CMD_OCTET_26            \
(                                           \
    BLE_SUPP_CMD_LE_SET_SCAN_RSP_DATA   |   \
    BLE_SUPP_CMD_LE_SET_ADV_ENABLE      |   \
    BLE_SUPP_CMD_LE_SET_SCAN_PARAMS     |   \
    BLE_SUPP_CMD_LE_SET_SCAN_ENABLE     |   \
    BLE_SUPP_CMD_LE_CREATE_CONN         |   \
    BLE_SUPP_CMD_LE_CREATE_CONN_CANCEL  |   \
    BLE_SUPP_CMD_LE_RD_WHITELIST_SIZE   |   \
    BLE_SUPP_CMD_LE_CLR_WHITELIST           \
)

/* Octet 27 */
#define BLE_SUPP_CMD_LE_ADD_DEV_WHITELIST   (1 << 0)
#define BLE_SUPP_CMD_LE_RMV_DEV_WHITELIST   (1 << 1)
#define BLE_SUPP_CMD_LE_CONN_UPDATE         (1 << 2)
#define BLE_SUPP_CMD_LE_SET_HOST_CHAN_CLASS (1 << 3)
#define BLE_SUPP_CMD_LE_RD_CHAN_MAP         (1 << 4)
#define BLE_SUPP_CMD_LE_RD_REM_USED_FEAT    (1 << 5)
#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
#define BLE_SUPP_CMD_LE_ENCRYPT             (1 << 6)
#else
#define BLE_SUPP_CMD_LE_ENCRYPT             (0 << 6)
#endif
#define BLE_SUPP_CMD_LE_RAND                (1 << 7)

#define BLE_LL_SUPP_CMD_OCTET_27            \
(                                           \
    BLE_SUPP_CMD_LE_ENCRYPT             |   \
    BLE_SUPP_CMD_LE_RAND                |   \
    BLE_SUPP_CMD_LE_ADD_DEV_WHITELIST   |   \
    BLE_SUPP_CMD_LE_RMV_DEV_WHITELIST   |   \
    BLE_SUPP_CMD_LE_CONN_UPDATE         |   \
    BLE_SUPP_CMD_LE_SET_HOST_CHAN_CLASS |   \
    BLE_SUPP_CMD_LE_RD_CHAN_MAP         |   \
    BLE_SUPP_CMD_LE_RD_REM_USED_FEAT        \
)

/* Octet 28 */
#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
#define BLE_SUPP_CMD_LE_START_ENCRYPT       (1 << 0)
#else
#define BLE_SUPP_CMD_LE_START_ENCRYPT       (0 << 0)
#endif
#define BLE_SUPP_CMD_LE_LTK_REQ_REPLY       (0 << 1)
#define BLE_SUPP_CMD_LE_LTK_REQ_NEG_REPLY   (0 << 2)
#define BLE_SUPP_CMD_LE_READ_SUPP_STATES    (1 << 3)
#define BLE_SUPP_CMD_LE_RX_TEST             (0 << 4)
#define BLE_SUPP_CMD_LE_TX_TEST             (0 << 5)
#define BLE_SUPP_CMD_LE_TEST_END            (0 << 6)

#define BLE_LL_SUPP_CMD_OCTET_28            \
(                                           \
    BLE_SUPP_CMD_LE_START_ENCRYPT       |   \
    BLE_SUPP_CMD_LE_LTK_REQ_REPLY       |   \
    BLE_SUPP_CMD_LE_LTK_REQ_NEG_REPLY   |   \
    BLE_SUPP_CMD_LE_READ_SUPP_STATES    |   \
    BLE_SUPP_CMD_LE_RX_TEST             |   \
    BLE_SUPP_CMD_LE_TX_TEST             |   \
    BLE_SUPP_CMD_LE_TEST_END                \
)

/* Octet 33 */
#define BLE_SUPP_CMD_LE_REM_CONN_PRR        (1 << 4)
#define BLE_SUPP_CMD_LE_REM_CONN_PRNR       (1 << 5)
#define BLE_SUPP_CMD_LE_SET_DATALEN         (0 << 6)
#define BLE_SUPP_CMD_LE_RD_SUGG_DATALEN     (0 << 7)

#define BLE_LL_SUPP_CMD_OCTET_33            \
(                                           \
    BLE_SUPP_CMD_LE_REM_CONN_PRR        |   \
    BLE_SUPP_CMD_LE_REM_CONN_PRNR       |   \
    BLE_SUPP_CMD_LE_SET_DATALEN         |   \
    BLE_SUPP_CMD_LE_RD_SUGG_DATALEN         \
)

/* Octet 35 */
#define BLE_SUPP_CMD_LE_RD_MAX_DATALEN      (1 << 3)
#define BLE_LL_SUPP_CMD_OCTET_35            (BLE_SUPP_CMD_LE_RD_MAX_DATALEN)

/* Defines the array of supported commands */
const uint8_t g_ble_ll_supp_cmds[BLE_LL_SUPP_CMD_LEN] =
{
    BLE_LL_SUPP_CMD_OCTET_0,            /* Octet 0 */
    0,
    0,
    0,
    0,
    BLE_LL_SUPP_CMD_OCTET_5,
    0,
    0,
    0,                                  /* Octet 8 */
    0,
    BLE_LL_SUPP_CMD_OCTET_10,
    0,
    0,
    0,
    BLE_LL_SUPP_CMD_OCTET_14,
    BLE_LL_SUPP_CMD_OCTET_15,
    0,                                  /* Octet 16 */
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,                                 /* Octet 24 */
    BLE_LL_SUPP_CMD_OCTET_25,
    BLE_LL_SUPP_CMD_OCTET_26,
    BLE_LL_SUPP_CMD_OCTET_27,
    BLE_LL_SUPP_CMD_OCTET_28,
    0,
    0,
    0,
    0,                                  /* Octet 32 */
    BLE_LL_SUPP_CMD_OCTET_33,
    0,
    BLE_LL_SUPP_CMD_OCTET_35
};
