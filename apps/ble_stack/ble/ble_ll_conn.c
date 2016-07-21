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
#include "ble/xcvr.h"
#include "controller/ble_ll.h"
#include "controller/ble_ll_hci.h"
#include "controller/ble_ll_scan.h"
#include "controller/ble_ll_whitelist.h"
#include "controller/ble_ll_sched.h"
#include "controller/ble_ll_ctrl.h"
#include "controller/ble_phy.h"
#include "ble_ll_conn_priv.h"
#include "hal/hal_cputime.h"
#include "hal/hal_gpio.h"

#if (BLETEST_THROUGHPUT_TEST == 1)
extern void bletest_completed_pkt(uint16_t handle);
#endif

/* XXX TODO
 * 1) I think if we are initiating and we already have a connection with
 * a device that we will still try and connect to it. Fix this.
 *  -> This is true. There are a couple things to do
 *      i) When a connection create is issued, if we already are connected
 *      deny it. BLE ERROR = 0x0B (ACL connection exists).
 *      ii) If we receive an advertisement while initiating and want to send
 *      a connect request to the device, make sure we dont have it.
 *      iii) I think I need to do something like this: I am initiating and
 *      advertising. Suppose the device I want to connect to sends me a connect
 *      request because I am advertising? What happens to connection? Deal
 *      with this!
 *
 * 2) Make sure we check incoming data packets for size and all that. You
 * know, supported octets and all that. For both rx and tx.
 *
 * 3) Make sure we are setting the schedule end time properly for both slave
 * and master. We should just set this to the end of the connection event.
 * We might want to guarantee a IFS time as well since the next event needs
 * to be scheduled prior to the start of the event to account for the time it
 * takes to get a frame ready (which is pretty much the IFS time).
 *
 * 4) looks like the current code will allow the 1st packet in a
 * connection to extend past the end of the allocated connection end
 * time. That is not good. Need to deal with that. Need to extend connection
 * end time.
 *
 * 6) Use error code 0x3E correctly! Connection failed to establish. If you
 * read the LE connection complete event, it says that if the connection
 * fails to be established that the connection complete event gets sent to
 * the host that issued the create connection. Need to resolve this.
 *
 * 7) How does peer address get set if we are using whitelist? Look at filter
 * policy and make sure you are doing this correctly.
 *
 * 8) Right now I use a fixed definition for required slots. CHange this.
 *
 * 10) See what connection state machine elements are purely master and
 * purely slave. We can make a union of them.
 *
 * 11) Not sure I am dealing with the connection terminate timeout perfectly.
 * I may extend a connection event too long although if it is always in terms
 * of connection events I am probably fine. Checking at end that the next
 * connection event will occur past terminate timeould would be fine.
 *
 * 12) When a slave receives a data packet in a connection it has to send a
 * response. Well, it should. If this packet will overrun the next scheduled
 * event, what should we do? Transmit anyway? Not transmit? For now, we just
 * transmit.
 */

/*
 * XXX: How should we deal with a late connection event? We need to determine
 * what we want to do under the following cases:
 *  1) The current connection event has not ended but a schedule item starts
 *  2) The connection event start cb is called but we are later than we
 *  expected. What to do? If we cant transmit at correct point in slot we
 *  are hosed. Well, anchor point can get really messed up!
 */

/* XXX: this does not belong here! Move to transport? */
extern int ble_hs_rx_data(struct os_mbuf *om);

/*
 * The amount of time that we will wait to hear the start of a receive
 * packet after we have transmitted a packet. This time is at least
 * an IFS time plus the time to receive the preamble and access address. We
 * add an additional 32 usecs just to be safe.
 *
 * XXX: move this definition and figure out how we determine the worst-case
 * jitter (spec. should have this).
 */
#define BLE_LL_WFR_USECS                    (BLE_LL_IFS + 40 + 32)

/* This is a dummy structure we use for the empty PDU */
struct ble_ll_empty_pdu
{
    struct os_mbuf om;
    struct os_mbuf_pkthdr pkt_hdr;
    struct ble_mbuf_hdr ble_hdr;
};

/* We cannot have more than 254 connections given our current implementation */
#if (NIMBLE_OPT_MAX_CONNECTIONS >= 255)
    #error "Maximum # of connections is 254"
#endif

/* Sleep clock accuracy table (in ppm) */
static const uint16_t g_ble_sca_ppm_tbl[8] =
{
    500, 250, 150, 100, 75, 50, 30, 20
};

/* Global LL connection parameters */
struct ble_ll_conn_global_params g_ble_ll_conn_params;

/* Pointer to connection state machine we are trying to create */
struct ble_ll_conn_sm *g_ble_ll_conn_create_sm;

/* Pointer to current connection */
struct ble_ll_conn_sm *g_ble_ll_conn_cur_sm;

/* Connection state machine array */
struct ble_ll_conn_sm g_ble_ll_conn_sm[NIMBLE_OPT_MAX_CONNECTIONS];

/* List of active connections */
struct ble_ll_conn_active_list g_ble_ll_conn_active_list;

/* List of free connections */
struct ble_ll_conn_free_list g_ble_ll_conn_free_list;

STATS_SECT_START(ble_ll_conn_stats)
    STATS_SECT_ENTRY(cant_set_sched)
    STATS_SECT_ENTRY(conn_ev_late)
    STATS_SECT_ENTRY(wfr_expirations)
    STATS_SECT_ENTRY(handle_not_found)
    STATS_SECT_ENTRY(no_conn_sm)
    STATS_SECT_ENTRY(no_free_conn_sm)
    STATS_SECT_ENTRY(rx_data_pdu_no_conn)
    STATS_SECT_ENTRY(rx_data_pdu_bad_aa)
    STATS_SECT_ENTRY(slave_rxd_bad_conn_req_params)
    STATS_SECT_ENTRY(slave_ce_failures)
    STATS_SECT_ENTRY(data_pdu_rx_dup)
    STATS_SECT_ENTRY(data_pdu_txg)
    STATS_SECT_ENTRY(data_pdu_txf)
    STATS_SECT_ENTRY(conn_req_txd)
    STATS_SECT_ENTRY(l2cap_enqueued)
    STATS_SECT_ENTRY(rx_ctrl_pdus)
    STATS_SECT_ENTRY(rx_l2cap_pdus)
    STATS_SECT_ENTRY(rx_malformed_ctrl_pdus)
    STATS_SECT_ENTRY(rx_bad_llid)
    STATS_SECT_ENTRY(tx_ctrl_pdus)
    STATS_SECT_ENTRY(tx_ctrl_bytes)
    STATS_SECT_ENTRY(tx_l2cap_pdus)
    STATS_SECT_ENTRY(tx_l2cap_bytes)
    STATS_SECT_ENTRY(tx_empty_pdus)
    STATS_SECT_ENTRY(mic_failures)
STATS_SECT_END
STATS_SECT_DECL(ble_ll_conn_stats) ble_ll_conn_stats;

STATS_NAME_START(ble_ll_conn_stats)
    STATS_NAME(ble_ll_conn_stats, cant_set_sched)
    STATS_NAME(ble_ll_conn_stats, conn_ev_late)
    STATS_NAME(ble_ll_conn_stats, wfr_expirations)
    STATS_NAME(ble_ll_conn_stats, handle_not_found)
    STATS_NAME(ble_ll_conn_stats, no_conn_sm)
    STATS_NAME(ble_ll_conn_stats, no_free_conn_sm)
    STATS_NAME(ble_ll_conn_stats, rx_data_pdu_no_conn)
    STATS_NAME(ble_ll_conn_stats, rx_data_pdu_bad_aa)
    STATS_NAME(ble_ll_conn_stats, slave_rxd_bad_conn_req_params)
    STATS_NAME(ble_ll_conn_stats, slave_ce_failures)
    STATS_NAME(ble_ll_conn_stats, data_pdu_rx_dup)
    STATS_NAME(ble_ll_conn_stats, data_pdu_txg)
    STATS_NAME(ble_ll_conn_stats, data_pdu_txf)
    STATS_NAME(ble_ll_conn_stats, conn_req_txd)
    STATS_NAME(ble_ll_conn_stats, l2cap_enqueued)
    STATS_NAME(ble_ll_conn_stats, rx_ctrl_pdus)
    STATS_NAME(ble_ll_conn_stats, rx_l2cap_pdus)
    STATS_NAME(ble_ll_conn_stats, rx_malformed_ctrl_pdus)
    STATS_NAME(ble_ll_conn_stats, rx_bad_llid)
    STATS_NAME(ble_ll_conn_stats, tx_ctrl_pdus)
    STATS_NAME(ble_ll_conn_stats, tx_ctrl_bytes)
    STATS_NAME(ble_ll_conn_stats, tx_l2cap_pdus)
    STATS_NAME(ble_ll_conn_stats, tx_l2cap_bytes)
    STATS_NAME(ble_ll_conn_stats, tx_empty_pdus)
    STATS_NAME(ble_ll_conn_stats, mic_failures)
STATS_NAME_END(ble_ll_conn_stats)

#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
/**
 * Called to determine if the received PDU is an empty PDU or not.
 */
static int
ble_ll_conn_is_empty_pdu(struct os_mbuf *rxpdu)
{
    int rc;
    uint8_t llid;

    llid = rxpdu->om_data[0] & BLE_LL_DATA_HDR_LLID_MASK;
    if ((llid == BLE_LL_LLID_DATA_FRAG) && (rxpdu->om_data[1] == 0)) {
        rc = 1;
    } else {
        rc = 0;
    }
    return rc;
}
#endif

/**
 * Called to return the currently running connection state machine end time.
 * Always called when interrupts are disabled.
 *
 * @return int 0: s1 is not least recently used. 1: s1 is least recently used
 */
int
ble_ll_conn_is_lru(struct ble_ll_conn_sm *s1, struct ble_ll_conn_sm *s2)
{
    int rc;

    /* Set time that we last serviced the schedule */
    if ((int32_t)(s1->last_scheduled - s2->last_scheduled) < 0) {
        rc = 1;
    } else {
        rc = 0;
    }

    return rc;
}

/**
 * Called to return the currently running connection state machine end time.
 * Always called when interrupts are disabled.
 *
 * @return uint32_t
 */
uint32_t
ble_ll_conn_get_ce_end_time(void)
{
    uint32_t ce_end_time;

    if (g_ble_ll_conn_cur_sm) {
        ce_end_time = g_ble_ll_conn_cur_sm->ce_end_time;
    } else {
        ce_end_time = cputime_get32();
    }
    return ce_end_time;
}

/**
 * Called when the current connection state machine is no longer being used.
 * This function will:
 *  -> Disable the PHY, which will prevent any transmit/receive interrupts.
 *  -> Disable the wait for response timer, if running.
 *  -> Remove the connection state machine from the scheduler.
 *  -> Sets the Link Layer state to standby.
 *  -> Sets the current state machine to NULL.
 *
 *  NOTE: the ordering of these function calls is important! We have to stop
 *  the PHY and remove the schedule item before we can set the state to
 *  standby and set the current state machine pointer to NULL.
 */
static void
ble_ll_conn_current_sm_over(void)
{
    /* Disable the PHY */
    ble_phy_disable();

    /* Disable the wfr timer */
    ble_ll_wfr_disable();

    /* Link-layer is in standby state now */
    ble_ll_state_set(BLE_LL_STATE_STANDBY);

    /* Set current LL connection to NULL */
    g_ble_ll_conn_cur_sm = NULL;
}

/**
 * Given a handle, find an active connection matching the handle
 *
 * @param handle
 *
 * @return struct ble_ll_conn_sm*
 */
struct ble_ll_conn_sm *
ble_ll_conn_find_active_conn(uint16_t handle)
{
    struct ble_ll_conn_sm *connsm;

    connsm = NULL;
    if ((handle != 0) && (handle <= NIMBLE_OPT_MAX_CONNECTIONS)) {
        connsm = &g_ble_ll_conn_sm[handle - 1];
        if (connsm->conn_state == BLE_LL_CONN_STATE_IDLE) {
            connsm = NULL;
        }
    }
    return connsm;
}

/**
 * Get a connection state machine.
 */
struct ble_ll_conn_sm *
ble_ll_conn_sm_get(void)
{
    struct ble_ll_conn_sm *connsm;

    connsm = STAILQ_FIRST(&g_ble_ll_conn_free_list);
    if (connsm) {
        STAILQ_REMOVE_HEAD(&g_ble_ll_conn_free_list, free_stqe);
    } else {
        STATS_INC(ble_ll_conn_stats, no_free_conn_sm);
    }

    return connsm;
}

/**
 * Calculate the amount of window widening for a given connection event. This
 * is the amount of time that a slave has to account for when listening for
 * the start of a connection event.
 *
 * @param connsm Pointer to connection state machine.
 *
 * @return uint32_t The current window widening amount (in microseconds)
 */
uint32_t
ble_ll_conn_calc_window_widening(struct ble_ll_conn_sm *connsm)
{
    uint32_t total_sca_ppm;
    uint32_t window_widening;
    int32_t time_since_last_anchor;
    uint32_t delta_msec;

    window_widening = 0;

    time_since_last_anchor = (int32_t)(connsm->anchor_point -
                                       connsm->last_anchor_point);
    if (time_since_last_anchor > 0) {
        delta_msec = cputime_ticks_to_usecs(time_since_last_anchor) / 1000;
        total_sca_ppm = g_ble_sca_ppm_tbl[connsm->master_sca] +
            NIMBLE_OPT_LL_OUR_SCA;
        window_widening = (total_sca_ppm * delta_msec) / 1000;
    }

    /* XXX: spec gives 16 usecs error btw. Probably should add that in */
    return window_widening;
}

