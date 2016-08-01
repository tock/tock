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
#include <assert.h>
#include "bsp/bsp.h"
#include "os/os.h"
#include "nimble/ble.h"
#include "nimble/nimble_opt.h"
#include "nimble/hci_common.h"
#include "controller/ble_ll.h"
#include "controller/ble_ll_hci.h"
#include "controller/ble_ll_conn.h"
#include "controller/ble_ll_ctrl.h"
#include "controller/ble_ll_scan.h"
#include "ble_ll_conn_priv.h"

/*
 * Used to limit the rate at which we send the number of completed packets
 * event to the host. This is the os time at which we can send an event.
 */
static uint32_t g_ble_ll_next_num_comp_pkt_evt;

/**
 * Called to check that the connection parameters are within range
 *
 * @param itvl_min
 * @param itvl_max
 * @param latency
 * @param spvn_tmo
 *
 * @return int BLE_ERR_INV_HCI_CMD_PARMS if invalid parameters, 0 otherwise
 */
int
ble_ll_conn_hci_chk_conn_params(uint16_t itvl_min, uint16_t itvl_max,
                                uint16_t latency, uint16_t spvn_tmo)
{
    uint32_t spvn_tmo_usecs;
    uint32_t min_spvn_tmo_usecs;

    if ((itvl_min > itvl_max) ||
        (itvl_min < BLE_HCI_CONN_ITVL_MIN) ||
        (itvl_min > BLE_HCI_CONN_ITVL_MAX) ||
        (latency > BLE_HCI_CONN_LATENCY_MAX) ||
        (spvn_tmo < BLE_HCI_CONN_SPVN_TIMEOUT_MIN) ||
        (spvn_tmo > BLE_HCI_CONN_SPVN_TIMEOUT_MAX)) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    /*
    * Supervision timeout (in msecs) must be more than:
    *  (1 + connLatency) * connIntervalMax * 1.25 msecs * 2.
    */
    spvn_tmo_usecs = spvn_tmo;
    spvn_tmo_usecs *= (BLE_HCI_CONN_SPVN_TMO_UNITS * 1000);
    min_spvn_tmo_usecs = (uint32_t)itvl_max * 2 * BLE_LL_CONN_ITVL_USECS;
    min_spvn_tmo_usecs *= (1 + latency);
    if (spvn_tmo_usecs <= min_spvn_tmo_usecs) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    return BLE_ERR_SUCCESS;
}

/**
 * Make a connect request PDU
 *
 * @param connsm
 */
static void
ble_ll_conn_req_pdu_make(struct ble_ll_conn_sm *connsm)
{
    uint8_t pdu_type;
    uint8_t *addr;
    uint8_t *dptr;
    struct os_mbuf *m;

    m = ble_ll_scan_get_pdu();
    assert(m != NULL);

    /* Construct first PDU header byte */
    pdu_type = BLE_ADV_PDU_TYPE_CONNECT_REQ;

    /* Get pointer to our device address */
    if (connsm->own_addr_type == BLE_HCI_ADV_OWN_ADDR_PUBLIC) {
        addr = g_dev_addr;
    } else if (connsm->own_addr_type == BLE_HCI_ADV_OWN_ADDR_RANDOM) {
        pdu_type |= BLE_ADV_PDU_HDR_TXADD_RAND;
        addr = g_random_addr;
    } else {
        /* XXX: unsupported for now  */
        addr = NULL;
        assert(0);
    }

    /* Set BLE transmit header */
    ble_ll_mbuf_init(m, BLE_CONNECT_REQ_LEN, pdu_type);

    /* Construct the connect request */
    dptr = m->om_data;
    memcpy(dptr, addr, BLE_DEV_ADDR_LEN);

    /* Skip the advertiser's address as we dont know that yet */
    dptr += (2 * BLE_DEV_ADDR_LEN);

    /* Access address */
    htole32(dptr, connsm->access_addr);
    dptr[4] = (uint8_t)connsm->crcinit;
    dptr[5] = (uint8_t)(connsm->crcinit >> 8);
    dptr[6] = (uint8_t)(connsm->crcinit >> 16);
    dptr[7] = connsm->tx_win_size;
    htole16(dptr + 8, connsm->tx_win_off);
    htole16(dptr + 10, connsm->conn_itvl);
    htole16(dptr + 12, connsm->slave_latency);
    htole16(dptr + 14, connsm->supervision_tmo);
    memcpy(dptr + 16, &connsm->chanmap, BLE_LL_CONN_CHMAP_LEN);
    dptr[21] = connsm->hop_inc | (connsm->master_sca << 5);
}

