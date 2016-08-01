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
#include "controller/ble_phy.h"
#include "controller/ble_ll.h"
#include "controller/ble_ll_sched.h"
#include "controller/ble_ll_adv.h"
#include "controller/ble_ll_scan.h"
#include "controller/ble_ll_hci.h"
#include "controller/ble_ll_whitelist.h"
#include "hal/hal_cputime.h"
#include "hal/hal_gpio.h"

/*
 * XXX:
 * 1) I think I can guarantee that we dont process things out of order if
 * I send an event when a scan request is sent. The scan_rsp_pending flag
 * code might be made simpler.
 *
 * 2) Interleave sending scan requests to different advertisers? I guess I need
 * a list of advertisers to which I sent a scan request and have yet to
 * receive a scan response from? Implement this.
 */

/* Dont allow more than 255 of these entries */
#if NIMBLE_OPT_LL_NUM_SCAN_DUP_ADVS > 255
    #error "Cannot have more than 255 duplicate entries!"
#endif
#if NIMBLE_OPT_LL_NUM_SCAN_RSP_ADVS > 255
    #error "Cannot have more than 255 scan response entries!"
#endif

/* The scanning state machine global object */
struct ble_ll_scan_sm g_ble_ll_scan_sm;

/*
 * Structure used to store advertisers. This is used to limit sending scan
 * requests to the same advertiser and also to filter duplicate events sent
 * to the host.
 */
struct ble_ll_scan_advertisers
{
    uint16_t            sc_adv_flags;
    struct ble_dev_addr adv_addr;
};

#define BLE_LL_SC_ADV_F_RANDOM_ADDR     (0x01)
#define BLE_LL_SC_ADV_F_SCAN_RSP_RXD    (0x02)
#define BLE_LL_SC_ADV_F_DIRECT_RPT_SENT (0x04)
#define BLE_LL_SC_ADV_F_ADV_RPT_SENT    (0x08)

/* Contains list of advertisers that we have heard scan responses from */
static uint8_t g_ble_ll_scan_num_rsp_advs;
struct ble_ll_scan_advertisers
g_ble_ll_scan_rsp_advs[NIMBLE_OPT_LL_NUM_SCAN_RSP_ADVS];

/* Used to filter duplicate advertising events to host */
static uint8_t g_ble_ll_scan_num_dup_advs;
struct ble_ll_scan_advertisers
g_ble_ll_scan_dup_advs[NIMBLE_OPT_LL_NUM_SCAN_DUP_ADVS];

/* See Vol 6 Part B Section 4.4.3.2. Active scanning backoff */
static void
ble_ll_scan_req_backoff(struct ble_ll_scan_sm *scansm, int success)
{
    scansm->scan_rsp_pending = 0;
    if (success) {
        scansm->scan_rsp_cons_fails = 0;
        ++scansm->scan_rsp_cons_ok;
        if (scansm->scan_rsp_cons_ok == 2) {
            scansm->scan_rsp_cons_ok = 0;
            if (scansm->upper_limit > 1) {
                scansm->upper_limit >>= 1;
            }
        }
        STATS_INC(ble_ll_stats, scan_req_txg);
    } else {
        scansm->scan_rsp_cons_ok = 0;
        ++scansm->scan_rsp_cons_fails;
        if (scansm->scan_rsp_cons_fails == 2) {
            scansm->scan_rsp_cons_fails = 0;
            if (scansm->upper_limit < 256) {
                scansm->upper_limit <<= 1;
            }
        }
        STATS_INC(ble_ll_stats, scan_req_txf);
    }

    scansm->backoff_count = rand() & (scansm->upper_limit - 1);
    ++scansm->backoff_count;
    assert(scansm->backoff_count <= 256);
}

/**
 * ble ll scan req pdu make
 *
 * Construct a SCAN_REQ PDU.
 *
 * @param scansm Pointer to scanning state machine
 * @param adv_addr Pointer to device address of advertiser
 * @param addr_type 0 if public; non-zero if random
 */
static void
ble_ll_scan_req_pdu_make(struct ble_ll_scan_sm *scansm, uint8_t *adv_addr,
                         uint8_t adv_addr_type)
{
    uint8_t     *dptr;
    uint8_t     pdu_type;
    uint8_t     *addr;
    struct os_mbuf *m;

    /* Construct first PDU header byte */
    pdu_type = BLE_ADV_PDU_TYPE_SCAN_REQ;
    if (adv_addr_type) {
        pdu_type |= BLE_ADV_PDU_HDR_RXADD_RAND;
    }

    /* Get the advertising PDU */
    m = scansm->scan_req_pdu;
    assert(m != NULL);

    /* Get pointer to our device address */
    if (scansm->own_addr_type == BLE_HCI_ADV_OWN_ADDR_PUBLIC) {
        addr = g_dev_addr;
    } else if (scansm->own_addr_type == BLE_HCI_ADV_OWN_ADDR_RANDOM) {
        pdu_type |= BLE_ADV_PDU_HDR_TXADD_RAND;
        addr = g_random_addr;
    } else {
        /* XXX: unsupported for now  */
        addr = NULL;
        assert(0);
    }

    ble_ll_mbuf_init(m, BLE_SCAN_REQ_LEN, pdu_type);

    /* Construct the scan request */
    dptr = m->om_data;
    memcpy(dptr, addr, BLE_DEV_ADDR_LEN);
    memcpy(dptr + BLE_DEV_ADDR_LEN, adv_addr, BLE_DEV_ADDR_LEN);
}