/**
 * Calculates the number of used channels in the channel map
 *
 * @param chmap
 *
 * @return uint8_t Number of used channels
 */
uint8_t
ble_ll_conn_calc_used_chans(uint8_t *chmap)
{
    int i;
    int j;
    uint8_t mask;
    uint8_t chanbyte;
    uint8_t used_channels;

    used_channels = 0;
    for (i = 0; i < BLE_LL_CONN_CHMAP_LEN; ++i) {
        chanbyte = chmap[i];
        if (chanbyte) {
            if (chanbyte == 0xff) {
                used_channels += 8;
            } else {
                mask = 0x01;
                for (j = 0; j < 8; ++j) {
                    if (chanbyte & mask) {
                        ++used_channels;
                    }
                    mask <<= 1;
                }
            }
        }
    }
    return used_channels;
}

static uint32_t
ble_ll_conn_calc_access_addr(void)
{
    uint32_t aa;
    uint16_t aa_low;
    uint16_t aa_high;
    uint32_t temp;
    uint32_t mask;
    uint32_t prev_bit;
    uint8_t bits_diff;
    uint8_t consecutive;
    uint8_t transitions;

    /* Calculate a random access address */
    aa = 0;
    while (1) {
        /* Get two, 16-bit random numbers */
        aa_low = rand() & 0xFFFF;
        aa_high = rand() & 0xFFFF;

        /* All four bytes cannot be equal */
        if (aa_low == aa_high) {
            continue;
        }

        /* Upper 6 bits must have 2 transitions */
        temp = aa_high & 0xFC00;
        if ((temp == 0) || (temp == 0xFC00)) {
            continue;
        }

        /* Cannot be access address or be 1 bit different */
        aa = aa_high;
        aa = (aa << 16) | aa_low;
        bits_diff = 0;
        temp = aa ^ BLE_ACCESS_ADDR_ADV;
        for (mask = 0x00000001; mask != 0; mask <<= 1) {
            if (mask & temp) {
                ++bits_diff;
                if (bits_diff > 1) {
                    break;
                }
            }
        }
        if (bits_diff <= 1) {
            continue;
        }

        /* Cannot have more than 24 transitions */
        transitions = 0;
        consecutive = 0;
        mask = 0x00000001;
        prev_bit = aa & mask;
        while (mask < 0x80000000) {
            mask <<= 1;
            if (mask & aa) {
                if (prev_bit == 0) {
                    ++transitions;
                    consecutive = 0;
                } else {
                    ++consecutive;
                }
            } else {
                if (prev_bit == 0) {
                    ++consecutive;
                } else {
                    ++transitions;
                    consecutive = 0;
                }
            }

            /* This is invalid! */
            if (consecutive > 6) {
                break;
            }
        }

        /* Cannot be more than 24 transitions */
        if ((consecutive > 6) || (transitions > 24)) {
            continue;
        }

        /* We have a valid access address */
        break;
    }
    return aa;
}

/**
 * Determine data channel index to be used for the upcoming/current
 * connection event
 *
 * @param conn
 *
 * @return uint8_t
 */
uint8_t
ble_ll_conn_calc_dci(struct ble_ll_conn_sm *conn)
{
    int     i;
    int     j;
    uint8_t chan;
    uint8_t curchan;
    uint8_t remap_index;
    uint8_t bitpos;
    uint8_t cntr;
    uint8_t mask;
    uint8_t usable_chans;

    /* Get next unmapped channel */
    curchan = conn->last_unmapped_chan + conn->hop_inc;
    if (curchan > BLE_PHY_NUM_DATA_CHANS) {
        curchan -= BLE_PHY_NUM_DATA_CHANS;
    }

    /* Set the current unmapped channel */
    conn->unmapped_chan = curchan;

    /* Is this a valid channel? */
    bitpos = 1 << (curchan & 0x07);
    if ((conn->chanmap[curchan >> 3] & bitpos) == 0) {

        /* Calculate remap index */
        remap_index = curchan % conn->num_used_chans;

        /* NOTE: possible to build a map but this would use memory. For now,
           we just calculate */
        /* Iterate through channel map to find this channel */
        chan = 0;
        cntr = 0;
        for (i = 0; i < BLE_LL_CONN_CHMAP_LEN; i++) {
            usable_chans = conn->chanmap[i];
            if (usable_chans != 0) {
                mask = 0x01;
                for (j = 0; j < 8; j++) {
                    if (usable_chans & mask) {
                        if (cntr == remap_index) {
                            return (chan + j);
                        }
                        ++cntr;
                    }
                    mask <<= 1;
                }
            }
            chan += 8;
        }
    }

    return curchan;
}

/**
 * Called when we are in the connection state and the wait for response timer
 * fires off.
 *
 * Context: Interrupt
 */
void
ble_ll_conn_wfr_timer_exp(void)
{
    struct ble_ll_conn_sm *connsm;

    connsm = g_ble_ll_conn_cur_sm;
    ble_ll_conn_current_sm_over();
    if (connsm) {
        ble_ll_event_send(&connsm->conn_ev_end);
        STATS_INC(ble_ll_conn_stats, wfr_expirations);
    }
}

/**
 * Callback for slave when it transmits a data pdu and the connection event
 * ends after the transmission.
 *
 * Context: Interrupt
 *
 * @param sch
 *
 */
static void
ble_ll_conn_wait_txend(void *arg)
{
    struct ble_ll_conn_sm *connsm;

    ble_ll_conn_current_sm_over();

    connsm = (struct ble_ll_conn_sm *)arg;
    ble_ll_event_send(&connsm->conn_ev_end);
}

#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
static void
ble_ll_conn_start_rx_encrypt(void *arg)
{
    struct ble_ll_conn_sm *connsm;

    connsm = (struct ble_ll_conn_sm *)arg;
    CONN_F_ENCRYPTED(connsm) = 1;
    ble_phy_encrypt_enable(connsm->enc_data.rx_pkt_cntr,
                           connsm->enc_data.iv,
                           connsm->enc_data.enc_block.cipher_text,
                           !CONN_IS_MASTER(connsm));
}

static void
ble_ll_conn_start_rx_unencrypt(void *arg)
{
    struct ble_ll_conn_sm *connsm;

    connsm = (struct ble_ll_conn_sm *)arg;
    CONN_F_ENCRYPTED(connsm) = 0;
    ble_phy_encrypt_disable();
}

static void
ble_ll_conn_txend_encrypt(void *arg)
{
    struct ble_ll_conn_sm *connsm;

    connsm = (struct ble_ll_conn_sm *)arg;
    CONN_F_ENCRYPTED(connsm) = 1;
    ble_ll_conn_current_sm_over();
    ble_ll_event_send(&connsm->conn_ev_end);
}

static void
ble_ll_conn_rxend_unencrypt(void *arg)
{
    struct ble_ll_conn_sm *connsm;

    connsm = (struct ble_ll_conn_sm *)arg;
    CONN_F_ENCRYPTED(connsm) = 0;
    ble_ll_conn_current_sm_over();
    ble_ll_event_send(&connsm->conn_ev_end);
}

static void
ble_ll_conn_continue_rx_encrypt(void *arg)
{
    struct ble_ll_conn_sm *connsm;

    connsm = (struct ble_ll_conn_sm *)arg;
    ble_phy_encrypt_set_pkt_cntr(connsm->enc_data.rx_pkt_cntr,
                                 !CONN_IS_MASTER(connsm));
}
#endif

/**
 * Returns the cputime of the next scheduled item on the scheduler list or
 * when the current connection will start its next interval (whichever is
 * earlier). This API is called when determining at what time we should end
 * the current connection event. The current connection event must end before
 * the next scheduled item. However, the current connection itself is not
 * in the scheduler list! Thus, we need to calculate the time at which the
 * next connection will start and not overrun it.
 *
 * @param connsm
 *
 * @return uint32_t
 */
static uint32_t
ble_ll_conn_get_next_sched_time(struct ble_ll_conn_sm *connsm)
{
    uint32_t itvl;
    uint32_t ce_end;
    uint32_t next_sched_time;

    /* Calculate time at which next connection event will start */
    itvl = connsm->conn_itvl * BLE_LL_CONN_ITVL_USECS;
    ce_end = connsm->anchor_point + cputime_usecs_to_ticks(itvl);

    if (ble_ll_sched_next_time(&next_sched_time)) {
        if (CPUTIME_LT(next_sched_time, ce_end)) {
            ce_end = next_sched_time;
        }
    }

    return ce_end;
}

/**
 * Called to check if certain connection state machine flags have been
 * set.
 *
 * @param connsm
 */
static void
ble_ll_conn_chk_csm_flags(struct ble_ll_conn_sm *connsm)
{
    uint8_t update_status;

#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
    if (connsm->csmflags.cfbit.send_ltk_req) {
        /*
         * Send Long term key request event to host. If masked, we need to
         * send a REJECT_IND.
         */
        if (ble_ll_hci_ev_ltk_req(connsm)) {
            ble_ll_ctrl_reject_ind_send(connsm, BLE_LL_CTRL_ENC_REQ,
                                        BLE_ERR_PINKEY_MISSING);
        }
        connsm->csmflags.cfbit.send_ltk_req = 0;
    }
#endif

    /*
     * There are two cases where this flag gets set:
     * 1) A connection update procedure was started and the event counter
     * has passed the instant.
     * 2) We successfully sent the reject reason.
     */
    if (connsm->csmflags.cfbit.host_expects_upd_event) {
        update_status = BLE_ERR_SUCCESS;
        if (IS_PENDING_CTRL_PROC(connsm, BLE_LL_CTRL_PROC_CONN_UPDATE)) {
            ble_ll_ctrl_proc_stop(connsm, BLE_LL_CTRL_PROC_CONN_UPDATE);
        } else {
            if (IS_PENDING_CTRL_PROC(connsm, BLE_LL_CTRL_PROC_CONN_PARAM_REQ)) {
                ble_ll_ctrl_proc_stop(connsm, BLE_LL_CTRL_PROC_CONN_PARAM_REQ);
                update_status = connsm->reject_reason;
            }
        }
        ble_ll_hci_ev_conn_update(connsm, update_status);
        connsm->csmflags.cfbit.host_expects_upd_event = 0;
    }
}

/**
 * Called when we want to send a data channel pdu inside a connection event.
 *
 * Context: interrupt
 *
 * @param connsm
 *
 * @return int 0: success; otherwise failure to transmit
 */