/**
 * Send a connection complete event
 *
 * @param status The BLE error code associated with the event
 */
void
ble_ll_conn_comp_event_send(struct ble_ll_conn_sm *connsm, uint8_t status)
{
    uint8_t *evbuf;

    if (ble_ll_hci_is_le_event_enabled(BLE_HCI_LE_SUBEV_CONN_COMPLETE)) {
        evbuf = os_memblock_get(&g_hci_cmd_pool);
        if (evbuf) {
            evbuf[0] = BLE_HCI_EVCODE_LE_META;
            evbuf[1] = BLE_HCI_LE_CONN_COMPLETE_LEN;
            evbuf[2] = BLE_HCI_LE_SUBEV_CONN_COMPLETE;
            evbuf[3] = status;
            htole16(evbuf + 4, connsm->conn_handle);
            evbuf[6] = connsm->conn_role - 1;
            evbuf[7] = connsm->peer_addr_type;
            memcpy(evbuf + 8, connsm->peer_addr, BLE_DEV_ADDR_LEN);
            htole16(evbuf + 14, connsm->conn_itvl);
            htole16(evbuf + 16, connsm->slave_latency);
            htole16(evbuf + 18, connsm->supervision_tmo);
            evbuf[20] = connsm->master_sca;
            ble_ll_hci_event_send(evbuf);
        }
    }
}

/**
 * Called to create and send the number of completed packets event to the
 * host.
 *
 * Because of the ridiculous spec, all the connection handles are contiguous and
 * then all the completed packets are contiguous. In order to avoid multiple
 * passes through the connection list or allocating a large stack variable or
 * malloc, I just use the event buffer and place the completed packets after
 * the last possible handle. I then copy the completed packets to make it
 * contiguous with the handles.
 *
 * @param connsm
 */
void
ble_ll_conn_num_comp_pkts_event_send(void)
{
    int event_sent;
    uint8_t *evbuf;
    uint8_t *handle_ptr;
    uint8_t *comp_pkt_ptr;
    uint8_t handles;
    struct ble_ll_conn_sm *connsm;

    /* Check rate limit */
    if ((uint32_t)(g_ble_ll_next_num_comp_pkt_evt - os_time_get()) <
         NIMBLE_OPT_NUM_COMP_PKT_RATE) {
        return;
    }

    /* Iterate through all the active, created connections */
    evbuf = NULL;
    handles = 0;
    handle_ptr = NULL;
    comp_pkt_ptr = NULL;
    event_sent = 0;
    SLIST_FOREACH(connsm, &g_ble_ll_conn_active_list, act_sle) {
        /*
         * Only look at connections that we have sent a connection complete
         * event and that either has packets enqueued or has completed packets.
         */
        if ((connsm->conn_state != BLE_LL_CONN_STATE_IDLE) &&
            (connsm->completed_pkts || !STAILQ_EMPTY(&connsm->conn_txq))) {
            /* If no buffer, get one, If cant get one, leave. */
            if (!evbuf) {
                evbuf = os_memblock_get(&g_hci_cmd_pool);
                if (!evbuf) {
                    break;
                }
                handles = 0;
                handle_ptr = evbuf + 3;
                comp_pkt_ptr = handle_ptr + (sizeof(uint16_t) * 60);
            }

            /* Add handle and complete packets */
            htole16(handle_ptr, connsm->conn_handle);
            htole16(comp_pkt_ptr, connsm->completed_pkts);
            connsm->completed_pkts = 0;
            handle_ptr += sizeof(uint16_t);
            comp_pkt_ptr += sizeof(uint16_t);
            ++handles;

            /*
             * The event buffer should fit at least 255 bytes so this means we
             * can fit up to 60 handles per event (a little more but who cares).
             */
            if (handles == 60) {
                evbuf[0] = BLE_HCI_EVCODE_NUM_COMP_PKTS;
                evbuf[1] = (handles * 2 * sizeof(uint16_t)) + 1;
                evbuf[2] = handles;
                ble_ll_hci_event_send(evbuf);
                evbuf = NULL;
                handles = 0;
                event_sent = 1;
            }
        }
    }

    /* Send event if there is an event to send */
    if (evbuf) {
        evbuf[0] = BLE_HCI_EVCODE_NUM_COMP_PKTS;
        evbuf[1] = (handles * 2 * sizeof(uint16_t)) + 1;
        evbuf[2] = handles;
        if (handles < 60) {
            /* Make the pkt counts contiguous with handles */
            memmove(handle_ptr, evbuf + 3 + (60 * 2), handles * 2);
        }
        ble_ll_hci_event_send(evbuf);
        event_sent = 1;
    }

    if (event_sent) {
        g_ble_ll_next_num_comp_pkt_evt = os_time_get() +
            NIMBLE_OPT_NUM_COMP_PKT_RATE;
    }
}


