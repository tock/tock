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
#include "os/os.h"
#include "bsp/bsp.h"
#include "ble/xcvr.h"
#include "nimble/ble.h"
#include "nimble/nimble_opt.h"
#include "nimble/hci_common.h"
#include "controller/ble_phy.h"
#include "controller/ble_ll.h"
#include "controller/ble_ll_adv.h"
#include "controller/ble_ll_sched.h"
#include "controller/ble_ll_scan.h"
#include "controller/ble_ll_whitelist.h"
#include "ble_ll_conn_priv.h"
#include "hal/hal_cputime.h"
#include "hal/hal_gpio.h"

/* XXX: TODO
 * 1) Need to look at advertising and scan request PDUs. Do I allocate these
 * once? Do I use a different pool for smaller ones? Do I statically declare
 * them?
 * 2) Only public device addresses supported right now.
 * 3) How do features get supported? What happens if device does not support
 * advertising? (for example)
 * 4) How to determine the advertising interval we will actually use. As of
 * now, we set it to max.
 * 5) How does the advertising channel tx power get set? I dont implement
 * that currently.
 */

/*
 * Advertising state machine
 *
 * The advertising state machine data structure.
 *
 *  adv_pdu_len
 *      The length of the advertising PDU that will be sent. This does not
 *      include the preamble, access address and CRC.
 */
struct ble_ll_adv_sm
{
    uint8_t enabled;
    uint8_t adv_type;
    uint8_t adv_len;
    uint8_t adv_chanmask;
    uint8_t adv_filter_policy;
    uint8_t own_addr_type;
    uint8_t peer_addr_type;
    uint8_t adv_chan;
    uint8_t scan_rsp_len;
    uint8_t adv_pdu_len;
    uint16_t adv_itvl_min;
    uint16_t adv_itvl_max;
    uint32_t adv_itvl_usecs;
    uint32_t adv_event_start_time;
    uint32_t adv_pdu_start_time;
    uint32_t adv_dir_hd_end_time;
    uint8_t initiator_addr[BLE_DEV_ADDR_LEN];
    uint8_t adv_data[BLE_ADV_DATA_MAX_LEN];
    uint8_t scan_rsp_data[BLE_SCAN_RSP_DATA_MAX_LEN];
    struct os_mbuf *scan_rsp_pdu;
    struct os_event adv_txdone_ev;
    struct ble_ll_sched_item adv_sch;
};

/* The advertising state machine global object */
struct ble_ll_adv_sm g_ble_ll_adv_sm;

/*
 * Worst case time needed for scheduled advertising item. This is the longest
 * possible time to receive a scan request and send a scan response (with the
 * appropriate IFS time between them). This number is calculated using the
 * following formula: IFS + SCAN_REQ + IFS + SCAN_RSP = 150 + 176 + 150 + 376.
 * Note: worst case time to tx adv, rx scan req and send scan rsp is 1228 usecs.
 * This assumes maximum sized advertising PDU and scan response PDU.
 *
 * For connectable advertising events no scan request is allowed. In this case
 * we just need to receive a connect request PDU: IFS + CONNECT_REQ = 150 + 352.
 * Note: worst-case is 376 + 150 + 352 = 878 usecs
 *
 * NOTE: The advertising PDU transmit time is NOT included here since we know
 * how long that will take (worst-case is 376 usecs).
 */
#define BLE_LL_ADV_SCHED_MAX_USECS          (852)
#define BLE_LL_ADV_DIRECT_SCHED_MAX_USECS   (502)

/**
 * Calculate the first channel that we should advertise upon when we start
 * an advertising event.
 *
 * @param advsm
 *
 * @return uint8_t The number of the first channel usable for advertising.
 */
static uint8_t
ble_ll_adv_first_chan(struct ble_ll_adv_sm *advsm)
{
    uint8_t adv_chan;

    /* Set first advertising channel */
    if (advsm->adv_chanmask & 0x01) {
        adv_chan = BLE_PHY_ADV_CHAN_START;
    } else if (advsm->adv_chanmask & 0x02) {
        adv_chan = BLE_PHY_ADV_CHAN_START + 1;
    } else {
        adv_chan = BLE_PHY_ADV_CHAN_START + 2;
    }

    return adv_chan;
}

/**
 * Compares the advertiser address in an advertising PDU (scan request or
 * connect request) with our address to see if there is a match
 *
 * @param rxbuf Pointer to received PDU
 *
 * @return int Returns memcmp return values (0 = match).
 */
static int
ble_ll_adv_addr_cmp(uint8_t *rxbuf)
{
    int rc;
    uint8_t *adva;
    uint8_t *our_addr;
    uint8_t rxaddr_type;

    /* Determine if this is addressed to us */
    rxaddr_type = rxbuf[0] & BLE_ADV_PDU_HDR_RXADD_MASK;
    if (rxaddr_type) {
        our_addr = g_random_addr;
    } else {
        our_addr = g_dev_addr;
    }

    adva = rxbuf + BLE_LL_PDU_HDR_LEN + BLE_DEV_ADDR_LEN;
    rc = memcmp(our_addr, adva, BLE_DEV_ADDR_LEN);

    return rc;
}

/**
 * Create the advertising PDU
 *
 * @param advsm Pointer to advertisement state machine
 */