static int
ble_ll_conn_tx_data_pdu(struct ble_ll_conn_sm *connsm)
{
    int rc;
    uint8_t md;
    uint8_t hdr_byte;
    uint8_t end_transition;
    uint8_t cur_txlen;
    uint8_t next_txlen;
    uint8_t cur_offset;
    uint16_t pktlen;
    uint32_t next_event_time;
    uint32_t ticks;
    struct os_mbuf *m;
    struct ble_mbuf_hdr *ble_hdr;
    struct os_mbuf_pkthdr *pkthdr;
    struct os_mbuf_pkthdr *nextpkthdr;
    struct ble_ll_empty_pdu empty_pdu;
    ble_phy_tx_end_func txend_func;

    /* For compiler warnings... */
    ble_hdr = NULL;
    m = NULL;
    md = 0;
    hdr_byte = BLE_LL_LLID_DATA_FRAG;

    /*
     * We need to check if we are retrying a pdu or if there is a pdu on
     * the transmit queue.
     */
    pkthdr = STAILQ_FIRST(&connsm->conn_txq);
    if (!connsm->cur_tx_pdu && !CONN_F_EMPTY_PDU_TXD(connsm) && !pkthdr) {
        CONN_F_EMPTY_PDU_TXD(connsm) = 1;
        goto conn_tx_pdu;
    }

    /*
     * If we dont have a pdu we have previously transmitted, take it off
     * the connection transmit queue
     */
    cur_offset = 0;
    if (!connsm->cur_tx_pdu && !CONN_F_EMPTY_PDU_TXD(connsm)) {
        /* Convert packet header to mbuf */
        m = OS_MBUF_PKTHDR_TO_MBUF(pkthdr);
        nextpkthdr = STAILQ_NEXT(pkthdr, omp_next);

#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
        /*
         * If we are encrypting, we are only allowed to send certain
         * kinds of LL control PDU's. If none is enqueued, send empty pdu!
         */
        if (connsm->enc_data.enc_state > CONN_ENC_S_ENCRYPTED) {
            if (!ble_ll_ctrl_enc_allowed_pdu(pkthdr)) {
                CONN_F_EMPTY_PDU_TXD(connsm) = 1;
                goto conn_tx_pdu;
            }

            /*
             * We will allow a next packet if it itself is allowed or we are
             * a slave and we are sending the START_ENC_RSP. The master has
             * to wait to receive the START_ENC_RSP from the slave before
             * packets can be let go.
             */
            if (nextpkthdr && !ble_ll_ctrl_enc_allowed_pdu(nextpkthdr)
                && ((connsm->conn_role == BLE_LL_CONN_ROLE_MASTER) ||
                    !ble_ll_ctrl_is_start_enc_rsp(m))) {
                nextpkthdr = NULL;
            }
        }
#endif
        /* Take packet off queue*/
        STAILQ_REMOVE_HEAD(&connsm->conn_txq, omp_next);
        ble_hdr = BLE_MBUF_HDR_PTR(m);

        /* Determine packet length we will transmit */
        cur_txlen = connsm->eff_max_tx_octets;
        pktlen = pkthdr->omp_len;
        if (cur_txlen > pktlen) {
            cur_txlen = pktlen;
        }
        ble_hdr->txinfo.pyld_len = cur_txlen;

        /* NOTE: header was set when first enqueued */
        hdr_byte = ble_hdr->txinfo.hdr_byte;
        connsm->cur_tx_pdu = m;
    } else {
        nextpkthdr = pkthdr;
        if (connsm->cur_tx_pdu) {
            m = connsm->cur_tx_pdu;
            ble_hdr = BLE_MBUF_HDR_PTR(m);
            pktlen = OS_MBUF_PKTLEN(m);
            cur_txlen = ble_hdr->txinfo.pyld_len;
            cur_offset = ble_hdr->txinfo.offset;
            if (cur_offset == 0) {
                hdr_byte = ble_hdr->txinfo.hdr_byte & BLE_LL_DATA_HDR_LLID_MASK;
            }
#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
            if (connsm->enc_data.enc_state > CONN_ENC_S_ENCRYPTED) {
                /* We will allow a next packet if it itself is allowed */
                pkthdr = OS_MBUF_PKTHDR(connsm->cur_tx_pdu);
                if (nextpkthdr && !ble_ll_ctrl_enc_allowed_pdu(nextpkthdr)
                    && ((connsm->conn_role == BLE_LL_CONN_ROLE_MASTER) ||
                        !ble_ll_ctrl_is_start_enc_rsp(connsm->cur_tx_pdu))) {
                    nextpkthdr = NULL;
                }
            }
#endif
        } else {
            /* Empty PDU here. NOTE: header byte gets set later */
            pktlen = 0;
            cur_txlen = 0;
#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
            if (connsm->enc_data.enc_state > CONN_ENC_S_ENCRYPTED) {
                /* We will allow a next packet if it itself is allowed */
                if (nextpkthdr && !ble_ll_ctrl_enc_allowed_pdu(nextpkthdr)) {
                    nextpkthdr = NULL;
                }
            }
#endif
        }
    }

    /*
     * Set the more data data flag if we have more data to send and we
     * have not been asked to terminate
     */
    if ((nextpkthdr || ((cur_offset + cur_txlen) < pktlen)) &&
         !connsm->csmflags.cfbit.terminate_ind_rxd) {
        /* Get next event time */
        next_event_time = ble_ll_conn_get_next_sched_time(connsm);

        /*
         * Dont bother to set the MD bit if we cannot do the following:
         *  -> wait IFS, send the current frame.
         *  -> wait IFS, receive a maximum size frame.
         *  -> wait IFS, send the next frame.
         *  -> wait IFS, receive a maximum size frame.
         *
         *  For slave:
         *  -> wait IFS, send current frame.
         *  -> wait IFS, receive maximum size frame.
         *  -> wait IFS, send next frame.
         */
        if ((cur_offset + cur_txlen) < pktlen) {
            next_txlen = pktlen - (cur_offset + cur_txlen);
        } else {
            if (nextpkthdr->omp_len > connsm->eff_max_tx_octets) {
                next_txlen = connsm->eff_max_tx_octets;
            } else {
                next_txlen = nextpkthdr->omp_len;
            }
        }

        /*
         * XXX: this calculation is based on using the current time
         * and assuming the transmission will occur an IFS time from
         * now. This is not the most accurate especially if we have
         * received a frame and we are replying to it.
         */
        ticks = (BLE_LL_IFS * 3) + connsm->eff_max_rx_time +
                BLE_TX_DUR_USECS_M(next_txlen) +
                BLE_TX_DUR_USECS_M(cur_txlen);

        if (connsm->conn_role == BLE_LL_CONN_ROLE_MASTER) {
            ticks += (BLE_LL_IFS + connsm->eff_max_rx_time);
        }

        ticks = cputime_usecs_to_ticks(ticks);
        if ((cputime_get32() + ticks) < next_event_time) {
            md = 1;
        }
     }

    /* If we send an empty PDU we need to initialize the header */
conn_tx_pdu:
    if (CONN_F_EMPTY_PDU_TXD(connsm)) {
        /*
         * This looks strange, but we dont use the data pointer in the mbuf
         * when we have an empty pdu.
         */
        m = (struct os_mbuf *)&empty_pdu;
        m->om_data = (uint8_t *)&empty_pdu;
        m->om_data += BLE_MBUF_MEMBLOCK_OVERHEAD;
        ble_hdr = &empty_pdu.ble_hdr;
        ble_hdr->txinfo.flags = 0;
        ble_hdr->txinfo.offset = 0;
        ble_hdr->txinfo.pyld_len = 0;
    }

    /* Set tx seqnum */
    if (connsm->tx_seqnum) {
        hdr_byte |= BLE_LL_DATA_HDR_SN_MASK;
    }

    /* If we have more data, set the bit */
    if (md) {
        hdr_byte |= BLE_LL_DATA_HDR_MD_MASK;
    }

    /* Set NESN (next expected sequence number) bit */
    if (connsm->next_exp_seqnum) {
        hdr_byte |= BLE_LL_DATA_HDR_NESN_MASK;
    }

    /* Set the header byte in the outgoing frame */
    ble_hdr->txinfo.hdr_byte = hdr_byte;

    /*
     * If we are a slave, check to see if this transmission will end the
     * connection event. We will end the connection event if we have
     * received a valid frame with the more data bit set to 0 and we dont
     * have more data.
     *
     * XXX: for a slave, we dont check to see if we can:
     *  -> wait IFS, rx frame from master (either big or small).
     *  -> wait IFS, send empty pdu or next pdu.
     *
     *  We could do this. Now, we just keep going and hope that we dont
     *  overrun next scheduled item.
     */
    if ((connsm->csmflags.cfbit.terminate_ind_rxd) ||
        ((connsm->conn_role == BLE_LL_CONN_ROLE_SLAVE) && (md == 0) &&
         (connsm->cons_rxd_bad_crc == 0) &&
         ((connsm->last_rxd_hdr_byte & BLE_LL_DATA_HDR_MD_MASK) == 0) &&
         !ble_ll_ctrl_is_terminate_ind(hdr_byte, m->om_data[0]))) {
        /* We will end the connection event */
        end_transition = BLE_PHY_TRANSITION_NONE;
        txend_func = ble_ll_conn_wait_txend;
    } else {
        /* Wait for a response here */
        end_transition = BLE_PHY_TRANSITION_TX_RX;
        txend_func = NULL;
    }

#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
    int is_ctrl;
    uint8_t llid;
    uint8_t opcode;

    llid = ble_hdr->txinfo.hdr_byte & BLE_LL_DATA_HDR_LLID_MASK;
    if (llid == BLE_LL_LLID_CTRL) {
        is_ctrl = 1;
        opcode = m->om_data[0];
    } else {
        is_ctrl = 0;
    }

    if (is_ctrl && (opcode == BLE_LL_CTRL_START_ENC_RSP)) {
        /*
         * Both master and slave send the START_ENC_RSP encrypted and receive
         * encrypted
         */
        CONN_F_ENCRYPTED(connsm) = 1;
        connsm->enc_data.tx_encrypted = 1;
        ble_phy_encrypt_enable(connsm->enc_data.tx_pkt_cntr,
                               connsm->enc_data.iv,
                               connsm->enc_data.enc_block.cipher_text,
                               CONN_IS_MASTER(connsm));
    } else if (is_ctrl && (opcode == BLE_LL_CTRL_START_ENC_REQ)) {
        /*
         * Only the slave sends this and it gets sent unencrypted but
         * we receive encrypted
         */
        CONN_F_ENCRYPTED(connsm) = 0;
        connsm->enc_data.enc_state = CONN_ENC_S_START_ENC_RSP_WAIT;
        connsm->enc_data.tx_encrypted = 0;
        ble_phy_encrypt_disable();
        if (txend_func == NULL) {
            txend_func = ble_ll_conn_start_rx_encrypt;
        } else {
            txend_func = ble_ll_conn_txend_encrypt;
        }
    } else if (is_ctrl && (opcode == BLE_LL_CTRL_PAUSE_ENC_RSP)) {
        /*
         * The slave sends the PAUSE_ENC_RSP encrypted. The master sends
         * it unencrypted (note that link was already set unencrypted).
         */
        if (connsm->conn_role == BLE_LL_CONN_ROLE_SLAVE) {
            CONN_F_ENCRYPTED(connsm) = 1;
            connsm->enc_data.tx_encrypted = 1;
            ble_phy_encrypt_enable(connsm->enc_data.tx_pkt_cntr,
                                   connsm->enc_data.iv,
                                   connsm->enc_data.enc_block.cipher_text,
                                   CONN_IS_MASTER(connsm));
            if (txend_func == NULL) {
                txend_func = ble_ll_conn_start_rx_unencrypt;
            } else {
                txend_func = ble_ll_conn_rxend_unencrypt;
            }
        } else {
            CONN_F_ENCRYPTED(connsm) = 0;
            connsm->enc_data.enc_state = CONN_ENC_S_UNENCRYPTED;
            connsm->enc_data.tx_encrypted = 0;
            ble_phy_encrypt_disable();
        }
    } else {
        /* If encrypted set packet counter */
        if (CONN_F_ENCRYPTED(connsm)) {
            connsm->enc_data.tx_encrypted = 1;
            ble_phy_encrypt_set_pkt_cntr(connsm->enc_data.tx_pkt_cntr,
                                         CONN_IS_MASTER(connsm));
            if (txend_func == NULL) {
                txend_func = ble_ll_conn_continue_rx_encrypt;
            }
        }
    }
#endif

    /* Set transmit end callback */
    ble_phy_set_txend_cb(txend_func, connsm);
    rc = ble_phy_tx(m, end_transition);
    if (!rc) {
        /* Log transmit on connection state */
        cur_txlen = ble_hdr->txinfo.pyld_len;
        ble_ll_log(BLE_LL_LOG_ID_CONN_TX,
                   hdr_byte,
                   ((uint16_t)ble_hdr->txinfo.offset << 8) | cur_txlen,
                   (uint32_t)m);

        /* Set last transmitted MD bit */
        CONN_F_LAST_TXD_MD(connsm) = md;

        /* Increment packets transmitted */
        if (CONN_F_EMPTY_PDU_TXD(connsm)) {
            STATS_INC(ble_ll_conn_stats, tx_empty_pdus);
        } else if ((hdr_byte & BLE_LL_DATA_HDR_LLID_MASK) == BLE_LL_LLID_CTRL) {
            STATS_INC(ble_ll_conn_stats, tx_ctrl_pdus);
            STATS_INCN(ble_ll_conn_stats, tx_ctrl_bytes, cur_txlen);
        } else {
            STATS_INC(ble_ll_conn_stats, tx_l2cap_pdus);
            STATS_INCN(ble_ll_conn_stats, tx_l2cap_bytes, cur_txlen);
        }
    }
    return rc;
}

/**
 * Schedule callback for start of connection event.
 *
 * Context: Interrupt
 *
 * @param sch
 *
 * @return int 0: scheduled item is still running. 1: schedule item is done.
 */
static int
ble_ll_conn_event_start_cb(struct ble_ll_sched_item *sch)
{
    int rc;
    uint32_t usecs;
    uint32_t wfr_time;
    struct ble_ll_conn_sm *connsm;

    /* XXX: note that we can extend end time here if we want. Look at this */

    /* Set current connection state machine */
    connsm = (struct ble_ll_conn_sm *)sch->cb_arg;
    g_ble_ll_conn_cur_sm = connsm;
    assert(connsm);

    /* Disable whitelisting as connections do not use it */
    ble_ll_whitelist_disable();

    /* Set LL state */
    ble_ll_state_set(BLE_LL_STATE_CONNECTION);

    /* Log connection event start */
    ble_ll_log(BLE_LL_LOG_ID_CONN_EV_START, (uint8_t)connsm->conn_handle,
               (uint16_t)connsm->ce_end_time, connsm->csmflags.conn_flags);

    /* Set channel */
    ble_phy_setchan(connsm->data_chan_index, connsm->access_addr,
                    connsm->crcinit);

    if (connsm->conn_role == BLE_LL_CONN_ROLE_MASTER) {
        /* Set start time of transmission */
        rc = ble_phy_tx_set_start_time(sch->start_time + XCVR_PROC_DELAY_USECS);
        if (!rc) {
#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
            if (CONN_F_ENCRYPTED(connsm)) {
                ble_phy_encrypt_enable(connsm->enc_data.tx_pkt_cntr,
                                       connsm->enc_data.iv,
                                       connsm->enc_data.enc_block.cipher_text,
                                       1);
            } else {
                ble_phy_encrypt_disable();
            }
#endif
            rc = ble_ll_conn_tx_data_pdu(connsm);
            if (!rc) {
                rc = BLE_LL_SCHED_STATE_RUNNING;
            } else {
                /* Inform LL task of connection event end */
                rc = BLE_LL_SCHED_STATE_DONE;
            }
        } else {
            STATS_INC(ble_ll_conn_stats, conn_ev_late);
            rc = BLE_LL_SCHED_STATE_DONE;
        }
    } else {
#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
            if (CONN_F_ENCRYPTED(connsm)) {
                ble_phy_encrypt_enable(connsm->enc_data.rx_pkt_cntr,
                                       connsm->enc_data.iv,
                                       connsm->enc_data.enc_block.cipher_text,
                                       1);
            } else {
                ble_phy_encrypt_disable();
            }
#endif
        /*
         * XXX: make sure I dont care that I get here early to start receiving.
         * I could use events compare and all that shit to start rx.
         */
        rc = ble_phy_rx();
        if (rc) {
            /* End the connection event as we have no more buffers */
            STATS_INC(ble_ll_conn_stats, slave_ce_failures);
            rc = BLE_LL_SCHED_STATE_DONE;
        } else {
            /*
             * Set flag that tells slave to set last anchor point if a packet
             * has been received.
             */
            connsm->csmflags.cfbit.slave_set_last_anchor = 1;

            /*
             * Set the wait for response time. The anchor point is when we
             * expect the master to start transmitting. Worst-case, we expect
             * to hear a reply within the anchor point plus:
             *  -> the current tx window size
             *  -> The current window widening amount
             *  -> Amount of time it takes to detect packet start.
             */
            usecs = connsm->slave_cur_tx_win_usecs + BLE_LL_WFR_USECS +
                connsm->slave_cur_window_widening;
            wfr_time = connsm->anchor_point + cputime_usecs_to_ticks(usecs);
            ble_ll_wfr_enable(wfr_time);

            /* Set next wakeup time to connection event end time */
            rc = BLE_LL_SCHED_STATE_RUNNING;
        }
    }

    if (rc == BLE_LL_SCHED_STATE_DONE) {
        ble_ll_event_send(&connsm->conn_ev_end);
        ble_ll_state_set(BLE_LL_STATE_STANDBY);
        g_ble_ll_conn_cur_sm = NULL;
    }

    /* Set time that we last serviced the schedule */
    connsm->last_scheduled = cputime_get32();
    return rc;
}