/**
 * Send a disconnection complete event.
 *
 * NOTE: we currently only send this event when we have a reason to send it;
 * not when it fails.
 *
 * @param reason The BLE error code to send as a disconnect reason
 */
void
ble_ll_disconn_comp_event_send(struct ble_ll_conn_sm *connsm, uint8_t reason)
{
    uint8_t *evbuf;

    if (ble_ll_hci_is_event_enabled(BLE_HCI_EVCODE_DISCONN_CMP)) {
        evbuf = os_memblock_get(&g_hci_cmd_pool);
        if (evbuf) {
            evbuf[0] = BLE_HCI_EVCODE_DISCONN_CMP;
            evbuf[1] = BLE_HCI_EVENT_DISCONN_COMPLETE_LEN;
            evbuf[2] = BLE_ERR_SUCCESS;
            htole16(evbuf + 3, connsm->conn_handle);
            evbuf[5] = reason;
            ble_ll_hci_event_send(evbuf);
        }
    }
}

/**
 * Process the HCI command to create a connection.
 *
 * Context: Link Layer task (HCI command processing)
 *
 * @param cmdbuf
 *
 * @return int
 */
int
ble_ll_conn_create(uint8_t *cmdbuf)
{
    int rc;
    struct hci_create_conn ccdata;
    struct hci_create_conn *hcc;
    struct ble_ll_conn_sm *connsm;

    /* If we are already creating a connection we should leave */
    if (g_ble_ll_conn_create_sm) {
        return BLE_ERR_CMD_DISALLOWED;
    }

    /* If already enabled, we return an error */
    if (ble_ll_scan_enabled()) {
        return BLE_ERR_CMD_DISALLOWED;
    }

    /* Retrieve command data */
    hcc = &ccdata;
    hcc->scan_itvl = le16toh(cmdbuf);
    hcc->scan_window = le16toh(cmdbuf + 2);

    /* Check interval and window */
    if ((hcc->scan_itvl < BLE_HCI_SCAN_ITVL_MIN) ||
        (hcc->scan_itvl > BLE_HCI_SCAN_ITVL_MAX) ||
        (hcc->scan_window < BLE_HCI_SCAN_WINDOW_MIN) ||
        (hcc->scan_window > BLE_HCI_SCAN_WINDOW_MAX) ||
        (hcc->scan_itvl < hcc->scan_window)) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    /* Check filter policy */
    hcc->filter_policy = cmdbuf[4];
    if (hcc->filter_policy > BLE_HCI_INITIATOR_FILT_POLICY_MAX) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    /* Get peer address type and address only if no whitelist used */
    if (hcc->filter_policy == 0) {
        hcc->peer_addr_type = cmdbuf[5];
        if (hcc->peer_addr_type > BLE_HCI_CONN_PEER_ADDR_MAX) {
            return BLE_ERR_INV_HCI_CMD_PARMS;
        }

        /* XXX: not supported */
        if (hcc->peer_addr_type > BLE_HCI_CONN_PEER_ADDR_RANDOM) {
            return BLE_ERR_UNSUPPORTED;
        }

        memcpy(&hcc->peer_addr, cmdbuf + 6, BLE_DEV_ADDR_LEN);
    }

    /* Get own address type (used in connection request) */
    hcc->own_addr_type = cmdbuf[12];
    if (hcc->own_addr_type > BLE_HCI_ADV_OWN_ADDR_MAX) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    /* XXX: not supported */
    if (hcc->own_addr_type > BLE_HCI_ADV_OWN_ADDR_RANDOM) {
        return BLE_ERR_UNSUPPORTED;
    }

    /* Check connection interval, latency and supervision timeoout */
    hcc->conn_itvl_min = le16toh(cmdbuf + 13);
    hcc->conn_itvl_max = le16toh(cmdbuf + 15);
    hcc->conn_latency = le16toh(cmdbuf + 17);
    hcc->supervision_timeout = le16toh(cmdbuf + 19);
    rc = ble_ll_conn_hci_chk_conn_params(hcc->conn_itvl_min,
                                         hcc->conn_itvl_max,
                                         hcc->conn_latency,
                                         hcc->supervision_timeout);
    if (rc) {
        return rc;
    }

    /* Min/max connection event lengths */
    hcc->min_ce_len = le16toh(cmdbuf + 21);
    hcc->max_ce_len = le16toh(cmdbuf + 23);
    if (hcc->min_ce_len > hcc->max_ce_len) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    /* Make sure we can accept a connection! */
    connsm = ble_ll_conn_sm_get();
    if (connsm == NULL) {
        return BLE_ERR_CONN_LIMIT;
    }

    /* Initialize state machine in master role and start state machine */
    ble_ll_conn_master_init(connsm, hcc);
    ble_ll_conn_sm_new(connsm);

    /* Create the connection request */
    ble_ll_conn_req_pdu_make(connsm);

    /* Start scanning */
    rc = ble_ll_scan_initiator_start(hcc);
    if (rc) {
        SLIST_REMOVE(&g_ble_ll_conn_active_list,connsm,ble_ll_conn_sm,act_sle);
        STAILQ_INSERT_TAIL(&g_ble_ll_conn_free_list, connsm, free_stqe);
    } else {
        /* Set the connection state machine we are trying to create. */
        g_ble_ll_conn_create_sm = connsm;
    }

    return rc;
}