/**
 * Checks to see if an advertiser is on the duplicate address list.
 *
 * @param addr Pointer to address
 * @param txadd TxAdd bit. 0: public; random otherwise
 *
 * @return uint8_t 0: not on list; any other value is
 */
static struct ble_ll_scan_advertisers *
ble_ll_scan_find_dup_adv(uint8_t *addr, uint8_t txadd)
{
    uint8_t num_advs;
    struct ble_ll_scan_advertisers *adv;

    /* Do we have an address match? Must match address type */
    adv = &g_ble_ll_scan_dup_advs[0];
    num_advs = g_ble_ll_scan_num_dup_advs;
    while (num_advs) {
        if (!memcmp(&adv->adv_addr, addr, BLE_DEV_ADDR_LEN)) {
            /* Address type must match */
            if (txadd) {
                if ((adv->sc_adv_flags & BLE_LL_SC_ADV_F_RANDOM_ADDR) == 0) {
                    continue;
                }
            } else {
                if (adv->sc_adv_flags & BLE_LL_SC_ADV_F_RANDOM_ADDR) {
                    continue;
                }
            }

            return adv;
        }
        ++adv;
        --num_advs;
    }

    return NULL;
}

/**
 * Check if a packet is a duplicate advertising packet.
 *
 * @param pdu_type
 * @param rxbuf
 *
 * @return int 0: not a duplicate. 1:duplicate
 */
int
ble_ll_scan_is_dup_adv(uint8_t pdu_type, uint8_t txadd, uint8_t *addr)
{
    struct ble_ll_scan_advertisers *adv;

    adv = ble_ll_scan_find_dup_adv(addr, txadd);
    if (adv) {
        /* Check appropriate flag (based on type of PDU) */
        if (pdu_type == BLE_ADV_PDU_TYPE_ADV_DIRECT_IND) {
            if (adv->sc_adv_flags & BLE_LL_SC_ADV_F_DIRECT_RPT_SENT) {
                return 1;
            }
        } else {
            if (adv->sc_adv_flags & BLE_LL_SC_ADV_F_ADV_RPT_SENT) {
                return 1;
            }
        }
    }

    return 0;
}

/**
 * Add an advertiser the list of duplicate advertisers. An address gets added to
 * the list of duplicate addresses when the controller sends an advertising
 * report to the host.
 *
 * @param addr
 * @param Txadd. TxAdd bit (0 public, random otherwise)
 */
void
ble_ll_scan_add_dup_adv(uint8_t *addr, uint8_t txadd)
{
    uint8_t num_advs;
    struct ble_ll_scan_advertisers *adv;

    /* Check to see if on list. */
    adv = ble_ll_scan_find_dup_adv(addr, txadd);
    if (!adv) {
        /* XXX: for now, if we dont have room, just leave */
        num_advs = g_ble_ll_scan_num_dup_advs;
        if (num_advs == NIMBLE_OPT_LL_NUM_SCAN_DUP_ADVS) {
            return;
        }

        /* Add the advertiser to the array */
        adv = &g_ble_ll_scan_dup_advs[num_advs];
        memcpy(&adv->adv_addr, addr, BLE_DEV_ADDR_LEN);
        ++g_ble_ll_scan_num_dup_advs;

        adv->sc_adv_flags = 0;
        if (txadd) {
            adv->sc_adv_flags |= BLE_LL_SC_ADV_F_RANDOM_ADDR;
        }
    }

    /*
     * XXX: need to set correct flag based on type of report being sent
     * for now, we dont send direct advertising reports
     */
    adv->sc_adv_flags |= BLE_LL_SC_ADV_F_ADV_RPT_SENT;
}

/**
 * Checks to see if we have received a scan response from this advertiser.
 *
 * @param adv_addr Address of advertiser
 * @param txadd TxAdd bit (0: public; random otherwise)
 *
 * @return int 0: have not received a scan response; 1 otherwise.
 */
static int
ble_ll_scan_have_rxd_scan_rsp(uint8_t *addr, uint8_t txadd)
{
    uint8_t num_advs;
    struct ble_ll_scan_advertisers *adv;

    /* Do we have an address match? Must match address type */
    adv = &g_ble_ll_scan_rsp_advs[0];
    num_advs = g_ble_ll_scan_num_rsp_advs;
    while (num_advs) {
        if (!memcmp(&adv->adv_addr, addr, BLE_DEV_ADDR_LEN)) {
            /* Address type must match */
            if (txadd) {
                if (adv->sc_adv_flags & BLE_LL_SC_ADV_F_RANDOM_ADDR) {
                    return 1;
                }
            } else {
                if ((adv->sc_adv_flags & BLE_LL_SC_ADV_F_RANDOM_ADDR) == 0) {
                    return 1;
                }
            }
        }
        ++adv;
        --num_advs;
    }

    return 0;
}

static void
ble_ll_scan_add_scan_rsp_adv(uint8_t *addr, uint8_t txadd)
{
    uint8_t num_advs;
    struct ble_ll_scan_advertisers *adv;

    /* XXX: for now, if we dont have room, just leave */
    num_advs = g_ble_ll_scan_num_rsp_advs;
    if (num_advs == NIMBLE_OPT_LL_NUM_SCAN_RSP_ADVS) {
        return;
    }

    /* Check if address is already on the list */
    if (ble_ll_scan_have_rxd_scan_rsp(addr, txadd)) {
        return;
    }

    /* Add the advertiser to the array */
    adv = &g_ble_ll_scan_rsp_advs[num_advs];
    memcpy(&adv->adv_addr, addr, BLE_DEV_ADDR_LEN);
    adv->sc_adv_flags = BLE_LL_SC_ADV_F_SCAN_RSP_RXD;
    if (txadd) {
        adv->sc_adv_flags |= BLE_LL_SC_ADV_F_RANDOM_ADDR;
    }
    ++g_ble_ll_scan_num_rsp_advs;

    return;
}