/**
 * Called to determine if the device is allowed to send the next pdu in the
 * connection event. This will always return 'true' if we are a slave. If we
 * are a master, we must be able to send the next fragment and get a minimum
 * sized response from the slave.
 *
 * Context: Interrupt context (rx end isr).
 *
 * @param connsm
 * @param begtime   Time at which IFS before pdu transmission starts
 *
 * @return int 0: not allowed to send 1: allowed to send
 */
static int
ble_ll_conn_can_send_next_pdu(struct ble_ll_conn_sm *connsm, uint32_t begtime)
{
    int rc;
    uint8_t rem_bytes;
    uint32_t ticks;
    uint32_t next_sched_time;
    struct os_mbuf *txpdu;
    struct os_mbuf_pkthdr *pkthdr;
    struct ble_mbuf_hdr *txhdr;

    rc = 1;
    if (connsm->conn_role == BLE_LL_CONN_ROLE_MASTER) {
        /* Get next scheduled item time */
        next_sched_time = ble_ll_conn_get_next_sched_time(connsm);

        txpdu = connsm->cur_tx_pdu;
        if (!txpdu) {
            pkthdr = STAILQ_FIRST(&connsm->conn_txq);
            if (pkthdr) {
                txpdu = OS_MBUF_PKTHDR_TO_MBUF(pkthdr);
            }
        } else {
            pkthdr = OS_MBUF_PKTHDR(txpdu);
        }

        if (txpdu) {
            txhdr = BLE_MBUF_HDR_PTR(txpdu);
            rem_bytes = pkthdr->omp_len - txhdr->txinfo.offset;
            if (rem_bytes > connsm->eff_max_tx_octets) {
                rem_bytes = connsm->eff_max_tx_octets;
            }
            ticks = BLE_TX_DUR_USECS_M(rem_bytes);
        } else {
            /* We will send empty pdu (just a LL header) */
            ticks = BLE_TX_DUR_USECS_M(0);
        }
        ticks += (BLE_LL_IFS * 2) + connsm->eff_max_rx_time;
        ticks = cputime_usecs_to_ticks(ticks);
        if ((begtime + ticks) >= next_sched_time) {
            rc = 0;
        }
    }

    return rc;
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
ble_ll_conn_spvn_timer_cb(void *arg)
{
    struct ble_ll_conn_sm *connsm;

    connsm = (struct ble_ll_conn_sm *)arg;
    ble_ll_event_send(&connsm->conn_spvn_ev);
}

/**
 * Called when a create connection command has been received. This initializes
 * a connection state machine in the master role.
 *
 * NOTE: Must be called before the state machine is started
 *
 * @param connsm
 * @param hcc
 */
void
ble_ll_conn_master_init(struct ble_ll_conn_sm *connsm,
                        struct hci_create_conn *hcc)
{
    /* Set master role */
    connsm->conn_role = BLE_LL_CONN_ROLE_MASTER;

    /* Set default ce parameters */
    connsm->tx_win_size = BLE_LL_CONN_TX_WIN_MIN;
    connsm->tx_win_off = 0;
    connsm->master_sca = NIMBLE_OPT_LL_MASTER_SCA;

    /* Hop increment is a random value between 5 and 16. */
    connsm->hop_inc = (rand() % 12) + 5;

    /* Set slave latency and supervision timeout */
    connsm->slave_latency = hcc->conn_latency;
    connsm->supervision_tmo = hcc->supervision_timeout;

    /* Set own address type and peer address if needed */
    connsm->own_addr_type = hcc->own_addr_type;
    if (hcc->filter_policy == 0) {
        memcpy(&connsm->peer_addr, &hcc->peer_addr, BLE_DEV_ADDR_LEN);
        connsm->peer_addr_type = hcc->peer_addr_type;
    }

    /* XXX: for now, just make connection interval equal to max */
    connsm->conn_itvl_min = hcc->conn_itvl_min;
    connsm->conn_itvl_max = hcc->conn_itvl_max;
    connsm->conn_itvl = hcc->conn_itvl_max;

    /* Check the min/max CE lengths are less than connection interval */
    if (hcc->min_ce_len > (connsm->conn_itvl * 2)) {
        connsm->min_ce_len = connsm->conn_itvl * 2;
    } else {
        connsm->min_ce_len = hcc->min_ce_len;
    }

    if (hcc->max_ce_len > (connsm->conn_itvl * 2)) {
        connsm->max_ce_len = connsm->conn_itvl * 2;
    } else {
        connsm->max_ce_len = hcc->max_ce_len;
    }

    /* Set channel map to map requested by host */
    connsm->num_used_chans = g_ble_ll_conn_params.num_used_chans;
    memcpy(connsm->chanmap, g_ble_ll_conn_params.master_chan_map,
           BLE_LL_CONN_CHMAP_LEN);

    /*  Calculate random access address and crc initialization value */
    connsm->access_addr = ble_ll_conn_calc_access_addr();
    connsm->crcinit = rand() & 0xffffff;

    /* Set initial schedule callback */
    connsm->conn_sch.sched_cb = ble_ll_conn_event_start_cb;
}

/**
 * Create a new connection state machine. This is done once per
 * connection when the HCI command "create connection" is issued to the
 * controller or when a slave receives a connect request.
 *
 * Context: Link Layer task
 *
 * @param connsm
 */
void
ble_ll_conn_sm_new(struct ble_ll_conn_sm *connsm)
{
    struct ble_ll_conn_global_params *conn_params;

    /* Reset following elements */
    connsm->csmflags.conn_flags = 0;
    connsm->event_cntr = 0;
    connsm->conn_state = BLE_LL_CONN_STATE_IDLE;
    connsm->disconnect_reason = 0;
    connsm->common_features = 0;
    connsm->vers_nr = 0;
    connsm->comp_id = 0;
    connsm->sub_vers_nr = 0;
    connsm->reject_reason = BLE_ERR_SUCCESS;
    connsm->conn_rssi = BLE_LL_CONN_UNKNOWN_RSSI;

    /* Reset current control procedure */
    connsm->cur_ctrl_proc = BLE_LL_CTRL_PROC_IDLE;
    connsm->pending_ctrl_procs = 0;

    /*
     * Set handle in connection update procedure to 0. If the handle
     * is non-zero it means that the host initiated the connection
     * parameter update request and the rest of the parameters are valid.
     */
    connsm->conn_param_req.handle = 0;

    /* Initialize connection supervision timer */
    cputime_timer_init(&connsm->conn_spvn_timer, ble_ll_conn_spvn_timer_cb,
                       connsm);

    /* Calculate the next data channel */
    connsm->last_unmapped_chan = 0;
    connsm->data_chan_index = ble_ll_conn_calc_dci(connsm);

    /* Initialize event */
    connsm->conn_spvn_ev.ev_arg = connsm;
    connsm->conn_spvn_ev.ev_queued = 0;
    connsm->conn_spvn_ev.ev_type = BLE_LL_EVENT_CONN_SPVN_TMO;

    /* Connection end event */
    connsm->conn_ev_end.ev_arg = connsm;
    connsm->conn_ev_end.ev_queued = 0;
    connsm->conn_ev_end.ev_type = BLE_LL_EVENT_CONN_EV_END;

    /* Initialize transmit queue and ack/flow control elements */
    STAILQ_INIT(&connsm->conn_txq);
    connsm->cur_tx_pdu = NULL;
    connsm->tx_seqnum = 0;
    connsm->next_exp_seqnum = 0;
    connsm->cons_rxd_bad_crc = 0;
    connsm->last_rxd_sn = 1;
    connsm->completed_pkts = 0;

    /* initialize data length mgmt */
    conn_params = &g_ble_ll_conn_params;
    connsm->max_tx_octets = conn_params->conn_init_max_tx_octets;
    connsm->max_rx_octets = conn_params->supp_max_rx_octets;
    connsm->max_tx_time = conn_params->conn_init_max_tx_time;
    connsm->max_rx_time = conn_params->supp_max_rx_time;
    connsm->rem_max_tx_time = BLE_LL_CONN_SUPP_TIME_MIN;
    connsm->rem_max_rx_time = BLE_LL_CONN_SUPP_TIME_MIN;
    connsm->eff_max_tx_time = BLE_LL_CONN_SUPP_TIME_MIN;
    connsm->eff_max_rx_time = BLE_LL_CONN_SUPP_TIME_MIN;
    connsm->rem_max_tx_octets = BLE_LL_CONN_SUPP_BYTES_MIN;
    connsm->rem_max_rx_octets = BLE_LL_CONN_SUPP_BYTES_MIN;
    connsm->eff_max_tx_octets = BLE_LL_CONN_SUPP_BYTES_MIN;
    connsm->eff_max_rx_octets = BLE_LL_CONN_SUPP_BYTES_MIN;

    /* Reset encryption data */
#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
    memset(&connsm->enc_data, 0, sizeof(struct ble_ll_conn_enc_data));
    connsm->enc_data.enc_state = CONN_ENC_S_UNENCRYPTED;
#endif

    /* Add to list of active connections */
    SLIST_INSERT_HEAD(&g_ble_ll_conn_active_list, connsm, act_sle);
}

/**
 * Called when a remotes data length parameters change.
 *
 * Context: Link Layer task
 *
 * @param connsm
 * @param req
 */
void
ble_ll_conn_datalen_update(struct ble_ll_conn_sm *connsm,
                           struct ble_ll_len_req *req)
{
    int send_event;
    uint16_t eff_time;
    uint16_t eff_bytes;

    /* Update parameters */
    connsm->rem_max_rx_time = req->max_rx_time;
    connsm->rem_max_tx_time = req->max_tx_time;
    connsm->rem_max_rx_octets = req->max_rx_bytes;
    connsm->rem_max_tx_octets = req->max_tx_bytes;

    /* Assume no event sent */
    send_event = 0;

    /* See if effective times have changed */
    eff_time = min(connsm->rem_max_tx_time, connsm->max_rx_time);
    if (eff_time != connsm->eff_max_rx_time) {
        connsm->eff_max_rx_time = eff_time;
        send_event = 1;
    }
    eff_time = min(connsm->rem_max_rx_time, connsm->max_tx_time);
    if (eff_time != connsm->eff_max_tx_time) {
        connsm->eff_max_tx_time = eff_time;
        send_event = 1;
    }
    eff_bytes = min(connsm->rem_max_tx_octets, connsm->max_rx_octets);
    if (eff_bytes != connsm->eff_max_rx_octets) {
        connsm->eff_max_rx_octets = eff_bytes;
        send_event = 1;
    }
    eff_bytes = min(connsm->rem_max_rx_octets, connsm->max_tx_octets);
    if (eff_bytes != connsm->eff_max_tx_octets) {
        connsm->eff_max_tx_octets = eff_bytes;
        send_event = 1;
    }

    if (send_event) {
        ble_ll_hci_ev_datalen_chg(connsm);
    }
}

/**
 * Called when a connection is terminated
 *
 * Context: Link Layer task.
 *
 * @param connsm
 * @param ble_err
 */
void
ble_ll_conn_end(struct ble_ll_conn_sm *connsm, uint8_t ble_err)
{
    struct os_mbuf *m;
    struct os_mbuf_pkthdr *pkthdr;

    /* Remove scheduler events just in case */
    ble_ll_sched_rmv_elem(&connsm->conn_sch);

    /* Stop supervision timer */
    cputime_timer_stop(&connsm->conn_spvn_timer);

    /* Stop any control procedures that might be running */
    os_callout_stop(&connsm->ctrl_proc_rsp_timer.cf_c);

    /* Remove from the active connection list */
    SLIST_REMOVE(&g_ble_ll_conn_active_list, connsm, ble_ll_conn_sm, act_sle);

    /* Free the current transmit pdu if there is one. */
    if (connsm->cur_tx_pdu) {
        os_mbuf_free_chain(connsm->cur_tx_pdu);
        connsm->cur_tx_pdu = NULL;
    }

    /* Free all packets on transmit queue */
    while (1) {
        /* Get mbuf pointer from packet header pointer */
        pkthdr = STAILQ_FIRST(&connsm->conn_txq);
        if (!pkthdr) {
            break;
        }
        STAILQ_REMOVE_HEAD(&connsm->conn_txq, omp_next);

        m = (struct os_mbuf *)((uint8_t *)pkthdr - sizeof(struct os_mbuf));
        os_mbuf_free_chain(m);
    }

    /* Make sure events off queue */
    os_eventq_remove(&g_ble_ll_data.ll_evq, &connsm->conn_spvn_ev);
    os_eventq_remove(&g_ble_ll_data.ll_evq, &connsm->conn_ev_end);

    /* Connection state machine is now idle */
    connsm->conn_state = BLE_LL_CONN_STATE_IDLE;

    /*
     * We need to send a disconnection complete event or a connection complete
     * event when the connection ends. We send a connection complete event
     * only when we were told to cancel the connection creation. If the
     * ble error is "success" it means that the reset command was received
     * and we should not send an event
     */
    if (ble_err) {
        if (ble_err == BLE_ERR_UNK_CONN_ID) {
            ble_ll_conn_comp_event_send(connsm, ble_err);
        } else {
            ble_ll_disconn_comp_event_send(connsm, ble_err);
        }
    }

    /* Put connection state machine back on free list */
    STAILQ_INSERT_TAIL(&g_ble_ll_conn_free_list, connsm, free_stqe);

    /* Log connection end */
    ble_ll_log(BLE_LL_LOG_ID_CONN_END,connsm->conn_handle,0,connsm->event_cntr);
}

/**
 * Called to move to the next connection event.
 *
 * @param connsm
 *
 * @return int
 */
static int
ble_ll_conn_next_event(struct ble_ll_conn_sm *connsm)
{
    uint16_t latency;
    uint32_t itvl;
    uint32_t tmo;
    uint32_t cur_ww;
    uint32_t max_ww;
    struct ble_ll_conn_upd_req *upd;

    /* XXX: deal with connection request procedure here as well */
    ble_ll_conn_chk_csm_flags(connsm);

    /* Set event counter to the next connection event that we will tx/rx in */
    itvl = connsm->conn_itvl * BLE_LL_CONN_ITVL_USECS;
    latency = 1;
    if (connsm->csmflags.cfbit.allow_slave_latency      &&
        !connsm->csmflags.cfbit.conn_update_sched       &&
        !connsm->csmflags.cfbit.chanmap_update_scheduled) {
        if (connsm->csmflags.cfbit.pkt_rxd) {
            latency += connsm->slave_latency;
            itvl = itvl * latency;
        }
    }
    connsm->event_cntr += latency;

    /* Set next connection event start time */
    connsm->anchor_point += cputime_usecs_to_ticks(itvl);

    /*
     * If a connection update has been scheduled and the event counter
     * is now equal to the instant, we need to adjust the start of the
     * connection by the the transmit window offset. We also copy in the
     * update parameters as they now should take effect.
     */
    if (connsm->csmflags.cfbit.conn_update_sched &&
        (connsm->event_cntr == connsm->conn_update_req.instant)) {

        /* Set flag so we send connection update event */
        upd = &connsm->conn_update_req;
        if ((connsm->conn_role == BLE_LL_CONN_ROLE_MASTER)  ||
            (connsm->conn_itvl != upd->interval)            ||
            (connsm->slave_latency != upd->latency)         ||
            (connsm->supervision_tmo != upd->timeout)) {
            connsm->csmflags.cfbit.host_expects_upd_event = 1;
        }

        connsm->conn_itvl = upd->interval;
        connsm->supervision_tmo = upd->timeout;
        connsm->slave_latency = upd->latency;
        connsm->tx_win_size = upd->winsize;
        connsm->slave_cur_tx_win_usecs =
            connsm->tx_win_size * BLE_LL_CONN_TX_WIN_USECS;
        connsm->tx_win_off = upd->winoffset;
        connsm->anchor_point +=
            cputime_usecs_to_ticks(upd->winoffset * BLE_LL_CONN_ITVL_USECS);

        /* Reset the connection supervision timeout */
        cputime_timer_stop(&connsm->conn_spvn_timer);
        tmo = connsm->supervision_tmo;
        tmo = tmo * BLE_HCI_CONN_SPVN_TMO_UNITS * 1000;
        tmo = cputime_usecs_to_ticks(tmo);
        cputime_timer_start(&connsm->conn_spvn_timer, connsm->anchor_point+tmo);

        /* Reset update scheduled flag */
        connsm->csmflags.cfbit.conn_update_sched = 0;
    }

    /*
     * If there is a channel map request pending and we have reached the
     * instant, change to new channel map. Note there is a special case here.
     * If we received a channel map update with an instant equal to the event
     * counter, when we get here the event counter has already been
     * incremented by 1. That is why we do a signed comparison and change to
     * new channel map once the event counter equals or has passed channel
     * map update instant.
     */
    if (connsm->csmflags.cfbit.chanmap_update_scheduled &&
        ((int16_t)(connsm->chanmap_instant - connsm->event_cntr) <= 0)) {

        /* XXX: there is a chance that the control packet is still on
         * the queue of the master. This means that we never successfully
         * transmitted update request. Would end up killing connection
           on slave side. Could ignore it or see if still enqueued. */
        connsm->num_used_chans =
            ble_ll_conn_calc_used_chans(connsm->req_chanmap);
        memcpy(connsm->chanmap, connsm->req_chanmap, BLE_LL_CONN_CHMAP_LEN);

        connsm->csmflags.cfbit.chanmap_update_scheduled = 0;

        ble_ll_ctrl_proc_stop(connsm, BLE_LL_CTRL_PROC_CHAN_MAP_UPD);

        /* XXX: host could have resent channel map command. Need to
           check to make sure we dont have to restart! */
    }

    /* Calculate data channel index of next connection event */
    while (latency > 0) {
        connsm->last_unmapped_chan = connsm->unmapped_chan;
        connsm->data_chan_index = ble_ll_conn_calc_dci(connsm);
        --latency;
    }

    /*
     * If we are trying to terminate connection, check if next wake time is
     * passed the termination timeout. If so, no need to continue with
     * connection as we will time out anyway.
     */
    if (connsm->pending_ctrl_procs & (1 << BLE_LL_CTRL_PROC_TERMINATE)) {
        if ((int32_t)(connsm->terminate_timeout - connsm->anchor_point) <= 0) {
            return -1;
        }
    }

    /*
     * Calculate ce end time. For a slave, we need to add window widening and
     * the transmit window if we still have one.
     */
    itvl = NIMBLE_OPT_LL_CONN_INIT_SLOTS * BLE_LL_SCHED_USECS_PER_SLOT;
    if (connsm->conn_role == BLE_LL_CONN_ROLE_SLAVE) {
        cur_ww = ble_ll_conn_calc_window_widening(connsm);
        max_ww = (connsm->conn_itvl * (BLE_LL_CONN_ITVL_USECS/2)) - BLE_LL_IFS;
        if (cur_ww >= max_ww) {
            return -1;
        }
        connsm->slave_cur_window_widening = cur_ww;
        itvl += cur_ww + connsm->slave_cur_tx_win_usecs;
    } else {
        /* We adjust end time for connection to end of time slot */
        itvl -= XCVR_TX_SCHED_DELAY_USECS;
    }
    connsm->ce_end_time = connsm->anchor_point + cputime_usecs_to_ticks(itvl);

    return 0;
}

/**
 * Called when a connection has been created. This function will
 *  -> Set the connection state to created.
 *  -> Start the connection supervision timer
 *  -> Set the Link Layer state to connection.
 *  -> Send a connection complete event.
 *
 *  See Section 4.5.2 Vol 6 Part B
 *
 *  Context: Link Layer
 *
 * @param connsm
 *
 * @ return 0: connection NOT created. 1: connection created
 */
static int
ble_ll_conn_created(struct ble_ll_conn_sm *connsm, uint32_t endtime)
{
    int rc;
    uint32_t usecs;

    /* Set state to created */
    connsm->conn_state = BLE_LL_CONN_STATE_CREATED;

    /* Set supervision timeout */
    usecs = connsm->conn_itvl * BLE_LL_CONN_ITVL_USECS * 6;
    cputime_timer_relative(&connsm->conn_spvn_timer, usecs);

    /* Clear packet received flag */
    connsm->csmflags.cfbit.pkt_rxd = 0;

    /* Consider time created the last scheduled time */
    connsm->last_scheduled = cputime_get32();

    /*
     * Set first connection event time. If slave the endtime is the receive end
     * time of the connect request. The actual connection starts 1.25 msecs plus
     * the transmit window offset from the end of the connection request.
     */
    rc = 1;
    connsm->last_anchor_point = endtime;
    if (connsm->conn_role == BLE_LL_CONN_ROLE_SLAVE) {
        connsm->slave_cur_tx_win_usecs =
            connsm->tx_win_size * BLE_LL_CONN_TX_WIN_USECS;
        usecs = 1250 + (connsm->tx_win_off * BLE_LL_CONN_TX_WIN_USECS);
        connsm->anchor_point = endtime + cputime_usecs_to_ticks(usecs);
        usecs = connsm->slave_cur_tx_win_usecs +
            (NIMBLE_OPT_LL_CONN_INIT_SLOTS * BLE_LL_SCHED_USECS_PER_SLOT);
        connsm->ce_end_time = connsm->anchor_point +
            cputime_usecs_to_ticks(usecs);
        connsm->slave_cur_window_widening = 0;

        /* Start the scheduler for the first connection event */
        while (ble_ll_sched_slave_new(connsm)) {
            if (ble_ll_conn_next_event(connsm)) {
                STATS_INC(ble_ll_conn_stats, cant_set_sched);
                rc = 0;
                break;
            }
        }
    }

    /* Send connection complete event to inform host of connection */
    if (rc) {
        /*
         * Section 4.5.10 Vol 6 PART B. If the max tx/rx time or octets
         * exceeds the minimum, data length procedure needs to occur
         */
        if ((connsm->max_tx_octets > BLE_LL_CONN_SUPP_BYTES_MIN) ||
            (connsm->max_rx_octets > BLE_LL_CONN_SUPP_BYTES_MIN) ||
            (connsm->max_tx_time > BLE_LL_CONN_SUPP_TIME_MIN) ||
            (connsm->max_rx_time > BLE_LL_CONN_SUPP_TIME_MIN)) {
            /* Start the data length update procedure */
            if (ble_ll_read_supp_features() & BLE_LL_FEAT_DATA_LEN_EXT) {
                ble_ll_ctrl_proc_start(connsm, BLE_LL_CTRL_PROC_DATA_LEN_UPD);
            }
        }
        ble_ll_conn_comp_event_send(connsm, BLE_ERR_SUCCESS);
    }

    return rc;
}

/**
 * Called upon end of connection event
 *
 * Context: Link-layer task
 *
 * @param void *arg Pointer to connection state machine
 *
 */
void
ble_ll_conn_event_end(void *arg)
{
    uint8_t ble_err;
    struct ble_ll_conn_sm *connsm;

    /* Better be a connection state machine! */
    connsm = (struct ble_ll_conn_sm *)arg;
    assert(connsm);

    /* Check if we need to resume scanning */
    ble_ll_scan_chk_resume();

    /* Log event end */
    ble_ll_log(BLE_LL_LOG_ID_CONN_EV_END, 0, connsm->event_cntr,
               connsm->ce_end_time);

    /* If we have transmitted the terminate IND successfully, we are done */
    if ((connsm->csmflags.cfbit.terminate_ind_txd) ||
        (connsm->csmflags.cfbit.terminate_ind_rxd)) {
        if (connsm->csmflags.cfbit.terminate_ind_txd) {
            ble_err = BLE_ERR_CONN_TERM_LOCAL;
        } else {
            /* Make sure the disconnect reason is valid! */
            ble_err = connsm->rxd_disconnect_reason;
            if (ble_err == 0) {
                ble_err = BLE_ERR_REM_USER_CONN_TERM;
            }
        }
        ble_ll_conn_end(connsm, ble_err);
        return;
    }

    /* Remove any connection end events that might be enqueued */
    os_eventq_remove(&g_ble_ll_data.ll_evq, &connsm->conn_ev_end);

    /*
     * If we have received a packet, we can set the current transmit window
     * usecs to 0 since we dont need to listen in the transmit window.
     */
    if (connsm->csmflags.cfbit.pkt_rxd) {
        connsm->slave_cur_tx_win_usecs = 0;
    }

    /* Move to next connection event */
    if (ble_ll_conn_next_event(connsm)) {
        ble_ll_conn_end(connsm, BLE_ERR_CONN_TERM_LOCAL);
        return;
    }

    /* Reset "per connection event" variables */
    connsm->cons_rxd_bad_crc = 0;
    connsm->csmflags.cfbit.pkt_rxd = 0;

    /* See if we need to start any control procedures */
    ble_ll_ctrl_chk_proc_start(connsm);

    /* Set initial schedule callback */
    connsm->conn_sch.sched_cb = ble_ll_conn_event_start_cb;

    /* XXX: I think all this fine for when we do connection updates, but
       we may want to force the first event to be scheduled. Not sure */
    /* Schedule the next connection event */
    while (ble_ll_sched_conn_reschedule(connsm)) {
        if (ble_ll_conn_next_event(connsm)) {
            ble_ll_conn_end(connsm, BLE_ERR_CONN_TERM_LOCAL);
            return;
        }
    }

    /* If we have completed packets, send an event */
    if (connsm->completed_pkts) {
        ble_ll_conn_num_comp_pkts_event_send();
    }
}

/**
 * Update the connection request PDU with the address type and address of
 * advertiser we are going to send connect request to.
 *
 * @param m
 * @param adva
 * @param addr_type
 * @param txoffset      The tx window offset for this connection
 */
static void
ble_ll_conn_req_pdu_update(struct os_mbuf *m, uint8_t *adva, uint8_t addr_type,
                           uint16_t txoffset)
{
    uint8_t pdu_type;
    uint8_t *dptr;
    struct ble_mbuf_hdr *ble_hdr;

    assert(m != NULL);

    ble_hdr = BLE_MBUF_HDR_PTR(m);
    pdu_type = ble_hdr->txinfo.hdr_byte;
    if (addr_type) {
        /* Set random address */
        pdu_type |= BLE_ADV_PDU_HDR_RXADD_MASK;
    } else {
        /* Set public device address */
        pdu_type &= ~BLE_ADV_PDU_HDR_RXADD_MASK;
    }

    /* Set BLE transmit header */
    ble_hdr->txinfo.hdr_byte = pdu_type;

    dptr = m->om_data;
    memcpy(dptr + BLE_DEV_ADDR_LEN, adva, BLE_DEV_ADDR_LEN);
    htole16(dptr + 20, txoffset);
}

/* Returns true if the address matches the connection peer address */
static int
ble_ll_conn_is_peer_adv(uint8_t addr_type, uint8_t *adva)
{
    int rc;
    struct ble_ll_conn_sm *connsm;

    /* XXX: Deal with different types of random addresses here! */
    connsm = g_ble_ll_conn_create_sm;
    if (connsm && (connsm->peer_addr_type == addr_type) &&
        !memcmp(adva, connsm->peer_addr, BLE_DEV_ADDR_LEN)) {
        rc = 1;
    } else {
        rc = 0;
    }

    return rc;
}

/**
 * Called when a connect request transmission is done.
 *
 * Context: ISR
 *
 * @param arg
 */
static void
ble_ll_conn_req_txend(void *arg)
{
    ble_ll_state_set(BLE_LL_STATE_STANDBY);
}

/**
 * Send a connection requestion to an advertiser
 *
 * Context: Interrupt
 *
 * @param addr_type Address type of advertiser
 * @param adva Address of advertiser
 */
static int
ble_ll_conn_request_send(uint8_t addr_type, uint8_t *adva, uint16_t txoffset)
{
    int rc;
    struct os_mbuf *m;

    m = ble_ll_scan_get_pdu();
    ble_ll_conn_req_pdu_update(m, adva, addr_type, txoffset);
    ble_phy_set_txend_cb(ble_ll_conn_req_txend, NULL);
    rc = ble_phy_tx(m, BLE_PHY_TRANSITION_NONE);
    return rc;
}

/**
 * Called when a schedule item overlaps the currently running connection
 * event. This generally should not happen, but if it does we stop the
 * current connection event to let the schedule item run.
 *
 * NOTE: the phy has been disabled as well as the wfr timer before this is
 * called.
 */
void
ble_ll_conn_event_halt(void)
{
    ble_ll_state_set(BLE_LL_STATE_STANDBY);
    if (g_ble_ll_conn_cur_sm) {
        g_ble_ll_conn_cur_sm->csmflags.cfbit.pkt_rxd = 0;
        ble_ll_event_send(&g_ble_ll_conn_cur_sm->conn_ev_end);
        g_ble_ll_conn_cur_sm = NULL;
    }
}

/**
 * Process a received PDU while in the initiating state.
 *
 * Context: Link Layer task.
 *
 * @param pdu_type
 * @param rxbuf
 */
void
ble_ll_init_rx_pkt_in(uint8_t *rxbuf, struct ble_mbuf_hdr *ble_hdr)
{
    uint8_t addr_type;
    uint8_t payload_len;
    uint32_t endtime;
    struct ble_ll_conn_sm *connsm;

    /* Get the connection state machine we are trying to create */
    connsm = g_ble_ll_conn_create_sm;

    /* If we have sent a connect request, we need to enter CONNECTION state */
    if (connsm && CONN_F_CONN_REQ_TXD(connsm)) {
        /* Set address of advertiser to which we are connecting. */
        if (ble_ll_scan_whitelist_enabled()) {
            /*
             * XXX: need to see if the whitelist tells us exactly what peer
             * addr type we should use? Not sure it matters. If whitelisting
             * is not used the peer addr and type already set
             */
            /* Get address type of advertiser */
            if (rxbuf[0] & BLE_ADV_PDU_HDR_TXADD_MASK) {
                addr_type = BLE_HCI_CONN_PEER_ADDR_RANDOM;
            } else {
                addr_type = BLE_HCI_CONN_PEER_ADDR_PUBLIC;
            }

            connsm->peer_addr_type = addr_type;
            memcpy(connsm->peer_addr, rxbuf + BLE_LL_PDU_HDR_LEN,
                   BLE_DEV_ADDR_LEN);
        }

        /* Connection has been created. Stop scanning */
        g_ble_ll_conn_create_sm = NULL;
        ble_ll_scan_sm_stop(0);
        payload_len = rxbuf[1] & BLE_ADV_PDU_HDR_LEN_MASK;;
        endtime = ble_hdr->beg_cputime + BLE_TX_DUR_USECS_M(payload_len);
        ble_ll_conn_created(connsm, endtime);
    } else {
        ble_ll_scan_chk_resume();
    }
}

/**
 * Called when a receive PDU has ended and we are in the initiating state.
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
ble_ll_init_rx_isr_end(struct os_mbuf *rxpdu, uint8_t crcok)
{
    int rc;
    int chk_send_req;
    uint8_t pdu_type;
    uint8_t addr_type;
    uint8_t *adv_addr;
    uint8_t *init_addr;
    uint8_t *rxbuf;
    uint8_t pyld_len;
    uint32_t endtime;
    struct ble_mbuf_hdr *ble_hdr;

    /*
     * We have to restart receive if we cant hand up pdu. We return 0 so that
     * the phy does not get disabled.
     */
    if (!rxpdu) {
        ble_phy_rx();
        return 0;
    }

    rc = -1;
    if (!crcok) {
        goto init_rx_isr_exit;
    }

    /* Only interested in ADV IND or ADV DIRECT IND */
    rxbuf = rxpdu->om_data;
    pdu_type = rxbuf[0] & BLE_ADV_PDU_HDR_TYPE_MASK;
    pyld_len = rxbuf[1] & BLE_ADV_PDU_HDR_LEN_MASK;

    switch (pdu_type) {
    case BLE_ADV_PDU_TYPE_ADV_IND:
        chk_send_req = 1;
        break;
    case BLE_ADV_PDU_TYPE_ADV_DIRECT_IND:
        init_addr = rxbuf + BLE_LL_PDU_HDR_LEN + BLE_DEV_ADDR_LEN;
        addr_type = rxbuf[0] & BLE_ADV_PDU_HDR_RXADD_MASK;
        if (ble_ll_is_our_devaddr(init_addr, addr_type)) {
            chk_send_req = 1;
        } else {
            chk_send_req = 0;
        }
        break;
    default:
        chk_send_req = 0;
        break;
    }

    /* Should we send a connect request? */
    if (chk_send_req) {
        /* Get advertisers address type */
        adv_addr = rxbuf + BLE_LL_PDU_HDR_LEN;
        if (rxbuf[0] & BLE_ADV_PDU_HDR_TXADD_MASK) {
            addr_type = BLE_HCI_CONN_PEER_ADDR_RANDOM;
        } else {
            addr_type = BLE_HCI_CONN_PEER_ADDR_PUBLIC;
        }

        /* Check filter policy */
        ble_hdr = BLE_MBUF_HDR_PTR(rxpdu);
        if (ble_ll_scan_whitelist_enabled()) {
            /* Check if device is on whitelist. If not, leave */
            if (!ble_ll_whitelist_match(adv_addr, addr_type)) {
                return -1;
            }

            /* Set BLE mbuf header flags */
            ble_hdr->rxinfo.flags |= BLE_MBUF_HDR_F_DEVMATCH;
        } else {
            /* XXX: Resolvable? Deal with those */
            /* XXX: HW device matching? If so, implement */
            /* Must match the connection address */
            if (!ble_ll_conn_is_peer_adv(addr_type, adv_addr)) {
                return -1;
            }
        }

        /* Attempt to schedule new connection. Possible that this might fail */
        endtime = ble_hdr->beg_cputime + BLE_TX_DUR_USECS_M(pyld_len);
        if (!ble_ll_sched_master_new(g_ble_ll_conn_create_sm, endtime,
                                     NIMBLE_OPT_LL_CONN_INIT_SLOTS)) {
            /* Setup to transmit the connect request */
            rc = ble_ll_conn_request_send(addr_type, adv_addr,
                                          g_ble_ll_conn_create_sm->tx_win_off);
            if (!rc) {
                CONN_F_CONN_REQ_TXD(g_ble_ll_conn_create_sm) = 1;
                STATS_INC(ble_ll_conn_stats, conn_req_txd);
            }
        } else {
            /* Count # of times we could not set schedule */
            STATS_INC(ble_ll_conn_stats, cant_set_sched);
        }
    }

init_rx_isr_exit:
    if (rc) {
        ble_ll_state_set(BLE_LL_STATE_STANDBY);
    }
    return rc;
}