static int
ble_ll_conn_process_conn_params(uint8_t *cmdbuf, struct ble_ll_conn_sm *connsm)
{
    int rc;
    struct hci_conn_update *hcu;

    /* Retrieve command data */
    hcu = &connsm->conn_param_req;
    hcu->handle = connsm->conn_handle;
    hcu->conn_itvl_min = le16toh(cmdbuf + 2);
    hcu->conn_itvl_max = le16toh(cmdbuf + 4);
    hcu->conn_latency = le16toh(cmdbuf + 6);
    hcu->supervision_timeout = le16toh(cmdbuf + 8);
    hcu->min_ce_len = le16toh(cmdbuf + 10);
    hcu->max_ce_len = le16toh(cmdbuf + 12);

    /* Check that parameter values are in range */
    rc = ble_ll_conn_hci_chk_conn_params(hcu->conn_itvl_min,
                                         hcu->conn_itvl_max,
                                         hcu->conn_latency,
                                         hcu->supervision_timeout);

    /* Check valid min/max ce length */
    if (rc || (hcu->min_ce_len > hcu->max_ce_len)) {
        hcu->handle = 0;
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }
    return rc;
}

/**
 * Called when the host issues the read remote features command
 *
 * @param cmdbuf
 *
 * @return int
 */
int
ble_ll_conn_hci_read_rem_features(uint8_t *cmdbuf)
{
    uint16_t handle;
    struct ble_ll_conn_sm *connsm;

    /* If no connection handle exit with error */
    handle = le16toh(cmdbuf);
    connsm = ble_ll_conn_find_active_conn(handle);
    if (!connsm) {
        return BLE_ERR_UNK_CONN_ID;
    }

    /* See if we support this feature */
    if (connsm->conn_role == BLE_LL_CONN_ROLE_SLAVE) {
        if ((ble_ll_read_supp_features() & BLE_LL_FEAT_SLAVE_INIT) == 0) {
            return BLE_ERR_UNKNOWN_HCI_CMD;
        }
    }

    /* Start the control procedure */
    ble_ll_ctrl_proc_start(connsm, BLE_LL_CTRL_PROC_FEATURE_XCHG);

    return BLE_ERR_SUCCESS;
}

/**
 * Called to process a connection update command.
 *
 * @param cmdbuf
 *
 * @return int
 */