/**
 * Send an advertising report to the host.
 *
 * NOTE: while we are allowed to send multiple devices in one report, we
 * will just send for one for now.
 *
 * @param pdu_type
 * @param txadd
 * @param rxbuf
 * @param rssi
 */
static void
ble_ll_hci_send_adv_report(uint8_t pdu_type, uint8_t txadd, uint8_t *rxbuf,
                           int8_t rssi)
{
    int rc;
    uint8_t evtype;
    uint8_t subev;
    uint8_t *evbuf;
    uint8_t adv_data_len;

    subev = BLE_HCI_LE_SUBEV_ADV_RPT;
    if (pdu_type == BLE_ADV_PDU_TYPE_ADV_DIRECT_IND) {
        /* XXX: NOTE: the direct advertising report is only used when InitA
           is a resolvable private address. We dont support that yet! */
        //subev = BLE_HCI_LE_SUBEV_DIRECT_ADV_RPT;
        evtype = BLE_HCI_ADV_RPT_EVTYPE_DIR_IND;
        adv_data_len = 0;
    } else {
        if (pdu_type == BLE_ADV_PDU_TYPE_ADV_IND) {
            evtype = BLE_HCI_ADV_RPT_EVTYPE_ADV_IND;
        } else if (pdu_type == BLE_ADV_PDU_TYPE_ADV_SCAN_IND) {
            evtype = BLE_HCI_ADV_RPT_EVTYPE_SCAN_IND;
        } else if (pdu_type == BLE_ADV_PDU_TYPE_ADV_NONCONN_IND) {
            evtype = BLE_HCI_ADV_RPT_EVTYPE_NONCONN_IND;
        } else {
            evtype = BLE_HCI_ADV_RPT_EVTYPE_SCAN_RSP;
        }
        subev = BLE_HCI_LE_SUBEV_ADV_RPT;

        adv_data_len = rxbuf[1] & BLE_ADV_PDU_HDR_LEN_MASK;
        adv_data_len -= BLE_DEV_ADDR_LEN;
    }

    if (ble_ll_hci_is_le_event_enabled(subev)) {
        evbuf = os_memblock_get(&g_hci_cmd_pool);
        if (evbuf) {
            evbuf[0] = BLE_HCI_EVCODE_LE_META;
            evbuf[1] = 12 + adv_data_len;
            evbuf[2] = subev;
            evbuf[3] = 1;       /* number of reports */
            evbuf[4] = evtype;

            /* XXX: need to deal with resolvable addresses here! */
            if (txadd) {
                evbuf[5] = BLE_HCI_ADV_OWN_ADDR_RANDOM;
            } else {
                evbuf[5] = BLE_HCI_ADV_OWN_ADDR_PUBLIC;
            }
            rxbuf += BLE_LL_PDU_HDR_LEN;
            memcpy(evbuf + 6, rxbuf, BLE_DEV_ADDR_LEN);
            evbuf[12] = adv_data_len;
            memcpy(evbuf + 13, rxbuf + BLE_DEV_ADDR_LEN,
                   adv_data_len);
            evbuf[13 + adv_data_len] = rssi;

            rc = ble_ll_hci_event_send(evbuf);
            if (!rc) {
                /* If filtering, add it to list of duplicate addresses */
                if (g_ble_ll_scan_sm.scan_filt_dups) {
                    ble_ll_scan_add_dup_adv(rxbuf, txadd);
                }
            }
        }
    }
}

/**
 * Checks the scanner filter policy to determine if we should allow or discard
 * the received PDU.
 *
 * NOTE: connect requests and scan requests are not passed here
 *
 * @param pdu_type
 * @param rxbuf
 *
 * @return int 0: pdu allowed by filter policy. 1: pdu not allowed
 */
int
ble_ll_scan_chk_filter_policy(uint8_t pdu_type, uint8_t *rxbuf, uint8_t flags)
{
    uint8_t *addr;
    uint8_t addr_type;
    int use_whitelist;
    int chk_inita;

    use_whitelist = 0;
    chk_inita = 0;

    switch (g_ble_ll_scan_sm.scan_filt_policy) {
    case BLE_HCI_SCAN_FILT_NO_WL:
        break;
    case BLE_HCI_SCAN_FILT_USE_WL:
        use_whitelist = 1;
        break;
    case BLE_HCI_SCAN_FILT_NO_WL_INITA:
        chk_inita = 1;
        break;
    case BLE_HCI_SCAN_FILT_USE_WL_INITA:
        chk_inita = 1;
        use_whitelist = 1;
        break;
    default:
        assert(0);
        break;
    }

    /* If we are using the whitelist, check that first */
    if (use_whitelist && (pdu_type != BLE_ADV_PDU_TYPE_SCAN_RSP)) {
        /* If there was a devmatch, we will allow the PDU */
        if (flags & BLE_MBUF_HDR_F_DEVMATCH) {
            return 0;
        } else {
            return 1;
        }
    }

    /* If this is a directed advertisement, check that it is for us */
    if (pdu_type == BLE_ADV_PDU_TYPE_ADV_DIRECT_IND) {
        /* Is this for us? If not, is it resolvable */
        addr = rxbuf + BLE_LL_PDU_HDR_LEN;
        addr_type = rxbuf[0] & BLE_ADV_PDU_HDR_RXADD_MASK;
        if (!ble_ll_is_our_devaddr(addr + BLE_DEV_ADDR_LEN, addr_type)) {
            if (!chk_inita || !ble_ll_is_resolvable_priv_addr(addr)) {
                return 1;
            }
        }
    }

    return 0;
}