/**
 * Function called when a timeout has occurred for a connection. There are
 * two types of timeouts: a connection supervision timeout and control
 * procedure timeout.
 *
 * Context: Link Layer task
 *
 * @param connsm
 * @param ble_err
 */
void
ble_ll_conn_timeout(struct ble_ll_conn_sm *connsm, uint8_t ble_err)
{
    int was_current;
    os_sr_t sr;

    was_current = 0;
    OS_ENTER_CRITICAL(sr);
    if (g_ble_ll_conn_cur_sm == connsm) {
        ble_ll_conn_current_sm_over();
        was_current = 1;
    }
    OS_EXIT_CRITICAL(sr);

    /* Check if we need to resume scanning */
    if (was_current) {
        ble_ll_scan_chk_resume();
    }

    ble_ll_conn_end(connsm, ble_err);
}

/**
 * Connection supervision timeout. When called, it means that the connection
 * supervision timeout has been reached. If reached, we end the connection.
 *
 * Context: Link Layer
 *
 * @param arg Pointer to connection state machine.
 */
void
ble_ll_conn_spvn_timeout(void *arg)
{
    ble_ll_conn_timeout((struct ble_ll_conn_sm *)arg, BLE_ERR_CONN_SPVN_TMO);
}

/**
 * Called when a data channel PDU has started that matches the access
 * address of the current connection. Note that the CRC of the PDU has not
 * been checked yet.
 *
 * Context: Interrupt
 */