int
ble_ll_conn_hci_update(uint8_t *cmdbuf)
{
    int rc;
    uint8_t ctrl_proc;
    uint16_t handle;
    struct ble_ll_conn_sm *connsm;
    struct hci_conn_update *hcu;

    /*
     * XXX: must deal with slave not supporting this feature and using
     * conn update! Right now, we only check if WE support the connection
     * parameters request procedure. We dont check if the remote does.
     * We should also be able to deal with sending the parameter request,
     * getting an UNKOWN_RSP ctrl pdu and resorting to use normal
     * connection update procedure.
     */

    /* If no connection handle exit with error */
    handle = le16toh(cmdbuf);
    connsm = ble_ll_conn_find_active_conn(handle);
    if (!connsm) {
        return BLE_ERR_UNK_CONN_ID;
    }

    /* Better not have this procedure ongoing! */
    if (IS_PENDING_CTRL_PROC(connsm, BLE_LL_CTRL_PROC_CONN_PARAM_REQ) ||
        IS_PENDING_CTRL_PROC(connsm, BLE_LL_CTRL_PROC_CONN_UPDATE)) {
        return BLE_ERR_CMD_DISALLOWED;
    }

    /* See if we support this feature */
    if ((ble_ll_read_supp_features() & BLE_LL_FEAT_CONN_PARM_REQ) == 0) {
        if (connsm->conn_role == BLE_LL_CONN_ROLE_SLAVE) {
            return BLE_ERR_UNKNOWN_HCI_CMD;
        }
        ctrl_proc = BLE_LL_CTRL_PROC_CONN_UPDATE;
    } else {
        ctrl_proc = BLE_LL_CTRL_PROC_CONN_PARAM_REQ;
    }

    /*
     * If we are a slave and the master has initiated the procedure already
     * we should deny the slave request for now. If we are a master and the
     * slave has initiated the procedure, we need to send a reject to the
     * slave.
     */
    if (connsm->csmflags.cfbit.awaiting_host_reply) {
        if (connsm->conn_role == BLE_LL_CONN_ROLE_SLAVE) {
            return BLE_ERR_LMP_COLLISION;
        } else {
            connsm->csmflags.cfbit.awaiting_host_reply = 0;

            /* XXX: If this fails no reject ind will be sent! */
            ble_ll_ctrl_reject_ind_send(connsm, connsm->host_reply_opcode,
                                        BLE_ERR_LMP_COLLISION);
        }
    }

    /*
     * If we are a slave and the master has initiated the channel map
     * update procedure we should deny the slave request for now.
     */
    if (connsm->csmflags.cfbit.chanmap_update_scheduled) {
        if (connsm->conn_role == BLE_LL_CONN_ROLE_SLAVE) {
            return BLE_ERR_DIFF_TRANS_COLL;
        }
    }

    /* Retrieve command data */
    hcu = &connsm->conn_param_req;
    hcu->handle = handle;
    hcu->conn_itvl_min = le16toh(cmdbuf + 2);
    hcu->conn_itvl_max = le16toh(cmdbuf + 4);
    hcu->conn_latency = le16toh(cmdbuf + 6);
    hcu->supervision_timeout = le16toh(cmdbuf + 8);
    hcu->min_ce_len = le16toh(cmdbuf + 10);
    hcu->max_ce_len = le16toh(cmdbuf + 12);
    if (hcu->min_ce_len > hcu->max_ce_len) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    /* Check that parameter values are in range */
    rc = ble_ll_conn_hci_chk_conn_params(hcu->conn_itvl_min,
                                         hcu->conn_itvl_max,
                                         hcu->conn_latency,
                                         hcu->supervision_timeout);
    if (!rc) {
        /* Start the control procedure */
        ble_ll_ctrl_proc_start(connsm, ctrl_proc);
    }

    return rc;
}

int
ble_ll_conn_hci_param_reply(uint8_t *cmdbuf, int positive_reply)
{
    int rc;
    uint8_t ble_err;
    uint8_t *dptr;
    uint8_t rsp_opcode;
    uint8_t len;
    uint16_t handle;
    struct os_mbuf *om;
    struct ble_ll_conn_sm *connsm;

    /* See if we support this feature */
    if ((ble_ll_read_supp_features() & BLE_LL_FEAT_CONN_PARM_REQ) == 0) {
        return BLE_ERR_UNKNOWN_HCI_CMD;
    }

    /* If no connection handle exit with error */
    handle = le16toh(cmdbuf);

    /* If we dont have a handle we cant do anything */
    connsm = ble_ll_conn_find_active_conn(handle);
    if (!connsm) {
        return BLE_ERR_UNK_CONN_ID;
    }

    /* Make sure connection parameters are valid if this is a positive reply */
    rc = BLE_ERR_SUCCESS;
    ble_err = cmdbuf[2];
    if (positive_reply) {
        rc = ble_ll_conn_process_conn_params(cmdbuf, connsm);
        if (rc) {
            ble_err = BLE_ERR_CONN_PARMS;
        }
    }

    /* The connection should be awaiting a reply. If not, just discard */
    if (connsm->csmflags.cfbit.awaiting_host_reply) {
        /* Get a control packet buffer */
        if (positive_reply && (rc == BLE_ERR_SUCCESS)) {
            om = os_msys_get_pkthdr(BLE_LL_CTRL_MAX_PAYLOAD + 1,
                                    sizeof(struct ble_mbuf_hdr));
            if (om) {
                dptr = om->om_data;
                rsp_opcode = ble_ll_ctrl_conn_param_reply(connsm, dptr,
                                                          &connsm->conn_cp);
                dptr[0] = rsp_opcode;
                len = g_ble_ll_ctrl_pkt_lengths[rsp_opcode] + 1;
                ble_ll_conn_enqueue_pkt(connsm, om, BLE_LL_LLID_CTRL, len);
            }
        } else {
            /* XXX: check return code and deal */
            ble_ll_ctrl_reject_ind_send(connsm, connsm->host_reply_opcode,
                                        ble_err);
        }
        connsm->csmflags.cfbit.awaiting_host_reply = 0;

        /* XXX: if we cant get a buffer, what do we do? We need to remember
         * reason if it was a negative reply. We also would need to have
           some state to tell us this happened */
    }

    return rc;
}