/**
 * Called to enable the receiver for scanning.
 *
 * Context: Link Layer task
 *
 * @param sch
 *
 * @return int
 */
static void
ble_ll_scan_start(struct ble_ll_scan_sm *scansm, uint8_t chan)
{
    int rc;

    /* Set channel */
    rc = ble_phy_setchan(chan, 0, 0);
    assert(rc == 0);

    /*
     * Set transmit end callback to NULL in case we transmit a scan request.
     * There is a callback for the connect request.
     */
    ble_phy_set_txend_cb(NULL, NULL);

#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
    ble_phy_encrypt_disable();
#endif

    /* Start receiving */
    rc = ble_phy_rx();
    if (!rc) {
        /* Enable/disable whitelisting */
        if (scansm->scan_filt_policy & 1) {
            ble_ll_whitelist_enable();
        } else {
            ble_ll_whitelist_disable();
        }

        /* Set link layer state to scanning */
        if (scansm->scan_type == BLE_SCAN_TYPE_INITIATE) {
            ble_ll_state_set(BLE_LL_STATE_INITIATING);
        } else {
            ble_ll_state_set(BLE_LL_STATE_SCANNING);
        }
    }

    /* If there is a still a scan response pending, we have failed! */
    if (scansm->scan_rsp_pending) {
        ble_ll_scan_req_backoff(scansm, 0);
    }
}

/**
 * Called to determine if we are inside or outside the scan window. If we
 * are inside the scan window it means that the device should be receiving
 * on the scan channel.
 *
 * Context: Link Layer
 *
 * @param scansm
 *
 * @return int 0: inside scan window 1: outside scan window
 */
static int
ble_ll_scan_window_chk(struct ble_ll_scan_sm *scansm, uint32_t cputime)
{
    int rc;
    uint8_t chan;
    uint32_t itvl;
    uint32_t win_start;

    itvl = cputime_usecs_to_ticks(scansm->scan_itvl * BLE_HCI_SCAN_ITVL);
    chan = scansm->scan_chan;
    win_start = scansm->scan_win_start_time;
#pragma GCC diagnostic ignored "-Wsign-compare"
    while ((int32_t)(cputime - win_start) >= itvl) {
        win_start += itvl;
        ++chan;
        if (chan == BLE_PHY_NUM_CHANS) {
            chan = BLE_PHY_ADV_CHAN_START;
        }
    }

    rc = 0;
    if (scansm->scan_window != scansm->scan_itvl) {
        itvl = cputime_usecs_to_ticks(scansm->scan_window * BLE_HCI_SCAN_ITVL);
        if ((cputime - win_start) >= itvl) {
            rc = 1;
        }
    }

    if (!rc) {
        /* Turn on the receiver and set state */
        ble_ll_scan_start(scansm, chan);
    }

    return rc;
}

/**
 * Stop the scanning state machine
 */
void
ble_ll_scan_sm_stop(int chk_disable)
{
    os_sr_t sr;
    uint8_t lls;
    struct ble_ll_scan_sm *scansm;

    /* Stop the scanning timer  */
    scansm = &g_ble_ll_scan_sm;
    cputime_timer_stop(&scansm->scan_timer);

    /* Disable scanning state machine */
    scansm->scan_enabled = 0;

    /* Count # of times stopped */
    STATS_INC(ble_ll_stats, scan_stops);

    /* Only set state if we are currently in a scan window */
    if (chk_disable) {
        OS_ENTER_CRITICAL(sr);
        lls = ble_ll_state_get();
        if ((lls == BLE_LL_STATE_SCANNING) || (lls == BLE_LL_STATE_INITIATING)) {
            /* Disable phy */
            ble_phy_disable();

            /* Set LL state to standby */
            ble_ll_state_set(BLE_LL_STATE_STANDBY);
        }
        OS_EXIT_CRITICAL(sr);
    }
}

static int
ble_ll_scan_sm_start(struct ble_ll_scan_sm *scansm)
{
    /*
     * This is not in the specification. I will reject the command with a
     * command disallowed error if no random address has been sent by the
     * host. All the parameter errors refer to the command parameter
     * (which in this case is just enable or disable) so that is why I chose
     * command disallowed.
     */
    if (scansm->own_addr_type == BLE_HCI_ADV_OWN_ADDR_RANDOM) {
        if (!ble_ll_is_valid_random_addr(g_random_addr)) {
            return BLE_ERR_CMD_DISALLOWED;
        }
    }

    /* Count # of times started */
    STATS_INC(ble_ll_stats, scan_starts);

    /* Set flag telling us that scanning is enabled */
    scansm->scan_enabled = 1;

    /* Set first advertising channel */
    scansm->scan_chan = BLE_PHY_ADV_CHAN_START;

    /* Reset scan request backoff parameters to default */
    scansm->upper_limit = 1;
    scansm->backoff_count = 1;
    scansm->scan_rsp_pending = 0;

    /* Forget filtered advertisers from previous scan. */
    g_ble_ll_scan_num_rsp_advs = 0;
    g_ble_ll_scan_num_dup_advs = 0;

    /* XXX: align to current or next slot???. */
    /* Schedule start time now */
    scansm->scan_win_start_time = cputime_get32();

    /* Post scanning event to start off the scanning process */
    ble_ll_event_send(&scansm->scan_sched_ev);

    return BLE_ERR_SUCCESS;
}