void
ble_ll_conn_rx_isr_start(void)
{
    struct ble_ll_conn_sm *connsm;

    /*
     * Disable wait for response timer since we receive a response. We dont
     * care if this is the response we were waiting for or not; the code
     * called at receive end will deal with ending the connection event
     * if needed
     */
    ble_ll_wfr_disable();
    connsm = g_ble_ll_conn_cur_sm;
    if (connsm) {
        connsm->csmflags.cfbit.pkt_rxd = 1;
    }
}

/**
 * Called from the Link Layer task when a data PDU has been received
 *
 * Context: Link layer task
 *
 * @param rxpdu Pointer to received pdu
 * @param rxpdu Pointer to ble mbuf header of received pdu
 */
void
ble_ll_conn_rx_data_pdu(struct os_mbuf *rxpdu, struct ble_mbuf_hdr *hdr)
{
    uint8_t hdr_byte;
    uint8_t rxd_sn;
    uint8_t *rxbuf;
    uint16_t acl_len;
    uint16_t acl_hdr;
    uint32_t tmo;
    struct ble_ll_conn_sm *connsm;

    if (BLE_MBUF_HDR_CRC_OK(hdr)) {
        /* XXX: there is a chance that the connection was thrown away and
           re-used before processing packets here. Fix this. */
        /* We better have a connection state machine */
        connsm = ble_ll_conn_find_active_conn(hdr->rxinfo.handle);
        if (connsm) {
            /* Reset the connection supervision timeout */
            cputime_timer_stop(&connsm->conn_spvn_timer);
            tmo = connsm->supervision_tmo * BLE_HCI_CONN_SPVN_TMO_UNITS * 1000;
            cputime_timer_relative(&connsm->conn_spvn_timer, tmo);

            /* Check state machine */
            ble_ll_conn_chk_csm_flags(connsm);

            /* Validate rx data pdu */
            rxbuf = rxpdu->om_data;
            hdr_byte = rxbuf[0];
            acl_len = rxbuf[1];
            acl_hdr = hdr_byte & BLE_LL_DATA_HDR_LLID_MASK;

            /* Check that the LLID is reasonable */
            if ((acl_hdr == 0) ||
                ((acl_hdr == BLE_LL_LLID_DATA_START) && (acl_len == 0))) {
                STATS_INC(ble_ll_conn_stats, rx_bad_llid);
                goto conn_rx_data_pdu_end;
            }

            /* Update RSSI */
            connsm->conn_rssi = hdr->rxinfo.rssi;

            /*
             * If we are a slave, we can only start to use slave latency
             * once we have received a NESN of 1 from the master
             */
            if (connsm->conn_role == BLE_LL_CONN_ROLE_SLAVE) {
                if (hdr_byte & BLE_LL_DATA_HDR_NESN_MASK) {
                    connsm->csmflags.cfbit.allow_slave_latency = 1;
                }
            }

            /*
             * Discard the received PDU if the sequence number is the same
             * as the last received sequence number
             */
            rxd_sn = hdr_byte & BLE_LL_DATA_HDR_SN_MASK;
            if (rxd_sn != connsm->last_rxd_sn) {
                /* Update last rxd sn */
                connsm->last_rxd_sn = rxd_sn;

                /* No need to do anything if empty pdu */
                if ((acl_hdr == BLE_LL_LLID_DATA_FRAG) && (acl_len == 0)) {
                    goto conn_rx_data_pdu_end;
                }

#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
                /*
                 * XXX: should we check to see if we are in a state where we
                 * might expect to get an encrypted PDU?
                 */
                if (BLE_MBUF_HDR_MIC_FAILURE(hdr)) {
                    STATS_INC(ble_ll_conn_stats, mic_failures);
                    ble_ll_conn_timeout(connsm, BLE_ERR_CONN_TERM_MIC);
                    goto conn_rx_data_pdu_end;
                }
#endif

                if (acl_hdr == BLE_LL_LLID_CTRL) {
                    /* Process control frame */
                    STATS_INC(ble_ll_conn_stats, rx_ctrl_pdus);
                    if (ble_ll_ctrl_rx_pdu(connsm, rxpdu)) {
                        STATS_INC(ble_ll_conn_stats, rx_malformed_ctrl_pdus);
                    }
                } else {
                    /* Count # of data frames */
                    STATS_INC(ble_ll_conn_stats, rx_l2cap_pdus);

                    /* NOTE: there should be at least two bytes available */
                    assert(OS_MBUF_LEADINGSPACE(rxpdu) >= 2);
                    os_mbuf_prepend(rxpdu, 2);
                    rxbuf = rxpdu->om_data;

                    acl_hdr = (acl_hdr << 12) | connsm->conn_handle;
                    htole16(rxbuf, acl_hdr);
                    htole16(rxbuf + 2, acl_len);
                    ble_hs_rx_data(rxpdu);
                }

                /* NOTE: we dont free the mbuf since we handed it off! */
                return;
            } else {
                STATS_INC(ble_ll_conn_stats, data_pdu_rx_dup);
            }
        } else {
            STATS_INC(ble_ll_conn_stats, no_conn_sm);
        }
    }

    /* Free buffer */
conn_rx_data_pdu_end:
    os_mbuf_free_chain(rxpdu);
}