/**
 * Called when HCI command to cancel a create connection command has been
 * received.
 *
 * Context: Link Layer (HCI command parser)
 *
 * @return int
 */
int
ble_ll_conn_create_cancel(void)
{
    int rc;
    struct ble_ll_conn_sm *connsm;

    /* XXX: BUG! I send the event before the command complete. Not good. */
    /*
     * If we receive this command and we have not got a connection
     * create command, we have to return disallowed. The spec does not say
     * what happens if the connection has already been established. We
     * return disallowed as well
     */
    connsm = g_ble_ll_conn_create_sm;
    if (connsm && (connsm->conn_state == BLE_LL_CONN_STATE_IDLE)) {
        /* stop scanning and end the connection event */
        g_ble_ll_conn_create_sm = NULL;
        ble_ll_scan_sm_stop(1);
        ble_ll_conn_end(connsm, BLE_ERR_UNK_CONN_ID);
        rc = BLE_ERR_SUCCESS;
    } else {
        /* If we are not attempting to create a connection*/
        rc = BLE_ERR_CMD_DISALLOWED;
    }

    return rc;
}

/**
 * Called to process a HCI disconnect command
 *
 * Context: Link Layer task (HCI command parser).
 *
 * @param cmdbuf
 *
 * @return int
 */
int
ble_ll_conn_hci_disconnect_cmd(uint8_t *cmdbuf)
{
    int rc;
    uint8_t reason;
    uint16_t handle;
    struct ble_ll_conn_sm *connsm;

    /* Check for valid parameters */
    handle = le16toh(cmdbuf);
    reason = cmdbuf[2];

    rc = BLE_ERR_INV_HCI_CMD_PARMS;
    if (handle <= BLE_LL_CONN_MAX_CONN_HANDLE) {
        /* Make sure reason is valid */
        switch (reason) {
        case BLE_ERR_AUTH_FAIL:
        case BLE_ERR_REM_USER_CONN_TERM:
        case BLE_ERR_RD_CONN_TERM_RESRCS:
        case BLE_ERR_RD_CONN_TERM_PWROFF:
        case BLE_ERR_UNSUPP_REM_FEATURE:
        case BLE_ERR_UNIT_KEY_PAIRING:
        case BLE_ERR_CONN_PARMS:
            connsm = ble_ll_conn_find_active_conn(handle);
            if (connsm) {
                /* Do not allow command if we are in process of disconnecting */
                if (connsm->disconnect_reason) {
                    rc = BLE_ERR_CMD_DISALLOWED;
                } else {
                    /* This control procedure better not be pending! */
                    assert(!IS_PENDING_CTRL_PROC(connsm, BLE_LL_CTRL_PROC_TERMINATE));

                    /* Record the disconnect reason */
                    connsm->disconnect_reason = reason;

                    /* Start this control procedure */
                    ble_ll_ctrl_terminate_start(connsm);

                    rc = BLE_ERR_SUCCESS;
                }
            } else {
                rc = BLE_ERR_UNK_CONN_ID;
            }
            break;
        default:
            break;
        }
    }

    return rc;
}

/**
 * Called to process a HCI disconnect command
 *
 * Context: Link Layer task (HCI command parser).
 *
 * @param cmdbuf
 *
 * @return int
 */