static void
ble_ll_adv_pdu_make(struct ble_ll_adv_sm *advsm, struct os_mbuf *m)
{
    int         is_direct_adv;
    uint8_t     adv_data_len;
    uint8_t     *dptr;
    uint8_t     pdulen;
    uint8_t     pdu_type;
    uint8_t     *addr;

    /* assume this is not a direct ind */
    adv_data_len = advsm->adv_len;
    pdulen = BLE_DEV_ADDR_LEN + adv_data_len;
    is_direct_adv = 0;

    /* Must be an advertising type! */
    switch (advsm->adv_type) {
    case BLE_HCI_ADV_TYPE_ADV_IND:
        pdu_type = BLE_ADV_PDU_TYPE_ADV_IND;
        break;

    case BLE_HCI_ADV_TYPE_ADV_NONCONN_IND:
        pdu_type = BLE_ADV_PDU_TYPE_ADV_NONCONN_IND;
        break;

    case BLE_HCI_ADV_TYPE_ADV_SCAN_IND:
        pdu_type = BLE_ADV_PDU_TYPE_ADV_SCAN_IND;
        break;

    case BLE_HCI_ADV_TYPE_ADV_DIRECT_IND_HD:
    case BLE_HCI_ADV_TYPE_ADV_DIRECT_IND_LD:
        is_direct_adv = 1;
        pdu_type = BLE_ADV_PDU_TYPE_ADV_DIRECT_IND;
        adv_data_len = 0;
        pdulen = BLE_ADV_DIRECT_IND_LEN;

        if (advsm->peer_addr_type == BLE_HCI_ADV_PEER_ADDR_RANDOM) {
            pdu_type |= BLE_ADV_PDU_HDR_RXADD_RAND;
        }
        break;

        /* Set these to avoid compiler warnings */
    default:
        pdulen = 0;
        pdu_type = 0;
        adv_data_len = 0xFF;
        break;
    }

    /* An invalid advertising data length indicates a memory overwrite */
    assert(adv_data_len <= BLE_ADV_DATA_MAX_LEN);

    /* Set the PDU length in the state machine (includes header) */
    advsm->adv_pdu_len = pdulen + BLE_LL_PDU_HDR_LEN;

    /* Construct scan response */
    if (advsm->own_addr_type == BLE_HCI_ADV_OWN_ADDR_PUBLIC) {
        addr = g_dev_addr;
    } else if (advsm->own_addr_type == BLE_HCI_ADV_OWN_ADDR_RANDOM) {
        pdu_type |= BLE_ADV_PDU_HDR_TXADD_RAND;
        addr = g_random_addr;
    } else {
        /* XXX: unsupported for now. Should never happen */
        addr = NULL;
        assert(0);
    }

    /* Get the advertising PDU and initialize it*/
    ble_ll_mbuf_init(m, pdulen, pdu_type);

    /* Construct advertisement */
    dptr = m->om_data;
    memcpy(dptr, addr, BLE_DEV_ADDR_LEN);
    dptr += BLE_DEV_ADDR_LEN;

    /* For ADV_DIRECT_IND, we need to put initiators address in there */
    if (is_direct_adv) {
        memcpy(dptr, advsm->initiator_addr, BLE_DEV_ADDR_LEN);
    }

    /* Copy in advertising data, if any */
    if (adv_data_len != 0) {
        memcpy(dptr, advsm->adv_data, adv_data_len);
    }
}

/**
 * Create a scan response PDU
 *
 * @param advsm
 */
static void
ble_ll_adv_scan_rsp_pdu_make(struct ble_ll_adv_sm *advsm)
{
    uint8_t     scan_rsp_len;
    uint8_t     *dptr;
    uint8_t     *addr;
    uint8_t     pdulen;
    uint8_t     hdr;
    struct os_mbuf *m;

    /* Make sure that the length is valid */
    scan_rsp_len = advsm->scan_rsp_len;
    assert(scan_rsp_len <= BLE_SCAN_RSP_DATA_MAX_LEN);

    /* Set PDU payload length */
    pdulen = BLE_DEV_ADDR_LEN + scan_rsp_len;

    /* Get the advertising PDU */
    m = advsm->scan_rsp_pdu;
    assert(m != NULL);

    /* Construct scan response */
    if (advsm->own_addr_type == BLE_HCI_ADV_OWN_ADDR_PUBLIC) {
        hdr = BLE_ADV_PDU_TYPE_SCAN_RSP;
        addr = g_dev_addr;
    } else if (advsm->own_addr_type == BLE_HCI_ADV_OWN_ADDR_RANDOM) {
        hdr = BLE_ADV_PDU_TYPE_SCAN_RSP | BLE_ADV_PDU_HDR_TXADD_RAND;
        addr = g_random_addr;
    } else {
        /* XXX: unsupported for now  */
        hdr = 0;
        addr = NULL;
        assert(0);
    }

    /* Set BLE transmit header */
    ble_ll_mbuf_init(m, pdulen, hdr);

    /* Put our address into buffer */
    dptr = m->om_data;
    memcpy(dptr, addr, BLE_DEV_ADDR_LEN);

    /* Copy in scan response data, if any */
    if (scan_rsp_len != 0) {
        memcpy(dptr + BLE_DEV_ADDR_LEN, advsm->scan_rsp_data, scan_rsp_len);
    }
}

/**
 * Called to indicate the advertising event is over.
 *
 * Context: Interrupt
 *
 * @param advsm
 *
 */
static void
ble_ll_adv_tx_done(void *arg)
{
    struct ble_ll_adv_sm *advsm;

    advsm = (struct ble_ll_adv_sm *)arg;
    os_eventq_put(&g_ble_ll_data.ll_evq, &advsm->adv_txdone_ev);

    ble_ll_log(BLE_LL_LOG_ID_ADV_TXDONE, ble_ll_state_get(), 0, 0);

    ble_ll_state_set(BLE_LL_STATE_STANDBY);
}

