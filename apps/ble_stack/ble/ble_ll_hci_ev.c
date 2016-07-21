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
#include "nimble/ble.h"
#include "nimble/hci_common.h"
#include "controller/ble_ll.h"
#include "controller/ble_ll_hci.h"
#include "controller/ble_ll_ctrl.h"
#include "ble_ll_conn_priv.h"

#if (BLETEST_CONCURRENT_CONN_TEST == 1)
extern void bletest_ltk_req_reply(uint16_t handle);
#endif

/**
 * Send a data length change event for a connection to the host.
 *
 * @param connsm Pointer to connection state machine
 */
void
ble_ll_hci_ev_datalen_chg(struct ble_ll_conn_sm *connsm)
{
    uint8_t *evbuf;

    if (ble_ll_hci_is_le_event_enabled(BLE_HCI_LE_SUBEV_DATA_LEN_CHG)) {
        evbuf = os_memblock_get(&g_hci_cmd_pool);
        if (evbuf) {
            evbuf[0] = BLE_HCI_EVCODE_LE_META;
            evbuf[1] = BLE_HCI_LE_DATA_LEN_CHG_LEN;
            evbuf[2] = BLE_HCI_LE_SUBEV_DATA_LEN_CHG;
            htole16(evbuf + 3, connsm->conn_handle);
            htole16(evbuf + 5, connsm->eff_max_tx_octets);
            htole16(evbuf + 7, connsm->eff_max_tx_time);
            htole16(evbuf + 9, connsm->eff_max_rx_octets);
            htole16(evbuf + 11, connsm->eff_max_rx_time);
            ble_ll_hci_event_send(evbuf);
        }
    }
}

/**
 * Send a connection parameter request event for a connection to the host.
 *
 * @param connsm Pointer to connection state machine
 */
void
ble_ll_hci_ev_rem_conn_parm_req(struct ble_ll_conn_sm *connsm,
                                struct ble_ll_conn_params *cp)
{
    uint8_t *evbuf;

    if (ble_ll_hci_is_le_event_enabled(BLE_HCI_LE_SUBEV_REM_CONN_PARM_REQ)) {
        evbuf = os_memblock_get(&g_hci_cmd_pool);
        if (evbuf) {
            evbuf[0] = BLE_HCI_EVCODE_LE_META;
            evbuf[1] = BLE_HCI_LE_REM_CONN_PARM_REQ_LEN;
            evbuf[2] = BLE_HCI_LE_SUBEV_REM_CONN_PARM_REQ;
            htole16(evbuf + 3, connsm->conn_handle);
            htole16(evbuf + 5, cp->interval_min);
            htole16(evbuf + 7, cp->interval_max);
            htole16(evbuf + 9, cp->latency);
            htole16(evbuf + 11, cp->timeout);
            ble_ll_hci_event_send(evbuf);
        }
    }
}

/**
 * Send a connection update event.
 *
 * @param connsm Pointer to connection state machine
 * @param status The error code.
 */
void
ble_ll_hci_ev_conn_update(struct ble_ll_conn_sm *connsm, uint8_t status)
{
    uint8_t *evbuf;

    if (ble_ll_hci_is_le_event_enabled(BLE_HCI_LE_SUBEV_CONN_UPD_COMPLETE)) {
        evbuf = os_memblock_get(&g_hci_cmd_pool);
        if (evbuf) {
            evbuf[0] = BLE_HCI_EVCODE_LE_META;
            evbuf[1] = BLE_HCI_LE_CONN_UPD_LEN;
            evbuf[2] = BLE_HCI_LE_SUBEV_CONN_UPD_COMPLETE;
            evbuf[3] = status;
            htole16(evbuf + 4, connsm->conn_handle);
            htole16(evbuf + 6, connsm->conn_itvl);
            htole16(evbuf + 8, connsm->slave_latency);
            htole16(evbuf + 10, connsm->supervision_tmo);
            ble_ll_hci_event_send(evbuf);
        }
    }
}