/**
 * Called to process the scanning OS event which was posted to the LL task
 *
 * Context: Link Layer task.
 *
 * @param arg
 */
void
ble_ll_scan_event_proc(void *arg)
{
    os_sr_t sr;
    int rxstate;
    int start_scan;
    uint8_t chan;
    uint32_t now;
    uint32_t dt;
    uint32_t win;
    uint32_t win_start;
    uint32_t scan_itvl;
    uint32_t next_event_time;
    struct ble_ll_scan_sm *scansm;

    /*
     * Get the scanning state machine. If not enabled (this is possible), just
     * leave and do nothing (just make sure timer is stopped).
     */
    scansm = (struct ble_ll_scan_sm *)arg;
    if (!scansm->scan_enabled) {
        cputime_timer_stop(&scansm->scan_timer);
        return;
    }

    /* Make sure the scan window start time and channel are up to date. */
    now = cputime_get32();

    scan_itvl = cputime_usecs_to_ticks(scansm->scan_itvl * BLE_HCI_SCAN_ITVL);
    chan = scansm->scan_chan;
    win_start = scansm->scan_win_start_time;
#pragma GCC diagnostic ignored "-Wsign-compare"
    while ((int32_t)(now - win_start) >= scan_itvl) {
        win_start += scan_itvl;
        ++chan;
        if (chan == BLE_PHY_NUM_CHANS) {
            chan = BLE_PHY_ADV_CHAN_START;
        }
    }

    dt = now - win_start;
    scansm->scan_chan = chan;
    scansm->scan_win_start_time = win_start;

    /* Determine on/off state based on scan window */
    rxstate = 1;
    next_event_time = win_start + scan_itvl;
    if (scansm->scan_window != scansm->scan_itvl) {
        win = cputime_usecs_to_ticks(scansm->scan_window * BLE_HCI_SCAN_ITVL);
        if (dt >= win) {
            rxstate = 0;
        } else {
            next_event_time = win_start + win;
        }
    }

    OS_ENTER_CRITICAL(sr);
    /*
     * If we are not in the standby state it means that the scheduled
     * scanning event was overlapped in the schedule. In this case all we do
     * is post the scan schedule end event.
     */
    start_scan = 1;
    switch (ble_ll_state_get()) {
    case BLE_LL_STATE_ADV:
    case BLE_LL_STATE_CONNECTION:
         start_scan = 0;
        break;
    case BLE_LL_STATE_INITIATING:
    case BLE_LL_STATE_SCANNING:
        /* Must disable PHY since we will move to a new channel */
        ble_phy_disable();
        break;
    case BLE_LL_STATE_STANDBY:
        break;
    default:
        assert(0);
        break;
    }
    if (start_scan && rxstate) {
        ble_ll_scan_start(scansm, scansm->scan_chan);
    }
    OS_EXIT_CRITICAL(sr);

    cputime_timer_start(&scansm->scan_timer, next_event_time);
}

/**
 * ble ll scan rx pdu start
 *
 * Called when a PDU reception has started and the Link Layer is in the
 * scanning state.
 *
 * Context: Interrupt
 *
 * @param rxpdu Pointer to where received data is being stored.
 *
 * @return int
 *  0: we will not attempt to reply to this frame
 *  1: we may send a response to this frame.
 */
int
ble_ll_scan_rx_isr_start(uint8_t pdu_type, struct os_mbuf *rxpdu)
{
    int rc;
    struct ble_ll_scan_sm *scansm;
    struct ble_mbuf_hdr *ble_hdr;

    rc = 0;
    scansm = &g_ble_ll_scan_sm;

    switch (scansm->scan_type) {
    case BLE_SCAN_TYPE_ACTIVE:
        /* If adv ind or scan ind, we may send scan request */
        if ((pdu_type == BLE_ADV_PDU_TYPE_ADV_IND) ||
            (pdu_type == BLE_ADV_PDU_TYPE_ADV_SCAN_IND)) {
            rc = 1;
        }

        /*
         * If this is the first PDU after we sent the scan response (as
         * denoted by the scan rsp pending flag), we set a bit in the ble
         * header so the link layer can check to see if the scan request
         * was successful. We do it this way to let the Link Layer do the
         * work for successful scan requests. If failed, we do the work here.
         */
        if (scansm->scan_rsp_pending) {
            if (pdu_type == BLE_ADV_PDU_TYPE_SCAN_RSP) {
                ble_hdr = BLE_MBUF_HDR_PTR(rxpdu);
                ble_hdr->rxinfo.flags |= BLE_MBUF_HDR_F_SCAN_RSP_CHK;
            } else {
                ble_ll_scan_req_backoff(scansm, 0);
            }
        }

        /* Disable wfr if running */
        ble_ll_wfr_disable();
        break;
    case BLE_SCAN_TYPE_PASSIVE:
    default:
        break;
    }

    return rc;
}

/**
 * Called when a receive PDU has ended.
 *
 * Context: Interrupt
 *
 * @param rxpdu
 *
 * @return int
 *       < 0: Disable the phy after reception.
 *      == 0: Success. Do not disable the PHY.
 *       > 0: Do not disable PHY as that has already been done.
 */