/**
 * This is the scheduler callback (called from interrupt context) which
 * transmits an advertisement.
 *
 * Context: Interrupt (scheduler)
 *
 * @param sch
 *
 * @return int
 */
static int
ble_ll_adv_tx_start_cb(struct ble_ll_sched_item *sch)
{
    int rc;
    uint8_t end_trans;
    uint32_t txstart;
    struct ble_ll_adv_sm *advsm;
    struct os_mbuf *adv_pdu;

    /* Get the state machine for the event */
    advsm = (struct ble_ll_adv_sm *)sch->cb_arg;

    /* Set channel */
    rc = ble_phy_setchan(advsm->adv_chan, 0, 0);
    assert(rc == 0);

    /* Set transmit start time. */
    txstart = sch->start_time + XCVR_PROC_DELAY_USECS;
    rc = ble_phy_tx_set_start_time(txstart);
    if (rc) {
        STATS_INC(ble_ll_stats, adv_late_starts);
        goto adv_tx_done;
    }

#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
    /* XXX: automatically do this in the phy based on channel? */
    ble_phy_encrypt_disable();
#endif

    /* Set phy mode based on type of advertisement */
    if (advsm->adv_type == BLE_HCI_ADV_TYPE_ADV_NONCONN_IND) {
        end_trans = BLE_PHY_TRANSITION_NONE;
        ble_phy_set_txend_cb(ble_ll_adv_tx_done, advsm);
    } else {
        end_trans = BLE_PHY_TRANSITION_TX_RX;
        ble_phy_set_txend_cb(NULL, NULL);
    }

    /* Get an advertising mbuf (packet header)  */
    adv_pdu = os_msys_get_pkthdr(BLE_MBUF_PAYLOAD_SIZE,
                                 sizeof(struct ble_mbuf_hdr));
    if (!adv_pdu) {
        ble_phy_disable();
        goto adv_tx_done;
    }

    ble_ll_adv_pdu_make(advsm, adv_pdu);

    /* Transmit advertisement */
    rc = ble_phy_tx(adv_pdu, end_trans);
    os_mbuf_free_chain(adv_pdu);
    if (rc) {
        goto adv_tx_done;
    }

    /* Enable/disable whitelisting based on filter policy */
    if (advsm->adv_filter_policy != BLE_HCI_ADV_FILT_NONE) {
        ble_ll_whitelist_enable();
    } else {
        ble_ll_whitelist_disable();
    }

    /* Set link layer state to advertising */
    ble_ll_state_set(BLE_LL_STATE_ADV);

    /* Count # of adv. sent */
    STATS_INC(ble_ll_stats, adv_txg);

    return BLE_LL_SCHED_STATE_RUNNING;

adv_tx_done:
    ble_ll_adv_tx_done(advsm);
    return BLE_LL_SCHED_STATE_DONE;
}

static void
ble_ll_adv_set_sched(struct ble_ll_adv_sm *advsm, int sched_new)
{
    uint32_t max_usecs;
    struct ble_ll_sched_item *sch;

    sch = &advsm->adv_sch;
    sch->cb_arg = advsm;
    sch->sched_cb = ble_ll_adv_tx_start_cb;
    sch->sched_type = BLE_LL_SCHED_TYPE_ADV;

    /* Set end time to maximum time this schedule item may take */
    max_usecs = BLE_TX_DUR_USECS_M(advsm->adv_pdu_len);
    switch (advsm->adv_type) {
    case BLE_HCI_ADV_TYPE_ADV_DIRECT_IND_LD:
    case BLE_HCI_ADV_TYPE_ADV_DIRECT_IND_HD:
        max_usecs += BLE_LL_ADV_DIRECT_SCHED_MAX_USECS;
        break;
    case BLE_HCI_ADV_TYPE_ADV_IND:
    case BLE_HCI_ADV_TYPE_ADV_SCAN_IND:
        max_usecs += BLE_LL_ADV_SCHED_MAX_USECS;
        break;
    default:
        break;
    }

    /*
     * XXX: For now, just schedule some additional time so we insure we have
     * enough time to do everything we want.
     */
    max_usecs += XCVR_PROC_DELAY_USECS;

    if (sched_new) {
        /*
         * We have to add the scheduling delay and tx start delay to the max
         * time of the event since the pdu does not start at the scheduled start.
         */
        max_usecs += XCVR_TX_SCHED_DELAY_USECS;
        sch->start_time = cputime_get32();
        sch->end_time = sch->start_time + cputime_usecs_to_ticks(max_usecs);
    } else {
        sch->start_time = advsm->adv_pdu_start_time -
            cputime_usecs_to_ticks(XCVR_TX_SCHED_DELAY_USECS);
        sch->end_time = advsm->adv_pdu_start_time +
            cputime_usecs_to_ticks(max_usecs);
    }
}

/**
 * Called when advertising need to be halted. This normally should not be called
 * and is only called when a scheduled item executes but advertising is still
 * running.
 *
 */
void
ble_ll_adv_halt(void)
{
    ble_ll_adv_tx_done(&g_ble_ll_adv_sm);
}

/**
 * Called by the HCI command parser when a set advertising parameters command
 * has been received.
 *
 * Context: Link Layer task (HCI command parser)
 *
 * @param cmd
 *
 * @return int
 */