#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
void
ble_ll_hci_ev_encrypt_chg(struct ble_ll_conn_sm *connsm, uint8_t status)
{
    uint8_t evcode;
    uint8_t *evbuf;
    uint8_t evlen;

    if (CONN_F_ENC_CHANGE_SENT(connsm) == 0) {
        evcode = BLE_HCI_EVCODE_ENCRYPT_CHG;
        evlen = BLE_HCI_EVENT_ENCRYPT_CHG_LEN;
    } else {
        evcode = BLE_HCI_EVCODE_ENC_KEY_REFRESH;
        evlen = BLE_HCI_EVENT_ENC_KEY_REFRESH_LEN;
    }

    if (ble_ll_hci_is_event_enabled(evcode)) {
        evbuf = os_memblock_get(&g_hci_cmd_pool);
        if (evbuf) {
            evbuf[0] = evcode;
            evbuf[1] = evlen;
            evbuf[2] = status;
            htole16(evbuf + 3, connsm->conn_handle);
            if (evcode == BLE_HCI_EVCODE_ENCRYPT_CHG) {
                if (status == BLE_ERR_SUCCESS) {
                    evbuf[5] = 0x01;
                } else {
                    evbuf[5] = 0;
                }
            }
            ble_ll_hci_event_send(evbuf);
        }
    }
    CONN_F_ENC_CHANGE_SENT(connsm) = 1;
}

/**
 * Send a long term key request event for a connection to the host.
 *
 * @param connsm Pointer to connection state machine
 */
int
ble_ll_hci_ev_ltk_req(struct ble_ll_conn_sm *connsm)
{
    int rc;
    uint8_t *evbuf;

    if (ble_ll_hci_is_le_event_enabled(BLE_HCI_LE_SUBEV_LT_KEY_REQ)) {
        evbuf = os_memblock_get(&g_hci_cmd_pool);
        if (evbuf) {
            evbuf[0] = BLE_HCI_EVCODE_LE_META;
            evbuf[1] = BLE_HCI_LE_LT_KEY_REQ_LEN;
            evbuf[2] = BLE_HCI_LE_SUBEV_LT_KEY_REQ;
            htole16(evbuf + 3, connsm->conn_handle);
            htole64(evbuf + 5, connsm->enc_data.host_rand_num);
            htole16(evbuf + 13, connsm->enc_data.enc_div);
            ble_ll_hci_event_send(evbuf);
        }
        rc = 0;
    } else {
        rc = -1;
    }

#if (BLETEST_CONCURRENT_CONN_TEST == 1)
    if (rc == 0) {
        bletest_ltk_req_reply(connsm->conn_handle);
    }
#endif
    return rc;
}
#endif

void
ble_ll_hci_ev_rd_rem_used_feat(struct ble_ll_conn_sm *connsm, uint8_t status)
{
    uint8_t *evbuf;

    if (ble_ll_hci_is_le_event_enabled(BLE_HCI_LE_SUBEV_RD_REM_USED_FEAT)) {
        evbuf = os_memblock_get(&g_hci_cmd_pool);
        if (evbuf) {
            evbuf[0] = BLE_HCI_EVCODE_LE_META;
            evbuf[1] = BLE_HCI_LE_RD_REM_USED_FEAT_LEN;
            evbuf[2] = BLE_HCI_LE_SUBEV_RD_REM_USED_FEAT;
            evbuf[3] = status;
            htole16(evbuf + 4, connsm->conn_handle);
            memset(evbuf + 6, 0, BLE_HCI_RD_LOC_SUPP_FEAT_RSPLEN);
            evbuf[6] = connsm->common_features;
            ble_ll_hci_event_send(evbuf);
        }
    }
}

void
ble_ll_hci_ev_rd_rem_ver(struct ble_ll_conn_sm *connsm, uint8_t status)
{
    uint8_t *evbuf;

    if (ble_ll_hci_is_event_enabled(BLE_HCI_EVCODE_RD_REM_VER_INFO_CMP)) {
        evbuf = os_memblock_get(&g_hci_cmd_pool);
        if (evbuf) {
            evbuf[0] = BLE_HCI_EVCODE_RD_REM_VER_INFO_CMP;
            evbuf[1] = BLE_HCI_EVENT_RD_RM_VER_LEN;
            evbuf[2] = status;
            htole16(evbuf + 3, connsm->conn_handle);
            evbuf[5] = connsm->vers_nr;
            htole16(evbuf + 6, connsm->comp_id);
            htole16(evbuf + 8, connsm->sub_vers_nr);
            ble_ll_hci_event_send(evbuf);
        }
    }
}