int
ble_ll_scan_rx_isr_end(struct os_mbuf *rxpdu, uint8_t crcok)
{
    int rc;
    int chk_send_req;
    int chk_whitelist;
    uint8_t pdu_type;
    uint8_t addr_type;
    uint8_t *adv_addr;
    uint8_t *rxbuf;
    struct ble_mbuf_hdr *ble_hdr;
    struct ble_ll_scan_sm *scansm;

    /* Get scanning state machine */
    scansm = &g_ble_ll_scan_sm;

    /*
     * The reason we do something different here (as opposed to failed CRC) is
     * that the received PDU will not be handed up in this case. So we have
     * to restart scanning and handle a failed scan request. Note that we
     * return 0 in this case because we dont want the phy disabled.
     */
    if (rxpdu == NULL) {
        if (scansm->scan_rsp_pending) {
            ble_ll_scan_req_backoff(scansm, 0);
        }
        ble_phy_rx();
        return 0;
    }

    /* Just leave if the CRC is not OK. */
    rc = -1;
    if (!crcok) {
        goto scan_rx_isr_exit;
    }

    /* Get pdu type, pointer to address and address "type"  */
    rxbuf = rxpdu->om_data;
    pdu_type = rxbuf[0] & BLE_ADV_PDU_HDR_TYPE_MASK;
    adv_addr = rxbuf + BLE_LL_PDU_HDR_LEN;
    if (rxbuf[0] & BLE_ADV_PDU_HDR_TXADD_MASK) {
        addr_type = BLE_ADDR_TYPE_RANDOM;
    } else {
        addr_type = BLE_ADDR_TYPE_PUBLIC;
    }

    /* Determine if request may be sent and if whitelist needs to be checked */
    chk_send_req = 0;
    switch (pdu_type) {
    case BLE_ADV_PDU_TYPE_ADV_IND:
    case BLE_ADV_PDU_TYPE_ADV_SCAN_IND:
        if (scansm->scan_type == BLE_SCAN_TYPE_ACTIVE) {
            chk_send_req = 1;
        }
        chk_whitelist = 1;
        break;
    case BLE_ADV_PDU_TYPE_ADV_NONCONN_IND:
    case BLE_ADV_PDU_TYPE_ADV_DIRECT_IND:
        chk_whitelist = 1;
        break;
    default:
        chk_whitelist = 0;
        break;
    }

    /* Set device match bit if we are whitelisting */
    if (chk_whitelist && (scansm->scan_filt_policy & 1)) {
        /* Check if device is on whitelist. If not, leave */
        if (!ble_ll_whitelist_match(adv_addr, addr_type)) {
            return -1;
        }
        ble_hdr = BLE_MBUF_HDR_PTR(rxpdu);
        ble_hdr->rxinfo.flags |= BLE_MBUF_HDR_F_DEVMATCH;
    }

    /* Should we send a scan request? */
    if (chk_send_req) {
        /*
         * Check to see if we have received a scan response from this
         * advertisor. If so, no need to send scan request.
         */
        if (ble_ll_scan_have_rxd_scan_rsp(adv_addr, addr_type)) {
            return -1;
        }

        /* Better not be a scan response pending */
        assert(scansm->scan_rsp_pending == 0);

        /* We want to send a request. See if backoff allows us */
        --scansm->backoff_count;
        if (scansm->backoff_count == 0) {
            /* Setup to transmit the scan request */
            ble_ll_scan_req_pdu_make(scansm, adv_addr, addr_type);
            rc = ble_phy_tx(scansm->scan_req_pdu, BLE_PHY_TRANSITION_TX_RX);

            /* Set "waiting for scan response" flag */
            scansm->scan_rsp_pending = 1;
        }
    }

scan_rx_isr_exit:
    if (rc) {
        ble_ll_state_set(BLE_LL_STATE_STANDBY);
    }
    return rc;
}

/**
 * Called to resume scanning. This is called after an advertising event or
 * connection event has ended. It is also called if we receive a packet while
 * in the initiating or scanning state.
 *
 * Context: Link Layer task
 */
void
ble_ll_scan_chk_resume(void)
{
    os_sr_t sr;
    struct ble_ll_scan_sm *scansm;

    scansm = &g_ble_ll_scan_sm;
    if (scansm->scan_enabled) {
        OS_ENTER_CRITICAL(sr);
        if (ble_ll_state_get() == BLE_LL_STATE_STANDBY) {
            ble_ll_scan_window_chk(scansm, cputime_get32());
        }
        OS_EXIT_CRITICAL(sr);
    }
}

/**
 * Connection supervision timer callback; means that the connection supervision
 * timeout has been reached and we should perform the appropriate actions.
 *
 * Context: Interrupt (cputimer)
 *
 * @param arg Pointer to connection state machine.
 */
void
ble_ll_scan_timer_cb(void *arg)
{
    struct ble_ll_scan_sm *scansm;

    scansm = (struct ble_ll_scan_sm *)arg;
    ble_ll_event_send(&scansm->scan_sched_ev);
}

/**
 * Called when the wait for response timer expires while in the scanning
 * state.
 *
 * Context: Interrupt.
 */
void
ble_ll_scan_wfr_timer_exp(void)
{
    struct ble_ll_scan_sm *scansm;

    ble_phy_disable();

    /*
     * If we timed out waiting for a response, the scan response pending
     * flag should be set. Deal with scan backoff. Put device back into rx.
     */
    scansm = &g_ble_ll_scan_sm;
    if (scansm->scan_rsp_pending) {
        ble_ll_scan_req_backoff(scansm, 0);
    }
    ble_phy_rx();
}