int
ble_ll_adv_set_adv_params(uint8_t *cmd)
{
    uint8_t adv_type;
    uint8_t adv_filter_policy;
    uint8_t adv_chanmask;
    uint8_t own_addr_type;
    uint8_t peer_addr_type;
    uint16_t adv_itvl_min;
    uint16_t adv_itvl_max;
    uint16_t min_itvl;
    struct ble_ll_adv_sm *advsm;

    /* If already enabled, we return an error */
    advsm = &g_ble_ll_adv_sm;
    if (advsm->enabled) {
        return BLE_ERR_CMD_DISALLOWED;
    }

    /* Make sure intervals are OK (along with advertising type */
    adv_itvl_min = le16toh(cmd);
    adv_itvl_max = le16toh(cmd + 2);
    adv_type = cmd[4];

    /*
     * Get the filter policy now since we will ignore it if we are doing
     * directed advertising
     */
    adv_filter_policy = cmd[14];

    /* Assume min interval based on low duty cycle/indirect advertising */
    min_itvl = BLE_LL_ADV_ITVL_MIN;

    switch (adv_type) {
    /* Fall through intentional */
    case BLE_HCI_ADV_TYPE_ADV_DIRECT_IND_HD:
        /* Ignore min/max interval */
        min_itvl = 0;
        adv_itvl_min = 0;
        adv_itvl_max = 0;

    case BLE_HCI_ADV_TYPE_ADV_DIRECT_IND_LD:
        adv_filter_policy = BLE_HCI_ADV_FILT_NONE;
        memcpy(advsm->initiator_addr, cmd + 7, BLE_DEV_ADDR_LEN);
        break;
    case BLE_HCI_ADV_TYPE_ADV_IND:
        /* Nothing to do */
        break;
    case BLE_HCI_ADV_TYPE_ADV_NONCONN_IND:
    case BLE_HCI_ADV_TYPE_ADV_SCAN_IND:
        min_itvl = BLE_LL_ADV_ITVL_NONCONN_MIN;
        break;
    default:
        min_itvl = 0xFFFF;
        break;
    }

    /* Make sure interval minimum is valid for the advertising type */
    if ((adv_itvl_min > adv_itvl_max) || (adv_itvl_min < min_itvl) ||
        (adv_itvl_min > BLE_HCI_ADV_ITVL_MAX) ||
        (adv_itvl_max > BLE_HCI_ADV_ITVL_MAX)) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    /* Check own and peer address type */
    own_addr_type =  cmd[5];
    peer_addr_type = cmd[6];

    if ((own_addr_type > BLE_HCI_ADV_OWN_ADDR_MAX) ||
        (peer_addr_type > BLE_HCI_ADV_PEER_ADDR_MAX)) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    /*
     * XXX: return unsupported feature if own is address is not public or
     * static random as we have not implemented non-static random addresses.
     */
    if (own_addr_type > BLE_HCI_ADV_OWN_ADDR_RANDOM) {
        return BLE_ERR_UNSUPPORTED;
    }

    /* There are only three adv channels, so check for any outside the range */
    adv_chanmask = cmd[13];
    if (((adv_chanmask & 0xF8) != 0) || (adv_chanmask == 0)) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    /* Check for valid filter policy */
    if (adv_filter_policy > BLE_HCI_ADV_FILT_MAX) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    /* Fill out rest of advertising state machine */
    advsm->own_addr_type = own_addr_type;
    advsm->peer_addr_type = peer_addr_type;
    advsm->adv_filter_policy = adv_filter_policy;
    advsm->adv_chanmask = adv_chanmask;
    advsm->adv_itvl_min = adv_itvl_min;
    advsm->adv_itvl_max = adv_itvl_max;
    advsm->adv_type = adv_type;

    return 0;
}

/**
 * Stop advertising state machine
 *
 * Context: Link Layer task.
 *
 * @param advsm
 */
static void
ble_ll_adv_sm_stop(struct ble_ll_adv_sm *advsm)
{
    os_sr_t sr;

    if (advsm->enabled) {
        /* Remove any scheduled advertising items */
        ble_ll_sched_rmv_elem(&advsm->adv_sch);
        os_eventq_remove(&g_ble_ll_data.ll_evq, &advsm->adv_txdone_ev);

        /* Set to standby if we are no longer advertising */
        OS_ENTER_CRITICAL(sr);
        if (ble_ll_state_get() == BLE_LL_STATE_ADV) {
            ble_ll_wfr_disable();
            ble_ll_state_set(BLE_LL_STATE_STANDBY);
        }
        OS_EXIT_CRITICAL(sr);

        /* Disable advertising */
        advsm->enabled = 0;
    }
}

/**
 * Start the advertising state machine. This is called when the host sends
 * the "enable advertising" command and is not called again while in the
 * advertising state.
 *
 * Context: Link-layer task.
 *
 * @param advsm Pointer to advertising state machine
 *
 * @return int
 */