/**
 * Called when a packet has been received while in the connection state.
 *
 * Context: Interrupt
 *
 * @param rxpdu
 * @param crcok
 *
 * @return int
 *       < 0: Disable the phy after reception.
 *      == 0: Success. Do not disable the PHY.
 *       > 0: Do not disable PHY as that has already been done.
 */
int
ble_ll_conn_rx_isr_end(struct os_mbuf *rxpdu, uint32_t aa)
{
    int rc;
    int is_ctrl;
    uint8_t hdr_byte;
    uint8_t hdr_sn;
    uint8_t hdr_nesn;
    uint8_t conn_sn;
    uint8_t conn_nesn;
    uint8_t reply;
    uint8_t rem_bytes;
    uint8_t opcode;
    uint8_t rx_pyld_len;
    uint32_t endtime;
    struct os_mbuf *txpdu;
    struct ble_ll_conn_sm *connsm;
    struct ble_mbuf_hdr *rxhdr;
    struct ble_mbuf_hdr *txhdr;

    /*
     * We should have a current connection state machine. If we dont, we just
     * hand the packet to the higher layer to count it.
     */
    rc = -1;
    connsm = g_ble_ll_conn_cur_sm;
    if (!connsm) {
        STATS_INC(ble_ll_conn_stats, rx_data_pdu_no_conn);
        goto conn_exit;
    }

    /* Double check access address. Better match connection state machine! */
    if (aa != connsm->access_addr) {
        STATS_INC(ble_ll_conn_stats, rx_data_pdu_bad_aa);
        goto conn_exit;
    }

    /* Set the handle in the ble mbuf header */
    rxhdr = BLE_MBUF_HDR_PTR(rxpdu);
    rxhdr->rxinfo.handle = connsm->conn_handle;
    hdr_byte = rxpdu->om_data[0];
    rx_pyld_len = rxpdu->om_data[1];

    /*
     * Check the packet CRC. A connection event can continue even if the
     * received PDU does not pass the CRC check. If we receive two consecutive
     * CRC errors we end the conection event.
     */
    if (!BLE_MBUF_HDR_CRC_OK(rxhdr)) {
        /*
         * Increment # of consecutively received CRC errors. If more than
         * one we will end the connection event.
         */
        ++connsm->cons_rxd_bad_crc;
        if (connsm->cons_rxd_bad_crc >= 2) {
            reply = 0;
        } else {
            if (connsm->conn_role == BLE_LL_CONN_ROLE_MASTER) {
                reply = CONN_F_LAST_TXD_MD(connsm);
            } else {
                /* A slave always responds with a packet */
                reply = 1;
            }
        }
    } else {
        /* Reset consecutively received bad crcs (since this one was good!) */
        connsm->cons_rxd_bad_crc = 0;

        /* Check for valid LLID before proceeding. */
        if ((hdr_byte & BLE_LL_DATA_HDR_LLID_MASK) == 0) {
            /*
             * XXX: for now, just exit since we dont trust the length
             * and may erroneously adjust anchor. Once we fix the anchor
             * point issue we need to decide what to do on bad llid. Note
             * that an error stat gets counted at the LL
             */
            reply = 0;
            goto conn_exit;
        }

        /* Set last received header byte */
        connsm->last_rxd_hdr_byte = hdr_byte;

        /*
         * If SN bit from header does not match NESN in connection, this is
         * a resent PDU and should be ignored.
         */
        hdr_sn = hdr_byte & BLE_LL_DATA_HDR_SN_MASK;
        conn_nesn = connsm->next_exp_seqnum;
        if ((hdr_sn && conn_nesn) || (!hdr_sn && !conn_nesn)) {
            connsm->next_exp_seqnum ^= 1;
#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
            if (CONN_F_ENCRYPTED(connsm) && !ble_ll_conn_is_empty_pdu(rxpdu)) {
                ++connsm->enc_data.rx_pkt_cntr;
            }
#endif
        }

#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
        ble_ll_log(BLE_LL_LOG_ID_CONN_RX,
                   hdr_byte,
                   (uint16_t)connsm->tx_seqnum << 8 | conn_nesn,
                   connsm->enc_data.rx_pkt_cntr);
#else
        ble_ll_log(BLE_LL_LOG_ID_CONN_RX,
                   hdr_byte,
                   (uint16_t)connsm->tx_seqnum << 8 | conn_nesn, 0);
#endif

        /*
         * Check NESN bit from header. If same as tx seq num, the transmission
         * is acknowledged. Otherwise we need to resend this PDU.
         */
        if (CONN_F_EMPTY_PDU_TXD(connsm) || connsm->cur_tx_pdu) {
            hdr_nesn = hdr_byte & BLE_LL_DATA_HDR_NESN_MASK;
            conn_sn = connsm->tx_seqnum;
            if ((hdr_nesn && conn_sn) || (!hdr_nesn && !conn_sn)) {
                /* We did not get an ACK. Must retry the PDU */
                STATS_INC(ble_ll_conn_stats, data_pdu_txf);
            } else {
                /* Transmit success */
                connsm->tx_seqnum ^= 1;
                STATS_INC(ble_ll_conn_stats, data_pdu_txg);

                /* If we transmitted the empty pdu, clear flag */
                if (CONN_F_EMPTY_PDU_TXD(connsm)) {
                    CONN_F_EMPTY_PDU_TXD(connsm) = 0;
                    goto chk_rx_terminate_ind;
                }

                /*
                 * Determine if we should remove packet from queue or if there
                 * are more fragments to send.
                 */
                txpdu = connsm->cur_tx_pdu;
                if (txpdu) {
#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
                    if (connsm->enc_data.tx_encrypted) {
                        ++connsm->enc_data.tx_pkt_cntr;
                    }
#endif
                    txhdr = BLE_MBUF_HDR_PTR(txpdu);
                    if ((txhdr->txinfo.hdr_byte & BLE_LL_DATA_HDR_LLID_MASK)
                        == BLE_LL_LLID_CTRL) {
                        connsm->cur_tx_pdu = NULL;
                        /* Note: the mbuf is freed by this call */
                        rc = ble_ll_ctrl_tx_done(txpdu, connsm);
                        if (rc) {
                            /* Means we transmitted a TERMINATE_IND */
                            goto conn_rx_pdu_end;
                        } else {
                            goto chk_rx_terminate_ind;
                        }
                    }

                    /* Increment offset based on number of bytes sent */
                    txhdr->txinfo.offset += txhdr->txinfo.pyld_len;
                    if (txhdr->txinfo.offset >= OS_MBUF_PKTLEN(txpdu)) {
                        /* If l2cap pdu, increment # of completed packets */
                        if (txhdr->txinfo.pyld_len != 0) {
#if (BLETEST_THROUGHPUT_TEST == 1)
                            bletest_completed_pkt(connsm->conn_handle);
#endif
                            ++connsm->completed_pkts;
                        }
                        os_mbuf_free_chain(txpdu);
                        connsm->cur_tx_pdu = NULL;
                    } else {
                        rem_bytes = OS_MBUF_PKTLEN(txpdu) - txhdr->txinfo.offset;
                        if (rem_bytes > connsm->eff_max_tx_octets) {
                            txhdr->txinfo.pyld_len = connsm->eff_max_tx_octets;
                        } else {
                            txhdr->txinfo.pyld_len = rem_bytes;
                        }
                    }
                }
            }
        }

        /* Should we continue connection event? */
        /* If this is a TERMINATE_IND, we have to reply */
chk_rx_terminate_ind:
        is_ctrl = 0;
        if ((hdr_byte & BLE_LL_DATA_HDR_LLID_MASK) == BLE_LL_LLID_CTRL) {
            is_ctrl = 1;
            opcode = rxpdu->om_data[2];
        }

        /* If we received a terminate IND, we must set some flags */
        if (is_ctrl && (opcode == BLE_LL_CTRL_TERMINATE_IND)) {
            connsm->csmflags.cfbit.terminate_ind_rxd = 1;
            connsm->rxd_disconnect_reason = rxpdu->om_data[3];
            reply = 1;
        } else if (connsm->conn_role == BLE_LL_CONN_ROLE_MASTER) {
            reply = CONN_F_LAST_TXD_MD(connsm) || (hdr_byte & BLE_LL_DATA_HDR_MD_MASK);
        } else {
            /* A slave always replies */
            reply = 1;
#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
            if (is_ctrl && (opcode == BLE_LL_CTRL_PAUSE_ENC_RSP)) {
                connsm->enc_data.enc_state = CONN_ENC_S_UNENCRYPTED;
            }
#endif
        }
    }

    /* If reply flag set, send data pdu and continue connection event */
    rc = -1;
    if (rx_pyld_len && CONN_F_ENCRYPTED(connsm)) {
        rx_pyld_len += BLE_LL_DATA_MIC_LEN;
    }
    endtime = rxhdr->beg_cputime + BLE_TX_DUR_USECS_M(rx_pyld_len);
    if (reply && ble_ll_conn_can_send_next_pdu(connsm, endtime)) {
        rc = ble_ll_conn_tx_data_pdu(connsm);
    }

conn_rx_pdu_end:
    /* Set anchor point (and last) if 1st received frame in connection event */
    if (connsm->csmflags.cfbit.slave_set_last_anchor) {
        connsm->csmflags.cfbit.slave_set_last_anchor = 0;
        connsm->last_anchor_point = rxhdr->beg_cputime;
        connsm->anchor_point = connsm->last_anchor_point;
    }

    /* Send link layer a connection end event if over */
conn_exit:
    if (rc) {
        ble_ll_conn_current_sm_over();
        if (connsm) {
            ble_ll_event_send(&connsm->conn_ev_end);
        }
    }

    return rc;
}

/**
 * Called to enqueue a packet on the transmit queue of a connection. Should
 * only be called by the controller.
 *
 * Context: Link Layer
 *
 *
 * @param connsm
 * @param om
 */