/**
 * Process a received PDU while in the scanning state.
 *
 * Context: Link Layer task.
 *
 * @param pdu_type
 * @param rxbuf
 */
void
ble_ll_scan_rx_pkt_in(uint8_t ptype, uint8_t *rxbuf, struct ble_mbuf_hdr *hdr)
{
    uint8_t *adv_addr;
    uint8_t *adva;
    uint8_t txadd;
    uint8_t rxadd;
    uint8_t scan_rsp_chk;
    struct ble_ll_scan_sm *scansm;
    struct ble_mbuf_hdr *ble_hdr;

    /* Set scan response check flag */
    scan_rsp_chk = hdr->rxinfo.flags & BLE_MBUF_HDR_F_SCAN_RSP_CHK;

    /* We dont care about scan requests or connect requests */
    if (!BLE_MBUF_HDR_CRC_OK(hdr) || (ptype == BLE_ADV_PDU_TYPE_SCAN_REQ) ||
        (ptype == BLE_ADV_PDU_TYPE_CONNECT_REQ)) {
        goto scan_continue;
    }

    /* Check the scanner filter policy */
    if (ble_ll_scan_chk_filter_policy(ptype, rxbuf, hdr->rxinfo.flags)) {
        goto scan_continue;
    }

    /* Get advertisers address type and a pointer to the address */
    txadd = rxbuf[0] & BLE_ADV_PDU_HDR_TXADD_MASK;
    adv_addr = rxbuf + BLE_LL_PDU_HDR_LEN;

    /*
     * XXX: The BLE spec is a bit unclear here. What if we get a scan
     * response from an advertiser that we did not send a request to?
     * Do we send an advertising report? Do we add it to list of devices
     * that we have heard a scan response from?
     */
    scansm = &g_ble_ll_scan_sm;
    if (ptype == BLE_ADV_PDU_TYPE_SCAN_RSP) {
        /*
         * If this is a scan response in reply to a request we sent we need
         * to store this advertiser's address so we dont send a request to it.
         */
        if (scansm->scan_rsp_pending && scan_rsp_chk) {
            /*
             * We could also check the timing of the scan reponse; make sure
             * that it is relatively close to the end of the scan request but
             * we wont for now.
             */
            ble_hdr = BLE_MBUF_HDR_PTR(scansm->scan_req_pdu);
            rxadd = ble_hdr->txinfo.hdr_byte & BLE_ADV_PDU_HDR_RXADD_MASK;
            adva = scansm->scan_req_pdu->om_data + BLE_DEV_ADDR_LEN;
            if (((txadd && rxadd) || ((txadd + rxadd) == 0)) &&
                !memcmp(adv_addr, adva, BLE_DEV_ADDR_LEN)) {
                /* We have received a scan response. Add to list */
                ble_ll_scan_add_scan_rsp_adv(adv_addr, txadd);

                /* Perform scan request backoff procedure */
                ble_ll_scan_req_backoff(scansm, 1);
            }
        }
    }

    /* Filter duplicates */
    if (scansm->scan_filt_dups) {
        if (ble_ll_scan_is_dup_adv(ptype, txadd, adv_addr)) {
            goto scan_continue;
        }
    }

    /* Send the advertising report */
    ble_ll_hci_send_adv_report(ptype, txadd, rxbuf, hdr->rxinfo.rssi);

scan_continue:
    /*
     * If the scan response check bit is set and we are pending a response,
     * we have failed the scan request (as we would have reset the scan rsp
     * pending flag if we received a valid response
     */
    if (scansm->scan_rsp_pending && scan_rsp_chk) {
        ble_ll_scan_req_backoff(scansm, 0);
    }

    ble_ll_scan_chk_resume();
    return;
}

int
ble_ll_scan_set_scan_params(uint8_t *cmd)
{
    uint8_t scan_type;
    uint8_t own_addr_type;
    uint8_t filter_policy;
    uint16_t scan_itvl;
    uint16_t scan_window;
    struct ble_ll_scan_sm *scansm;

    /* If already enabled, we return an error */
    scansm = &g_ble_ll_scan_sm;
    if (scansm->scan_enabled) {
        return BLE_ERR_CMD_DISALLOWED;
    }

    /* Get the scan interval and window */
    scan_type = cmd[0];
    scan_itvl  = le16toh(cmd + 1);
    scan_window = le16toh(cmd + 3);
    own_addr_type = cmd[5];
    filter_policy = cmd[6];

    /* Check scan type */
    if ((scan_type != BLE_HCI_SCAN_TYPE_PASSIVE) &&
        (scan_type != BLE_HCI_SCAN_TYPE_ACTIVE)) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    /* Check interval and window */
    if ((scan_itvl < BLE_HCI_SCAN_ITVL_MIN) ||
        (scan_itvl > BLE_HCI_SCAN_ITVL_MAX) ||
        (scan_window < BLE_HCI_SCAN_WINDOW_MIN) ||
        (scan_window > BLE_HCI_SCAN_WINDOW_MAX) ||
        (scan_itvl < scan_window)) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    /* Check own addr type */
    if (own_addr_type > BLE_HCI_ADV_OWN_ADDR_MAX) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    /* Check scanner filter policy */
    if (filter_policy > BLE_HCI_SCAN_FILT_MAX) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    /* Set state machine parameters */
    scansm->scan_type = scan_type;
    scansm->scan_itvl = scan_itvl;
    scansm->scan_window = scan_window;
    scansm->scan_filt_policy = filter_policy;
    scansm->own_addr_type = own_addr_type;

    return 0;
}