static int
ble_ll_adv_sm_start(struct ble_ll_adv_sm *advsm)
{
    uint8_t adv_chan;

    /*
     * This is not in the specification. I will reject the command with a
     * command disallowed error if no random address has been sent by the
     * host. All the parameter errors refer to the command
     * parameter (which in this case is just enable or disable) so that
     * is why I chose command disallowed.
     */
    if (advsm->own_addr_type == BLE_HCI_ADV_OWN_ADDR_RANDOM) {
        if (!ble_ll_is_valid_random_addr(g_random_addr)) {
            return BLE_ERR_CMD_DISALLOWED;
        }
    }

    /* Set flag telling us that advertising is enabled */
    advsm->enabled = 1;

    /* Determine the advertising interval we will use */
    if (advsm->adv_type == BLE_HCI_ADV_TYPE_ADV_DIRECT_IND_HD) {
        /* Set it to max. allowed for high duty cycle advertising */
        advsm->adv_itvl_usecs = BLE_LL_ADV_PDU_ITVL_HD_MS_MAX;
    } else {
        advsm->adv_itvl_usecs = (uint32_t)advsm->adv_itvl_max;
        advsm->adv_itvl_usecs *= BLE_LL_ADV_ITVL;
    }

    /* Create scan response PDU (if needed) */
    if (advsm->adv_type != BLE_HCI_ADV_TYPE_ADV_NONCONN_IND) {
        ble_ll_adv_scan_rsp_pdu_make(advsm);
    }

    /* Set first advertising channel */
    adv_chan = ble_ll_adv_first_chan(advsm);
    advsm->adv_chan = adv_chan;

    /*
     * Schedule advertising. We set the initial schedule start and end
     * times to the earliest possible start/end.
     */
    ble_ll_adv_set_sched(advsm, 1);
    ble_ll_sched_adv_new(&advsm->adv_sch);

    return BLE_ERR_SUCCESS;
}

void
ble_ll_adv_scheduled(uint32_t sch_start)
{
    struct ble_ll_adv_sm *advsm;

    advsm = &g_ble_ll_adv_sm;

    /* The event start time is when we start transmission of the adv PDU */
    advsm->adv_event_start_time = sch_start +
        cputime_usecs_to_ticks(XCVR_TX_SCHED_DELAY_USECS);

    advsm->adv_pdu_start_time = advsm->adv_event_start_time;

    /*
     * Set the time at which we must end directed, high-duty cycle advertising.
     * Does not matter that we calculate this value if we are not doing high
     * duty cycle advertising.
     */
    advsm->adv_dir_hd_end_time = advsm->adv_event_start_time +
        cputime_usecs_to_ticks(BLE_LL_ADV_STATE_HD_MAX * 1000);
}

/**
 * Called when the LE HCI command read advertising channel tx power command
 * has been received. Returns the current advertising transmit power.
 *
 * Context: Link Layer task (HCI command parser)
 *
 * @return int
 */
int
ble_ll_adv_read_txpwr(uint8_t *rspbuf, uint8_t *rsplen)
{
    rspbuf[0] = NIMBLE_OPT_LL_TX_PWR_DBM;
    *rsplen = 1;
    return BLE_ERR_SUCCESS;
}

/**
 * Turn advertising on/off.
 *
 * Context: Link Layer task
 *
 * @param cmd
 *
 * @return int
 */
int
ble_ll_adv_set_enable(uint8_t *cmd)
{
    int rc;
    uint8_t enable;
    struct ble_ll_adv_sm *advsm;

    advsm = &g_ble_ll_adv_sm;

    rc = 0;
    enable = cmd[0];
    if (enable == 1) {
        /* If already enabled, do nothing */
        if (!advsm->enabled) {
            /* Start the advertising state machine */
            rc = ble_ll_adv_sm_start(advsm);
        }
    } else if (enable == 0) {
        ble_ll_adv_sm_stop(advsm);
    } else {
        rc = BLE_ERR_INV_HCI_CMD_PARMS;
    }

    return rc;
}

/**
 * Set the scan response data that the controller will send.
 *
 * @param cmd
 * @param len
 *
 * @return int
 */
int
ble_ll_adv_set_scan_rsp_data(uint8_t *cmd, uint8_t len)
{
    uint8_t datalen;
    os_sr_t sr;
    struct ble_ll_adv_sm *advsm;

    /* Check for valid scan response data length */
    datalen = cmd[0];
    if (datalen > BLE_SCAN_RSP_DATA_MAX_LEN) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    /* Copy the new data into the advertising structure. */
    advsm = &g_ble_ll_adv_sm;
    advsm->scan_rsp_len = datalen;
    memcpy(advsm->scan_rsp_data, cmd + 1, datalen);

    /* Re-make the scan response PDU since data may have changed */
    OS_ENTER_CRITICAL(sr);
    /*
     * XXX: there is a chance, even with interrupts disabled, that
     * we are transmitting the scan response PDU while writing to it.
     */
    ble_ll_adv_scan_rsp_pdu_make(advsm);
    OS_EXIT_CRITICAL(sr);

    return 0;
}

/**
 * Called by the LL HCI command parser when a set advertising
 * data command has been sent from the host to the controller.
 *
 * @param cmd Pointer to command data
 * @param len Length of command data
 *
 * @return int 0: success; BLE_ERR_INV_HCI_CMD_PARMS otherwise.
 */
int
ble_ll_adv_set_adv_data(uint8_t *cmd, uint8_t len)
{
    uint8_t datalen;
    struct ble_ll_adv_sm *advsm;

    /* Check for valid advertising data length */
    datalen = cmd[0];
    if (datalen > BLE_ADV_DATA_MAX_LEN) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    /* Copy the new data into the advertising structure. */
    advsm = &g_ble_ll_adv_sm;
    advsm->adv_len = datalen;
    memcpy(advsm->adv_data, cmd + 1, datalen);

    return 0;
}