void
ble_ll_conn_enqueue_pkt(struct ble_ll_conn_sm *connsm, struct os_mbuf *om,
                        uint8_t hdr_byte, uint8_t length)
{
    os_sr_t sr;
    struct os_mbuf_pkthdr *pkthdr;
    struct ble_mbuf_hdr *ble_hdr;
    int lifo;

    /* Initialize the mbuf */
    ble_ll_mbuf_init(om, length, hdr_byte);

    /*
     * We need to set the initial payload length if the total length of the
     * PDU exceeds the maximum allowed for the connection for any single tx.
     */
    if (length > connsm->eff_max_tx_octets) {
        ble_hdr = BLE_MBUF_HDR_PTR(om);
        ble_hdr->txinfo.pyld_len = connsm->eff_max_tx_octets;
    }

    lifo = 0;
#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
    if (connsm->enc_data.enc_state > CONN_ENC_S_ENCRYPTED) {
        uint8_t llid;

        /*
         * If this is one of the following types we need to insert it at
         * head of queue.
         */
        ble_hdr = BLE_MBUF_HDR_PTR(om);
        llid = ble_hdr->txinfo.hdr_byte & BLE_LL_DATA_HDR_LLID_MASK;
        if (llid == BLE_LL_LLID_CTRL) {
            switch (om->om_data[0]) {
            case BLE_LL_CTRL_TERMINATE_IND:
            case BLE_LL_CTRL_REJECT_IND:
            case BLE_LL_CTRL_REJECT_IND_EXT:
            case BLE_LL_CTRL_START_ENC_REQ:
            case BLE_LL_CTRL_START_ENC_RSP:
                lifo = 1;
                break;
            case BLE_LL_CTRL_PAUSE_ENC_RSP:
                if (connsm->conn_role == BLE_LL_CONN_ROLE_MASTER) {
                    lifo = 1;
                }
                break;
            default:
                break;
            }
        }
    }
#endif

    /* Add to transmit queue for the connection */
    pkthdr = OS_MBUF_PKTHDR(om);
    OS_ENTER_CRITICAL(sr);
    if (lifo) {
        STAILQ_INSERT_HEAD(&connsm->conn_txq, pkthdr, omp_next);
    } else {
        STAILQ_INSERT_TAIL(&connsm->conn_txq, pkthdr, omp_next);
    }
    OS_EXIT_CRITICAL(sr);
}

/**
 * Data packet from host.
 *
 * Context: Link Layer task
 *
 * @param om
 * @param handle
 * @param length
 *
 * @return int
 */
void
ble_ll_conn_tx_pkt_in(struct os_mbuf *om, uint16_t handle, uint16_t length)
{
    uint8_t hdr_byte;
    uint16_t conn_handle;
    uint16_t pb;
    struct ble_ll_conn_sm *connsm;

    /* See if we have an active matching connection handle */
    conn_handle = handle & 0x0FFF;
    connsm = ble_ll_conn_find_active_conn(conn_handle);
    if (connsm) {
        /* Construct LL header in buffer (NOTE: pb already checked) */
        pb = handle & 0x3000;
        if (pb == 0) {
            hdr_byte = BLE_LL_LLID_DATA_START;
        } else {
            hdr_byte = BLE_LL_LLID_DATA_FRAG;
        }

        /* Add to total l2cap pdus enqueue */
        STATS_INC(ble_ll_conn_stats, l2cap_enqueued);

        /* Clear flags field in BLE header */
        ble_ll_conn_enqueue_pkt(connsm, om, hdr_byte, length);
    } else {
        /* No connection found! */
        STATS_INC(ble_ll_conn_stats, handle_not_found);
        os_mbuf_free_chain(om);
    }
}

/**
 * Called to set the global channel mask that we use for all connections.
 *
 * @param num_used_chans
 * @param chanmap
 */
void
ble_ll_conn_set_global_chanmap(uint8_t num_used_chans, uint8_t *chanmap)
{
    struct ble_ll_conn_sm *connsm;
    struct ble_ll_conn_global_params *conn_params;

    /* Do nothing if same channel map */
    conn_params = &g_ble_ll_conn_params;
    if (!memcmp(conn_params->master_chan_map, chanmap, BLE_LL_CONN_CHMAP_LEN)) {
        return;
    }

    /* Change channel map and cause channel map update procedure to start */
    conn_params->num_used_chans = num_used_chans;
    memcpy(conn_params->master_chan_map, chanmap, BLE_LL_CONN_CHMAP_LEN);

    /* Perform channel map update */
    SLIST_FOREACH(connsm, &g_ble_ll_conn_active_list, act_sle) {
        if (connsm->conn_role == BLE_LL_CONN_ROLE_MASTER) {
            ble_ll_ctrl_proc_start(connsm, BLE_LL_CTRL_PROC_CHAN_MAP_UPD);
        }
    }
}

/**
 * Called when a device has received a connect request while advertising and
 * the connect request has passed the advertising filter policy and is for
 * us. This will start a connection in the slave role assuming that we dont
 * already have a connection with this device and that the connect request
 * parameters are valid.
 *
 * Context: Link Layer
 *
 * @param rxbuf Pointer to received Connect Request PDU
 * @param conn_req_end receive end time of connect request
 *
 * @return 0: connection not started; 1 connecton started
 */
int
ble_ll_conn_slave_start(uint8_t *rxbuf, uint32_t conn_req_end)
{
    int rc;
    uint32_t temp;
    uint32_t crcinit;
    uint8_t *inita;
    uint8_t *dptr;
    uint8_t addr_type;
    struct ble_ll_conn_sm *connsm;

    /* Ignore the connection request if we are already connected*/
    inita = rxbuf + BLE_LL_PDU_HDR_LEN;
    SLIST_FOREACH(connsm, &g_ble_ll_conn_active_list, act_sle) {
        if (!memcmp(&connsm->peer_addr, inita, BLE_DEV_ADDR_LEN)) {
            if (rxbuf[0] & BLE_ADV_PDU_HDR_TXADD_MASK) {
                addr_type = BLE_HCI_CONN_PEER_ADDR_RANDOM;
            } else {
                addr_type = BLE_HCI_CONN_PEER_ADDR_PUBLIC;
            }
            if (connsm->peer_addr_type == addr_type) {
                return 0;
            }
        }
    }

    /* Allocate a connection. If none available, dont do anything */
    connsm = ble_ll_conn_sm_get();
    if (connsm == NULL) {
        return 0;
    }

    /* Set the pointer at the start of the connection data */
    dptr = rxbuf + BLE_LL_CONN_REQ_ADVA_OFF + BLE_DEV_ADDR_LEN;

    /* Set connection state machine information */
    connsm->access_addr = le32toh(dptr);
    crcinit = dptr[6];
    crcinit = (crcinit << 8) | dptr[5];
    crcinit = (crcinit << 8) | dptr[4];
    connsm->crcinit = crcinit;
    connsm->tx_win_size = dptr[7];
    connsm->tx_win_off = le16toh(dptr + 8);
    connsm->conn_itvl = le16toh(dptr + 10);
    connsm->slave_latency = le16toh(dptr + 12);
    connsm->supervision_tmo = le16toh(dptr + 14);
    memcpy(&connsm->chanmap, dptr + 16, BLE_LL_CONN_CHMAP_LEN);
    connsm->hop_inc = dptr[21] & 0x1F;
    connsm->master_sca = dptr[21] >> 5;

    /* Error check parameters */
    if ((connsm->tx_win_off > connsm->conn_itvl) ||
        (connsm->conn_itvl < BLE_HCI_CONN_ITVL_MIN) ||
        (connsm->conn_itvl > BLE_HCI_CONN_ITVL_MAX) ||
        (connsm->tx_win_size < BLE_LL_CONN_TX_WIN_MIN) ||
        (connsm->slave_latency > BLE_LL_CONN_SLAVE_LATENCY_MAX)) {
        goto err_slave_start;
    }

    /* Slave latency cannot cause a supervision timeout */
    temp = (connsm->slave_latency + 1) * (connsm->conn_itvl * 2) *
            BLE_LL_CONN_ITVL_USECS;
    if ((connsm->supervision_tmo * 10000) <= temp ) {
        goto err_slave_start;
    }

    /*
     * The transmit window must be less than or equal to the lesser of 10
     * msecs or the connection interval minus 1.25 msecs.
     */
    temp = connsm->conn_itvl - 1;
    if (temp > 8) {
        temp = 8;
    }
    if (connsm->tx_win_size > temp) {
        goto err_slave_start;
    }

    /* XXX: might want to set this differently based on adv. filter policy! */
    /* Set the address of device that we are connecting with */
    memcpy(&connsm->peer_addr, rxbuf + BLE_LL_PDU_HDR_LEN, BLE_DEV_ADDR_LEN);
    if (rxbuf[0] & BLE_ADV_PDU_HDR_TXADD_MASK) {
        connsm->peer_addr_type = BLE_HCI_CONN_PEER_ADDR_RANDOM;
    } else {
        connsm->peer_addr_type = BLE_HCI_CONN_PEER_ADDR_PUBLIC;
    }

    /* Calculate number of used channels; make sure it meets min requirement */
    connsm->num_used_chans = ble_ll_conn_calc_used_chans(connsm->chanmap);
    if (connsm->num_used_chans < 2) {
        goto err_slave_start;
    }

    /* Start the connection state machine */
    connsm->conn_role = BLE_LL_CONN_ROLE_SLAVE;
    ble_ll_conn_sm_new(connsm);

    /* Set initial schedule callback */
    connsm->conn_sch.sched_cb = ble_ll_conn_event_start_cb;

    rc = ble_ll_conn_created(connsm, conn_req_end);
    if (!rc) {
        SLIST_REMOVE(&g_ble_ll_conn_active_list, connsm, ble_ll_conn_sm, act_sle);
        STAILQ_INSERT_TAIL(&g_ble_ll_conn_free_list, connsm, free_stqe);
    }
    return rc;

err_slave_start:
    STAILQ_INSERT_TAIL(&g_ble_ll_conn_free_list, connsm, free_stqe);
    STATS_INC(ble_ll_conn_stats, slave_rxd_bad_conn_req_params);
    return 0;
}

/**
 * Called to reset the connection module. When this function is called the
 * scheduler has been stopped and the phy has been disabled. The LL should
 * be in the standby state.
 *
 * Context: Link Layer task
 */
void
ble_ll_conn_module_reset(void)
{
    uint8_t max_phy_pyld;
    uint16_t maxbytes;
    struct ble_ll_conn_sm *connsm;
    struct ble_ll_conn_global_params *conn_params;

    /* Kill the current one first (if one is running) */
    if (g_ble_ll_conn_cur_sm) {
        connsm = g_ble_ll_conn_cur_sm;
        g_ble_ll_conn_cur_sm = NULL;
        ble_ll_conn_end(connsm, BLE_ERR_SUCCESS);
    }

    /* Now go through and end all the connections */
    while (1) {
        connsm = SLIST_FIRST(&g_ble_ll_conn_active_list);
        if (!connsm) {
            break;
        }
        ble_ll_conn_end(connsm, BLE_ERR_SUCCESS);
    }

    /* Get the maximum supported PHY PDU size from the PHY */

    /* Configure the global LL parameters */
    conn_params = &g_ble_ll_conn_params;
    max_phy_pyld = ble_phy_max_data_pdu_pyld();

    maxbytes = min(NIMBLE_OPT_LL_SUPP_MAX_RX_BYTES, max_phy_pyld);
    conn_params->supp_max_rx_octets = maxbytes;
    conn_params->supp_max_rx_time =
        BLE_TX_DUR_USECS_M(maxbytes + BLE_LL_DATA_MIC_LEN);

    maxbytes = min(NIMBLE_OPT_LL_SUPP_MAX_TX_BYTES, max_phy_pyld);
    conn_params->supp_max_tx_octets = maxbytes;
    conn_params->supp_max_tx_time =
        BLE_TX_DUR_USECS_M(maxbytes + BLE_LL_DATA_MIC_LEN);

    maxbytes = min(NIMBLE_OPT_LL_CONN_INIT_MAX_TX_BYTES, max_phy_pyld);
    conn_params->conn_init_max_tx_octets = maxbytes;
    conn_params->conn_init_max_tx_time =
        BLE_TX_DUR_USECS_M(maxbytes + BLE_LL_DATA_MIC_LEN);

    conn_params->sugg_tx_octets = BLE_LL_CONN_SUPP_BYTES_MIN;
    conn_params->sugg_tx_time = BLE_LL_CONN_SUPP_TIME_MIN;

    /* Mask in all channels by default */
    conn_params->num_used_chans = BLE_PHY_NUM_DATA_CHANS;
    memset(conn_params->master_chan_map, 0xff, BLE_LL_CONN_CHMAP_LEN - 1);
    conn_params->master_chan_map[4] = 0x1f;

    /* Reset statistics */
    memset((uint8_t *)&ble_ll_conn_stats + sizeof(struct stats_hdr), 0,
           sizeof(struct stats_ble_ll_conn_stats) - sizeof(struct stats_hdr));
}

/* Initialize the connection module */
void
ble_ll_conn_module_init(void)
{
    int rc;
    uint16_t i;
    struct ble_ll_conn_sm *connsm;

    /* Initialize list of active conections */
    SLIST_INIT(&g_ble_ll_conn_active_list);
    STAILQ_INIT(&g_ble_ll_conn_free_list);

    /*
     * Take all the connections off the free memory pool and add them to
     * the free connection list, assigning handles in linear order. Note:
     * the specification allows a handle of zero; we just avoid using it.
     */
    connsm = &g_ble_ll_conn_sm[0];
    for (i = 0; i < NIMBLE_OPT_MAX_CONNECTIONS; ++i) {

        memset(connsm, 0, sizeof(struct ble_ll_conn_sm));
        connsm->conn_handle = i + 1;
        STAILQ_INSERT_TAIL(&g_ble_ll_conn_free_list, connsm, free_stqe);

        /* Initialize fixed schedule elements */
        connsm->conn_sch.sched_type = BLE_LL_SCHED_TYPE_CONN;
        connsm->conn_sch.cb_arg = connsm;
        ++connsm;
    }

    /* Register connection statistics */
    rc = stats_init_and_reg(STATS_HDR(ble_ll_conn_stats),
                            STATS_SIZE_INIT_PARMS(ble_ll_conn_stats, STATS_SIZE_32),
                            STATS_NAME_INIT_PARMS(ble_ll_conn_stats),
                            "ble_ll_conn");
    assert(rc == 0);

    /* Call reset to finish reset of initialization */
    ble_ll_conn_module_reset();
}