/**
 * ble ll scan set enable
 *
 *  HCI scan set enable command processing function
 *
 *  Context: Link Layer task (HCI Command parser).
 *
 * @param cmd Pointer to command buffer
 *
 * @return int BLE error code.
 */
int
ble_ll_scan_set_enable(uint8_t *cmd)
{
    int rc;
    uint8_t filter_dups;
    uint8_t enable;
    struct ble_ll_scan_sm *scansm;

    /* Check for valid parameters */
    enable = cmd[0];
    filter_dups = cmd[1];
    if ((filter_dups > 1) || (enable > 1)) {
        return BLE_ERR_INV_HCI_CMD_PARMS;
    }

    rc = BLE_ERR_SUCCESS;
    scansm = &g_ble_ll_scan_sm;
    if (enable) {
        /* If already enabled, do nothing */
        if (!scansm->scan_enabled) {
            /* Start the scanning state machine */
            scansm->scan_filt_dups = filter_dups;
            rc = ble_ll_scan_sm_start(scansm);
        }
    } else {
        if (scansm->scan_enabled) {
            ble_ll_scan_sm_stop(1);
        }
    }

    return rc;
}

/**
 * Checks if controller can change the whitelist. If scanning is enabled and
 * using the whitelist the controller is not allowed to change the whitelist.
 *
 * @return int 0: not allowed to change whitelist; 1: change allowed.
 */
int
ble_ll_scan_can_chg_whitelist(void)
{
    int rc;
    struct ble_ll_scan_sm *scansm;

    scansm = &g_ble_ll_scan_sm;
    if (scansm->scan_enabled && (scansm->scan_filt_policy & 1)) {
        rc = 0;
    } else {
        rc = 1;
    }

    return rc;
}

int
ble_ll_scan_initiator_start(struct hci_create_conn *hcc)
{
    struct ble_ll_scan_sm *scansm;

    scansm = &g_ble_ll_scan_sm;
    scansm->scan_type = BLE_SCAN_TYPE_INITIATE;
    scansm->scan_itvl = hcc->scan_itvl;
    scansm->scan_window = hcc->scan_window;
    scansm->scan_filt_policy = hcc->filter_policy;
    scansm->own_addr_type = hcc->own_addr_type;
    return ble_ll_scan_sm_start(scansm);
}

/**
 * Checks to see if the scanner is enabled.
 *
 * @return int 0: not enabled; enabled otherwise
 */
int
ble_ll_scan_enabled(void)
{
    return (int)g_ble_ll_scan_sm.scan_enabled;
}

/* Returns the PDU allocated by the scanner */
struct os_mbuf *
ble_ll_scan_get_pdu(void)
{
    return g_ble_ll_scan_sm.scan_req_pdu;
}

/* Returns the global scanning state machine */
struct ble_ll_scan_sm *
ble_ll_scan_sm_get(void)
{
    return &g_ble_ll_scan_sm;
}

/* Returns true if whitelist is enabled for scanning */
int
ble_ll_scan_whitelist_enabled(void)
{
    return g_ble_ll_scan_sm.scan_filt_policy & 1;
}

/**
 * Called when the controller receives the reset command. Resets the
 * scanning state machine to its initial state.
 *
 * @return int
 */
void
ble_ll_scan_reset(void)
{
    struct ble_ll_scan_sm *scansm;

    /* If enabled, stop it. */
    scansm = &g_ble_ll_scan_sm;
    if (scansm->scan_enabled) {
        ble_ll_scan_sm_stop(0);
    }

    /* Free the scan request pdu */
    os_mbuf_free_chain(scansm->scan_req_pdu);

    /* Reset duplicate advertisers and those from which we rxd a response */
    g_ble_ll_scan_num_rsp_advs = 0;
    memset(&g_ble_ll_scan_rsp_advs[0], 0, sizeof(g_ble_ll_scan_rsp_advs));

    g_ble_ll_scan_num_dup_advs = 0;
    memset(&g_ble_ll_scan_dup_advs[0], 0, sizeof(g_ble_ll_scan_dup_advs));

    /* Call the init function again */
    ble_ll_scan_init();
}

/**
 * ble ll scan init
 *
 * Initialize a scanner. Must be called before scanning can be started.
 * Expected to be called with a un-initialized or reset scanning state machine.
 */
void
ble_ll_scan_init(void)
{
    struct ble_ll_scan_sm *scansm;

    /* Clear state machine in case re-initialized */
    scansm = &g_ble_ll_scan_sm;
    memset(scansm, 0, sizeof(struct ble_ll_scan_sm));

    /* Initialize scanning window end event */
    scansm->scan_sched_ev.ev_type = BLE_LL_EVENT_SCAN;
    scansm->scan_sched_ev.ev_arg = scansm;

    /* Set all non-zero default parameters */
    scansm->scan_itvl = BLE_HCI_SCAN_ITVL_DEF;
    scansm->scan_window = BLE_HCI_SCAN_WINDOW_DEF;

    /* Initialize connection supervision timer */
    cputime_timer_init(&scansm->scan_timer, ble_ll_scan_timer_cb, scansm);

    /* Get a scan request mbuf (packet header) and attach to state machine */
    scansm->scan_req_pdu = os_msys_get_pkthdr(BLE_MBUF_PAYLOAD_SIZE,
                                              sizeof(struct ble_mbuf_hdr));
    assert(scansm->scan_req_pdu != NULL);
}