/**
 * Called when the LL receives a scan request or connection request
 *
 * Context: Called from interrupt context.
 *
 * @param rxbuf
 *
 * @return -1: request not for us or is a connect request.
 *          0: request (scan) is for us and we successfully went from rx to tx.
 *        > 0: PHY error attempting to go from rx to tx.
 */
static int
ble_ll_adv_rx_req(uint8_t pdu_type, struct os_mbuf *rxpdu)
{
    int rc;
    uint8_t chk_whitelist;
    uint8_t txadd;
    uint8_t *rxbuf;
    struct ble_mbuf_hdr *ble_hdr;
    struct ble_ll_adv_sm *advsm;

    rxbuf = rxpdu->om_data;
    if (ble_ll_adv_addr_cmp(rxbuf)) {
        return -1;
    }

    /* Set device match bit if we are whitelisting */
    advsm = &g_ble_ll_adv_sm;
    if (pdu_type == BLE_ADV_PDU_TYPE_SCAN_REQ) {
        chk_whitelist = advsm->adv_filter_policy & 1;
    } else {
        chk_whitelist = advsm->adv_filter_policy & 2;
    }

    /* Set device match bit if we are whitelisting */
    ble_hdr = BLE_MBUF_HDR_PTR(rxpdu);
    if (chk_whitelist) {
        /* Get the scanners address type */
        if (rxbuf[0] & BLE_ADV_PDU_HDR_TXADD_MASK) {
            txadd = BLE_ADDR_TYPE_RANDOM;
        } else {
            txadd = BLE_ADDR_TYPE_PUBLIC;
        }

        /* Check for whitelist match */
        if (!ble_ll_whitelist_match(rxbuf + BLE_LL_PDU_HDR_LEN, txadd)) {
            return -1;
        }
        ble_hdr->rxinfo.flags |= BLE_MBUF_HDR_F_DEVMATCH;
    }

    /* Setup to transmit the scan response if appropriate */
    rc = -1;
    if (pdu_type == BLE_ADV_PDU_TYPE_SCAN_REQ) {
        ble_phy_set_txend_cb(ble_ll_adv_tx_done, &g_ble_ll_adv_sm);
        rc = ble_phy_tx(advsm->scan_rsp_pdu, BLE_PHY_TRANSITION_NONE);
        if (!rc) {
            ble_hdr->rxinfo.flags |= BLE_MBUF_HDR_F_SCAN_RSP_TXD;
            STATS_INC(ble_ll_stats, scan_rsp_txg);
        }
    }

    return rc;
}

/**
 * Called when a connect request has been received.
 *
 * Context: Link Layer
 *
 * @param rxbuf
 * @param flags
 *
 * @return 0: no connection started. 1: connection started
 */
int
ble_ll_adv_conn_req_rxd(uint8_t *rxbuf, struct ble_mbuf_hdr *hdr)
{
    int valid;
    uint8_t pyld_len;
    uint8_t *inita;
    uint32_t endtime;
    struct ble_ll_adv_sm *advsm;

    /* Check filter policy. */
    valid = 0;
    advsm = &g_ble_ll_adv_sm;
    if (advsm->adv_filter_policy & 2) {
        if (hdr->rxinfo.flags & BLE_MBUF_HDR_F_DEVMATCH) {
            /* valid connection request received */
            valid = 1;
        }
    } else {
        /* If this is for us? */
        if (!ble_ll_adv_addr_cmp(rxbuf)) {
            /*
             * Only accept connect requests from the desired address if we
             * are doing directed advertising
             */
            if ((advsm->adv_type == BLE_HCI_ADV_TYPE_ADV_DIRECT_IND_HD) ||
                (advsm->adv_type == BLE_HCI_ADV_TYPE_ADV_DIRECT_IND_LD)) {
                /* XXX: not sure if this works if address is random */
                /* Compare addresses */
                inita = rxbuf + BLE_LL_PDU_HDR_LEN;
                if (!memcmp(advsm->initiator_addr, inita, BLE_DEV_ADDR_LEN)) {
                    valid = 1;
                }
            } else {
                valid = 1;
            }
        }
    }

    if (valid) {
        /* Try to start slave connection. If successful, stop advertising */
        pyld_len = rxbuf[1] & BLE_ADV_PDU_HDR_LEN_MASK;
        endtime = hdr->beg_cputime + BLE_TX_DUR_USECS_M(pyld_len);
        valid = ble_ll_conn_slave_start(rxbuf, endtime);
        if (valid) {
            ble_ll_adv_sm_stop(advsm);
        }
    }

    return valid;
}

/**
 * Called on phy rx pdu end when in advertising state.
 *
 * There are only two pdu types we care about in this state: scan requests
 * and connection requests. When we receive a scan request we must determine if
 * we need to send a scan response and that needs to be acted on within T_IFS.
 *
 * When we receive a connection request, we need to determine if we will allow
 * this device to start a connection with us. However, no immediate response is
 * sent so we handle this at the link layer task.
 *
 * Context: Interrupt
 *
 * @param pdu_type Type of pdu received.
 * @param rxpdu Pointer to received PDU
 *
 * @return int
 *       < 0: Disable the phy after reception.
 *      == 0: Do not disable the PHY
 *       > 0: Do not disable PHY as that has already been done.
 */