int
ble_ll_conn_hci_rd_rem_ver_cmd(uint8_t *cmdbuf)
{
    uint16_t handle;
    struct ble_ll_conn_sm *connsm;

    /* Check for valid parameters */
    handle = le16toh(cmdbuf);
    connsm = ble_ll_conn_find_active_conn(handle);
    if (!connsm) {
        return BLE_ERR_UNK_CONN_ID;
    }

    /* Return error if in progress */
    if (IS_PENDING_CTRL_PROC(connsm, BLE_LL_CTRL_PROC_VERSION_XCHG)) {
        return BLE_ERR_CMD_DISALLOWED;
    }

    /*
     * Start this control procedure. If we have already done this control
     * procedure we set the pending bit so that the host gets an event because
     * it is obviously expecting one (or would not have sent the command).
     * NOTE: we cant just send the event here. That would cause the event to
     * be queued before the command status.
     */
    if (!connsm->csmflags.cfbit.version_ind_sent) {
        ble_ll_ctrl_proc_start(connsm, BLE_LL_CTRL_PROC_VERSION_XCHG);
    } else {
        connsm->pending_ctrl_procs |= (1 << BLE_LL_CTRL_PROC_VERSION_XCHG);
    }

    return BLE_ERR_SUCCESS;
}

/**
 * Called to read the RSSI for a given connection handle
 *
 * @param cmdbuf
 * @param rspbuf
 * @param rsplen
 *
 * @return int
 */
int
ble_ll_conn_hci_rd_rssi(uint8_t *cmdbuf, uint8_t *rspbuf, uint8_t *rsplen)
{
    int rc;
    int8_t rssi;
    uint16_t handle;
    struct ble_ll_conn_sm *connsm;

    handle = le16toh(cmdbuf);
    connsm = ble_ll_conn_find_active_conn(handle);
    if (!connsm) {
        rssi = 127;
        rc = BLE_ERR_UNK_CONN_ID;
    } else {
        rssi = connsm->conn_rssi;
        rc = BLE_ERR_SUCCESS;
    }

    htole16(rspbuf, handle);
    rspbuf[2] = (uint8_t)rssi;
    *rsplen = 3;

    /* Place the RSSI of the connection into the response buffer */
    return rc;
}

/**
 * Called to read the current channel map of a connection
 *
 * @param cmdbuf
 * @param rspbuf
 * @param rsplen
 *
 * @return int
 */
int
ble_ll_conn_hci_rd_chan_map(uint8_t *cmdbuf, uint8_t *rspbuf, uint8_t *rsplen)
{
    int rc;
    uint16_t handle;
    struct ble_ll_conn_sm *connsm;

    handle = le16toh(cmdbuf);
    connsm = ble_ll_conn_find_active_conn(handle);
    if (!connsm) {
        rc = BLE_ERR_UNK_CONN_ID;
    } else {
        if (connsm->csmflags.cfbit.chanmap_update_scheduled) {
            memcpy(rspbuf + 2, &connsm->req_chanmap[0], BLE_LL_CONN_CHMAP_LEN);
        } else {
            memcpy(rspbuf + 2, &connsm->chanmap[0], BLE_LL_CONN_CHMAP_LEN);
        }
        rc = BLE_ERR_SUCCESS;
    }

    htole16(rspbuf, handle);
    *rsplen = sizeof(uint16_t) + BLE_LL_CONN_CHMAP_LEN;
    return rc;
}

/**
 * Called when the host issues the LE command "set host channel classification"
 *
 * @param cmdbuf
 *
 * @return int
 */
int
ble_ll_conn_hci_set_chan_class(uint8_t *cmdbuf)
{
    int rc;
    uint8_t num_used_chans;

    /*
     * The HCI command states that the host is allowed to mask in just one
     * channel but the Link Layer needs minimum two channels to operate. So
     * I will not allow this command if there are less than 2 channels masked.
     */
    rc = BLE_ERR_SUCCESS;
    num_used_chans = ble_ll_conn_calc_used_chans(cmdbuf);
    if ((num_used_chans < 2) || ((cmdbuf[4] & 0xe0) != 0)) {
        rc = BLE_ERR_INV_HCI_CMD_PARMS;
    }

    /* Set the host channel mask */
    ble_ll_conn_set_global_chanmap(num_used_chans, cmdbuf);
    return rc;
}