int
ble_ll_adv_rx_isr_end(uint8_t pdu_type, struct os_mbuf *rxpdu, int crcok)
{
    int rc;

    rc = -1;
    if (rxpdu == NULL) {
        ble_ll_adv_tx_done(&g_ble_ll_adv_sm);
    } else {
        if (crcok) {
            if ((pdu_type == BLE_ADV_PDU_TYPE_SCAN_REQ) ||
                (pdu_type == BLE_ADV_PDU_TYPE_CONNECT_REQ)) {
                /* Process request */
                rc = ble_ll_adv_rx_req(pdu_type, rxpdu);
            }
        }
    }

    if (rc) {
        ble_ll_state_set(BLE_LL_STATE_STANDBY);
    }

    return rc;
}

/**
 * Process a received packet at the link layer task when in the advertising
 * state
 *
 * Context: Link Layer
 *
 *
 * @param ptype
 * @param rxbuf
 * @param hdr
 *
 * @return int
 */
void
ble_ll_adv_rx_pkt_in(uint8_t ptype, uint8_t *rxbuf, struct ble_mbuf_hdr *hdr)
{
    int adv_event_over;

    /*
     * If we have received a scan request and we are transmitting a response
     * or we have received a valid connect request, dont "end" the advertising
     * event. In the case of a connect request we will stop advertising. In
     * the case of the scan response transmission we will get a transmit
     * end callback.
     */
    adv_event_over = 1;
    if (BLE_MBUF_HDR_CRC_OK(hdr)) {
        if (ptype == BLE_ADV_PDU_TYPE_CONNECT_REQ) {
            if (ble_ll_adv_conn_req_rxd(rxbuf, hdr)) {
                adv_event_over = 0;
            }
        } else {
            if ((ptype == BLE_ADV_PDU_TYPE_SCAN_REQ) &&
                (hdr->rxinfo.flags & BLE_MBUF_HDR_F_SCAN_RSP_TXD)) {
                adv_event_over = 0;
            }
        }
    }

    if (adv_event_over) {
        ble_ll_adv_event_done(&g_ble_ll_adv_sm);
    }
}

/**
 * Called when a receive PDU has started and we are advertising.
 *
 * Context: interrupt
 *
 * @param pdu_type
 * @param rxpdu
 *
 * @return int
 *   < 0: A frame we dont want to receive.
 *   = 0: Continue to receive frame. Dont go from rx to tx
 *   > 0: Continue to receive frame and go from rx to tx when done
 */
int
ble_ll_adv_rx_isr_start(uint8_t pdu_type)
{
    int rc;
    struct ble_ll_adv_sm *advsm;

    /* Assume we will abort the frame */
    rc = -1;

    /* If we get a scan request we must tell the phy to go from rx to tx */
    advsm = &g_ble_ll_adv_sm;
    if (pdu_type == BLE_ADV_PDU_TYPE_SCAN_REQ) {
        /* Only accept scan requests if we are indirect adv or scan adv */
        if ((advsm->adv_type == BLE_HCI_ADV_TYPE_ADV_SCAN_IND) ||
            (advsm->adv_type == BLE_HCI_ADV_TYPE_ADV_IND)) {
            rc = 1;
        }
    } else {
        /* Only accept connect requests if connectable advertising event */
        if (pdu_type == BLE_ADV_PDU_TYPE_CONNECT_REQ) {
            if ((advsm->adv_type == BLE_HCI_ADV_TYPE_ADV_DIRECT_IND_HD) ||
                (advsm->adv_type == BLE_HCI_ADV_TYPE_ADV_DIRECT_IND_LD) ||
                (advsm->adv_type == BLE_HCI_ADV_TYPE_ADV_IND)) {
                rc = 0;
            }
        }
    }

    /*
     * If we abort the frame, we need to post the LL task to check if the
     * advertising event is over.
     */
    if (rc < 0) {
        ble_ll_adv_tx_done(advsm);
    }

    return rc;
}

/**
 * Called when an advertising event is over.
 *
 * Context: Link Layer task.
 *
 * @param arg Pointer to advertising state machine.
 */