#if (BLE_LL_CFG_FEAT_DATA_LEN_EXT == 1)
int
ble_ll_conn_hci_set_data_len(uint8_t *cmdbuf, uint8_t *rspbuf, uint8_t *rsplen)
{
    int rc;
    uint16_t handle;
    uint16_t txoctets;
    uint16_t txtime;
    struct ble_ll_conn_sm *connsm;

    /* Find connection */
    handle = le16toh(cmdbuf);
    connsm = ble_ll_conn_find_active_conn(handle);
    if (!connsm) {
        rc = BLE_ERR_UNK_CONN_ID;
    } else {
        txoctets = le16toh(cmdbuf + 2);
        txtime = le16toh(cmdbuf + 4);

        /* Make sure it is valid */
        if (!ble_ll_chk_txrx_octets(txoctets) ||
            !ble_ll_chk_txrx_time(txtime)) {
            rc = BLE_ERR_INV_HCI_CMD_PARMS;
        } else {
            rc = BLE_ERR_SUCCESS;
        }

        /* XXX: should I check against max supported? I think so */

        /*
         * XXX: For now; we will simply ignore what the host asks as we are
         * allowed to do so by the spec.
         */
    }

    htole16(rspbuf, handle);
    *rsplen = sizeof(uint16_t);
    return rc;
}
#endif

#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
/**
 * LE start encrypt command
 *
 * @param cmdbuf
 *
 * @return int
 */
int
ble_ll_conn_hci_le_start_encrypt(uint8_t *cmdbuf)
{
    int rc;
    uint16_t handle;
    struct ble_ll_conn_sm *connsm;

    handle = le16toh(cmdbuf);
    connsm = ble_ll_conn_find_active_conn(handle);
    if (!connsm) {
        rc = BLE_ERR_UNK_CONN_ID;
    } else if (connsm->conn_role == BLE_LL_CONN_ROLE_SLAVE) {
        rc = BLE_ERR_UNSPECIFIED;
    } else if (connsm->cur_ctrl_proc == BLE_LL_CTRL_PROC_ENCRYPT) {
        /*
         * The specification does not say what to do here but the host should
         * not be telling us to start encryption while we are in the process
         * of honoring a previous start encrypt.
         */
        rc = BLE_ERR_CMD_DISALLOWED;
    } else {
        /* Start the control procedure */
        connsm->enc_data.host_rand_num = le64toh(cmdbuf + 2);
        connsm->enc_data.enc_div = le16toh(cmdbuf + 10);
        swap_buf(connsm->enc_data.enc_block.key, cmdbuf + 12, 16);
        ble_ll_ctrl_proc_start(connsm, BLE_LL_CTRL_PROC_ENCRYPT);
        rc = BLE_ERR_SUCCESS;
    }

    return rc;
}

/**
 * Called to process the LE long term key reply.
 *
 * Context: Link Layer Task.
 *
 * @param cmdbuf
 * @param rspbuf
 * @param ocf
 *
 * @return int
 */
int
ble_ll_conn_hci_le_ltk_reply(uint8_t *cmdbuf, uint8_t *rspbuf, uint8_t ocf)
{
    int rc;
    uint16_t handle;
    struct ble_ll_conn_sm *connsm;

    /* Find connection handle */
    handle = le16toh(cmdbuf);
    connsm = ble_ll_conn_find_active_conn(handle);
    if (!connsm) {
        rc = BLE_ERR_UNK_CONN_ID;
        goto ltk_key_cmd_complete;
    }

    /* Should never get this if we are a master! */
    if (connsm->conn_role == BLE_LL_CONN_ROLE_MASTER) {
        rc = BLE_ERR_UNSPECIFIED;
        goto ltk_key_cmd_complete;
    }

    /* The connection should be awaiting a reply. If not, just discard */
    if (connsm->enc_data.enc_state == CONN_ENC_S_LTK_REQ_WAIT) {
        if (ocf == BLE_HCI_OCF_LE_LT_KEY_REQ_REPLY) {
            swap_buf(connsm->enc_data.enc_block.key, cmdbuf + 2, 16);
            ble_ll_calc_session_key(connsm);
            ble_ll_ctrl_start_enc_send(connsm, BLE_LL_CTRL_START_ENC_REQ);
        } else {
            /* We received a negative reply! Send REJECT_IND */
            ble_ll_ctrl_reject_ind_send(connsm, BLE_LL_CTRL_ENC_REQ,
                                        BLE_ERR_PINKEY_MISSING);
            connsm->enc_data.enc_state = CONN_ENC_S_LTK_NEG_REPLY;
        }
    }
    rc = BLE_ERR_SUCCESS;

ltk_key_cmd_complete:
    htole16(rspbuf, handle);
    return rc;
}
#endif