void
ble_ll_adv_event_done(void *arg)
{
    uint8_t mask;
    uint8_t final_adv_chan;
    int32_t delta_t;
    uint32_t itvl;
    uint32_t start_time;
    struct ble_ll_adv_sm *advsm;

    /* Stop advertising event */
    advsm = (struct ble_ll_adv_sm *)arg;
    assert(advsm->enabled);

    /* Remove the element from the schedule if it is still there. */
    ble_ll_sched_rmv_elem(&advsm->adv_sch);
    os_eventq_remove(&g_ble_ll_data.ll_evq, &advsm->adv_txdone_ev);

    /*
     * Check if we have ended our advertising event. If our last advertising
     * packet was sent on the last channel, it means we are done with this
     * event.
     */
    if (advsm->adv_chanmask & 0x04) {
        final_adv_chan = BLE_PHY_ADV_CHAN_START + 2;
    } else if (advsm->adv_chanmask & 0x02) {
        final_adv_chan = BLE_PHY_ADV_CHAN_START + 1;
    } else {
        final_adv_chan = BLE_PHY_ADV_CHAN_START;
    }

    if (advsm->adv_chan == final_adv_chan) {
        /* Check if we need to resume scanning */
        ble_ll_scan_chk_resume();

        /* This event is over. Set adv channel to first one */
        advsm->adv_chan = ble_ll_adv_first_chan(advsm);

        /* Calculate start time of next advertising event */
        itvl = advsm->adv_itvl_usecs;
        if (advsm->adv_type != BLE_HCI_ADV_TYPE_ADV_DIRECT_IND_HD) {
            itvl += rand() % (BLE_LL_ADV_DELAY_MS_MAX * 1000);
        }
        advsm->adv_event_start_time += cputime_usecs_to_ticks(itvl);
        advsm->adv_pdu_start_time = advsm->adv_event_start_time;

        /*
         * The scheduled time better be in the future! If it is not, we will
         * just keep advancing until we the time is in the future
         */
        start_time = advsm->adv_pdu_start_time -
            cputime_usecs_to_ticks(XCVR_TX_SCHED_DELAY_USECS);

        delta_t = (int32_t)(start_time - cputime_get32());
        if (delta_t < 0) {
            /* Calculate start time of next advertising event */
            while (delta_t < 0) {
                itvl = advsm->adv_itvl_usecs;
                if (advsm->adv_type != BLE_HCI_ADV_TYPE_ADV_DIRECT_IND_HD) {
                    itvl += rand() % (BLE_LL_ADV_DELAY_MS_MAX * 1000);
                }
                itvl = cputime_usecs_to_ticks(itvl);
                advsm->adv_event_start_time += itvl;
                advsm->adv_pdu_start_time = advsm->adv_event_start_time;
                delta_t += (int32_t)itvl;
            }
        }
    } else {
        /*
         * Move to next advertising channel. If not in the mask, just
         * increment by 1. We can do this because we already checked if we
         * just transmitted on the last advertising channel
         */
        ++advsm->adv_chan;
        mask = 1 << (advsm->adv_chan - BLE_PHY_ADV_CHAN_START);
        if ((mask & advsm->adv_chanmask) == 0) {
            ++advsm->adv_chan;
        }

        /*
         * We will transmit right away. Set next pdu start time to now
         * plus a xcvr start delay just so we dont count late adv starts
         */
        advsm->adv_pdu_start_time = cputime_get32() +
            cputime_usecs_to_ticks(XCVR_TX_SCHED_DELAY_USECS);
    }

    /*
     * Stop high duty cycle directed advertising if we have been doing
     * it for more than 1.28 seconds
     */
    if (advsm->adv_type == BLE_HCI_ADV_TYPE_ADV_DIRECT_IND_HD) {
        if (advsm->adv_pdu_start_time >= advsm->adv_dir_hd_end_time) {
            /* Disable advertising */
            advsm->enabled = 0;
            ble_ll_conn_comp_event_send(NULL, BLE_ERR_DIR_ADV_TMO);
            ble_ll_scan_chk_resume();
            return;
        }
    }

    /* Schedule advertising transmit */
    ble_ll_adv_set_sched(advsm, 0);

    /*
     * In the unlikely event we cant reschedule this, just post a done
     * event and we will reschedule the next advertising event
     */
    if (ble_ll_sched_adv_reschedule(&advsm->adv_sch)) {
        os_eventq_put(&g_ble_ll_data.ll_evq, &advsm->adv_txdone_ev);
    }
}

/**
 * Checks if the controller can change the whitelist. If advertising is enabled
 * and is using the whitelist the controller is not allowed to change the
 * whitelist.
 *
 * @return int 0: not allowed to change whitelist; 1: change allowed.
 */
int
ble_ll_adv_can_chg_whitelist(void)
{
    int rc;
    struct ble_ll_adv_sm *advsm;

    advsm = &g_ble_ll_adv_sm;
    if (advsm->enabled && (advsm->adv_filter_policy != BLE_HCI_ADV_FILT_NONE)) {
        rc = 0;
    } else {
        rc = 1;
    }

    return rc;
}

/**
 * Called when the LL wait for response timer expires while in the advertising
 * state. Disables the phy and
 *
 */
void
ble_ll_adv_wfr_timer_exp(void)
{
    ble_phy_disable();
    ble_ll_adv_tx_done(&g_ble_ll_adv_sm);
}

/**
 * Reset the advertising state machine.
 *
 * Context: Link Layer task
 *
 */
void
ble_ll_adv_reset(void)
{
    struct ble_ll_adv_sm *advsm;
    advsm = &g_ble_ll_adv_sm;

    /* Stop advertising state machine */
    ble_ll_adv_sm_stop(advsm);

    /* Free scan pdu's */
    os_mbuf_free_chain(advsm->scan_rsp_pdu);

    /* re-initialize the advertiser state machine */
    ble_ll_adv_init();
}

/* Called to determine if advertising is enabled */
uint8_t
ble_ll_adv_enabled(void)
{
    return g_ble_ll_adv_sm.enabled;
}

/**
 * Initialize the advertising functionality of a BLE device. This should
 * be called once on initialization
 */
void
ble_ll_adv_init(void)
{
    struct ble_ll_adv_sm *advsm;

    /* Set default advertising parameters */
    advsm = &g_ble_ll_adv_sm;
    memset(advsm, 0, sizeof(struct ble_ll_adv_sm));

    advsm->adv_itvl_min = BLE_HCI_ADV_ITVL_DEF;
    advsm->adv_itvl_max = BLE_HCI_ADV_ITVL_DEF;
    advsm->adv_chanmask = BLE_HCI_ADV_CHANMASK_DEF;

    /* Initialize advertising tx done event */
    advsm->adv_txdone_ev.ev_type = BLE_LL_EVENT_ADV_EV_DONE;
    advsm->adv_txdone_ev.ev_arg = advsm;

    /* Get a scan response mbuf (packet header) and attach to state machine */
    advsm->scan_rsp_pdu = os_msys_get_pkthdr(BLE_MBUF_PAYLOAD_SIZE,
                                             sizeof(struct ble_mbuf_hdr));
    assert(advsm->scan_rsp_pdu != NULL);
}

