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
#include "stats/stats.h"
#include "bsp/bsp.h"
#include "nimble/ble.h"
#include "nimble/nimble_opt.h"
#include "nimble/hci_common.h"
#include "controller/ble_hw.h"
#include "controller/ble_phy.h"
#include "controller/ble_ll.h"
#include "controller/ble_ll_adv.h"
#include "controller/ble_ll_sched.h"
#include "controller/ble_ll_scan.h"
#include "controller/ble_ll_hci.h"
#include "ble_ll_conn_priv.h"
#include "hal/hal_cputime.h"

/* XXX:
 *
 * 1) use the sanity task!
 * 2) Need to figure out what to do with packets that we hand up that did
 * not pass the filter policy for the given state. Currently I count all
 * packets I think. Need to figure out what to do with this.
 * 3) For the features defined, we need to conditionally compile code.
 * 4) Should look into always disabled the wfr interrupt if we receive the
 * start of a frame. Need to look at the various states to see if this is the
 * right thing to do.
 */

/* Supported states */
#define BLE_LL_S_NCA                    (0x00000000001)
#define BLE_LL_S_SA                     (0x00000000002)
#define BLE_LL_S_CA                     (0x00000000004)
#define BLE_LL_S_HDCA                   (0x00000000008)
#define BLE_LL_S_PS                     (0x00000000010)
#define BLE_LL_S_AS                     (0x00000000020)
#define BLE_LL_S_INIT                   (0x00000000040)
#define BLE_LL_S_SLAVE                  (0x00000000080)
#define BLE_LL_S_NCA_PS                 (0x00000000100)
#define BLE_LL_S_SA_PS                  (0x00000000200)
#define BLE_LL_S_CA_PS                  (0x00000000400)
#define BLE_LL_S_HDCA_PS                (0x00000000800)
#define BLE_LL_S_NCA_AS                 (0x00000001000)
#define BLE_LL_S_SA_AS                  (0x00000002000)
#define BLE_LL_S_CA_AS                  (0x00000004000)
#define BLE_LL_S_HDCA_AS                (0x00000008000)
#define BLE_LL_S_NCA_INIT               (0x00000010000)
#define BLE_LL_S_SA_INIT                (0x00000020000)
#define BLE_LL_S_NCA_MASTER             (0x00000040000)
#define BLE_LL_S_SA_MASTER              (0x00000080000)
#define BLE_LL_S_NCA_SLAVE              (0x00000100000)
#define BLE_LL_S_SA_SLAVE               (0x00000200000)
#define BLE_LL_S_PS_INIT                (0x00000400000)
#define BLE_LL_S_AS_INIT                (0x00000800000)
#define BLE_LL_S_PS_MASTER              (0x00001000000)
#define BLE_LL_S_AS_MASTER              (0x00002000000)
#define BLE_LL_S_PS_SLAVE               (0x00004000000)
#define BLE_LL_S_AS_SLAVE               (0x00008000000)
#define BLE_LL_S_INIT_MASTER            (0x00010000000)
#define BLE_LL_S_LDCA                   (0x00020000000)
#define BLE_LL_S_LDCA_PS                (0x00040000000)
#define BLE_LL_S_LDCA_AS                (0x00080000000)
#define BLE_LL_S_CA_INIT                (0x00100000000)
#define BLE_LL_S_HDCA_INIT              (0x00200000000)
#define BLE_LL_S_LDCA_INIT              (0x00400000000)
#define BLE_LL_S_CA_MASTER              (0x00800000000)
#define BLE_LL_S_HDCA_MASTER            (0x01000000000)
#define BLE_LL_S_LDCA_MASTER            (0x02000000000)
#define BLE_LL_S_CA_SLAVE               (0x04000000000)
#define BLE_LL_S_HDCA_SLAVE             (0x08000000000)
#define BLE_LL_S_LDCA_SLAVE             (0x10000000000)
#define BLE_LL_S_INIT_SLAVE             (0x20000000000)

#define BLE_LL_SUPPORTED_STATES             \
(                                           \
    BLE_LL_S_NCA                    |       \
    BLE_LL_S_SA                     |       \
    BLE_LL_S_CA                     |       \
    BLE_LL_S_HDCA                   |       \
    BLE_LL_S_PS                     |       \
    BLE_LL_S_AS                     |       \
    BLE_LL_S_INIT                   |       \
    BLE_LL_S_SLAVE                  |       \
    BLE_LL_S_NCA_PS                 |       \
    BLE_LL_S_SA_PS                  |       \
    BLE_LL_S_CA_PS                  |       \
    BLE_LL_S_HDCA_PS                |       \
    BLE_LL_S_NCA_AS                 |       \
    BLE_LL_S_SA_AS                  |       \
    BLE_LL_S_CA_AS                  |       \
    BLE_LL_S_HDCA_AS                |       \
    BLE_LL_S_NCA_INIT               |       \
    BLE_LL_S_SA_INIT                |       \
    BLE_LL_S_NCA_MASTER             |       \
    BLE_LL_S_SA_MASTER              |       \
    BLE_LL_S_NCA_SLAVE              |       \
    BLE_LL_S_SA_SLAVE               |       \
    BLE_LL_S_PS_INIT                |       \
    BLE_LL_S_AS_INIT                |       \
    BLE_LL_S_PS_MASTER              |       \
    BLE_LL_S_AS_MASTER              |       \
    BLE_LL_S_PS_SLAVE               |       \
    BLE_LL_S_AS_SLAVE               |       \
    BLE_LL_S_INIT_MASTER            |       \
    BLE_LL_S_LDCA                   |       \
    BLE_LL_S_LDCA_PS                |       \
    BLE_LL_S_LDCA_AS                |       \
    BLE_LL_S_CA_INIT                |       \
    BLE_LL_S_HDCA_INIT              |       \
    BLE_LL_S_LDCA_INIT              |       \
    BLE_LL_S_CA_MASTER              |       \
    BLE_LL_S_HDCA_MASTER            |       \
    BLE_LL_S_LDCA_MASTER            |       \
    BLE_LL_S_CA_SLAVE               |       \
    BLE_LL_S_HDCA_SLAVE             |       \
    BLE_LL_S_LDCA_SLAVE             |       \
    BLE_LL_S_INIT_SLAVE)

/* The global BLE LL data object */
struct ble_ll_obj g_ble_ll_data;

/* Global link layer statistics */
STATS_SECT_DECL(ble_ll_stats) ble_ll_stats;
STATS_NAME_START(ble_ll_stats)
    STATS_NAME(ble_ll_stats, hci_cmds)
    STATS_NAME(ble_ll_stats, hci_cmd_errs)
    STATS_NAME(ble_ll_stats, hci_events_sent)
    STATS_NAME(ble_ll_stats, bad_ll_state)
    STATS_NAME(ble_ll_stats, bad_acl_hdr)
    STATS_NAME(ble_ll_stats, rx_adv_pdu_crc_ok)
    STATS_NAME(ble_ll_stats, rx_adv_pdu_crc_err)
    STATS_NAME(ble_ll_stats, rx_adv_bytes_crc_ok)
    STATS_NAME(ble_ll_stats, rx_adv_bytes_crc_err)
    STATS_NAME(ble_ll_stats, rx_data_pdu_crc_ok)
    STATS_NAME(ble_ll_stats, rx_data_pdu_crc_err)
    STATS_NAME(ble_ll_stats, rx_data_bytes_crc_ok)
    STATS_NAME(ble_ll_stats, rx_data_bytes_crc_err)
    STATS_NAME(ble_ll_stats, rx_adv_malformed_pkts)
    STATS_NAME(ble_ll_stats, rx_adv_ind)
    STATS_NAME(ble_ll_stats, rx_adv_direct_ind)
    STATS_NAME(ble_ll_stats, rx_adv_nonconn_ind)
    STATS_NAME(ble_ll_stats, rx_scan_reqs)
    STATS_NAME(ble_ll_stats, rx_scan_rsps)
    STATS_NAME(ble_ll_stats, rx_connect_reqs)
    STATS_NAME(ble_ll_stats, rx_scan_ind)
    STATS_NAME(ble_ll_stats, adv_txg)
    STATS_NAME(ble_ll_stats, adv_late_starts)
    STATS_NAME(ble_ll_stats, sched_state_conn_errs)
    STATS_NAME(ble_ll_stats, sched_state_adv_errs)
    STATS_NAME(ble_ll_stats, scan_starts)
    STATS_NAME(ble_ll_stats, scan_stops)
    STATS_NAME(ble_ll_stats, scan_req_txf)
    STATS_NAME(ble_ll_stats, scan_req_txg)
    STATS_NAME(ble_ll_stats, scan_rsp_txg)
STATS_NAME_END(ble_ll_stats)

/* The BLE LL task data structure */
#define BLE_LL_STACK_SIZE   (80)
struct os_task g_ble_ll_task;
os_stack_t g_ble_ll_stack[BLE_LL_STACK_SIZE];

/* XXX: temporary logging until we transition to real logging */
#ifdef BLE_LL_LOG
struct ble_ll_log
{
    uint8_t log_id;
    uint8_t log_a8;
    uint16_t log_a16;
    uint32_t log_a32;
    uint32_t cputime;

};

#define BLE_LL_LOG_LEN  (256)

static bssnz_t struct ble_ll_log g_ble_ll_log[BLE_LL_LOG_LEN];
static uint8_t g_ble_ll_log_index;

void
ble_ll_log(uint8_t id, uint8_t arg8, uint16_t arg16, uint32_t arg32)
{
    os_sr_t sr;
    struct ble_ll_log *le;

    OS_ENTER_CRITICAL(sr);
    le = &g_ble_ll_log[g_ble_ll_log_index];
    le->cputime = cputime_get32();
    le->log_id = id;
    le->log_a8 = arg8;
    le->log_a16 = arg16;
    le->log_a32 = arg32;
    ++g_ble_ll_log_index;
    if (g_ble_ll_log_index == BLE_LL_LOG_LEN) {
        g_ble_ll_log_index = 0;
    }
    OS_EXIT_CRITICAL(sr);
}
#endif

/**
 * Counts the number of advertising PDU's received, by type. For advertising
 * PDU's that contain a destination address, we still count these packets even
 * if they are not for us.
 *
 * @param pdu_type
 */
static void
ble_ll_count_rx_adv_pdus(uint8_t pdu_type)
{
    /* Count received packet types  */
    switch (pdu_type) {
    case BLE_ADV_PDU_TYPE_ADV_IND:
        STATS_INC(ble_ll_stats, rx_adv_ind);
        break;
    case BLE_ADV_PDU_TYPE_ADV_DIRECT_IND:
        STATS_INC(ble_ll_stats, rx_adv_direct_ind);
        break;
    case BLE_ADV_PDU_TYPE_ADV_NONCONN_IND:
        STATS_INC(ble_ll_stats, rx_adv_nonconn_ind);
        break;
    case BLE_ADV_PDU_TYPE_SCAN_REQ:
        STATS_INC(ble_ll_stats, rx_scan_reqs);
        break;
    case BLE_ADV_PDU_TYPE_SCAN_RSP:
        STATS_INC(ble_ll_stats, rx_scan_rsps);
        break;
    case BLE_ADV_PDU_TYPE_CONNECT_REQ:
        STATS_INC(ble_ll_stats, rx_connect_reqs);
        break;
    case BLE_ADV_PDU_TYPE_ADV_SCAN_IND:
        STATS_INC(ble_ll_stats, rx_scan_ind);
        break;
    default:
        break;
    }
}

int
ble_ll_chk_txrx_octets(uint16_t octets)
{
    int rc;

    if ((octets < BLE_LL_CONN_SUPP_BYTES_MIN) ||
        (octets > BLE_LL_CONN_SUPP_BYTES_MAX)) {
        rc = 0;
    } else {
        rc = 1;
    }

    return rc;
}

int
ble_ll_chk_txrx_time(uint16_t time)
{
    int rc;

    if ((time < BLE_LL_CONN_SUPP_TIME_MIN) ||
        (time > BLE_LL_CONN_SUPP_TIME_MAX)) {
        rc = 0;
    } else {
        rc = 1;
    }

    return rc;
}

int
ble_ll_is_resolvable_priv_addr(uint8_t *addr)
{
    /* XXX: implement this */
    return 0;
}

/* Checks to see that the device is a valid random address */
int
ble_ll_is_valid_random_addr(uint8_t *addr)
{
    int i;
    int rc;
    uint16_t sum;
    uint8_t addr_type;

    /* Make sure all bits are neither one nor zero */
    sum = 0;
    for (i = 0; i < (BLE_DEV_ADDR_LEN -1); ++i) {
        sum += addr[i];
    }
    sum += addr[5] & 0x3f;

    if ((sum == 0) || (sum == ((5*255) + 0x3f))) {
        return 0;
    }

    /* Get the upper two bits of the address */
    rc = 1;
    addr_type = addr[5] & 0xc0;
    if (addr_type == 0xc0) {
        /* Static random address. No other checks needed */
    } else if (addr_type == 0x40) {
        /* Resolvable */
        sum = addr[3] + addr[4] + (addr[5] & 0x3f);
        if ((sum == 0) || (sum == (255 + 255 + 0x3f))) {
            rc = 0;
        }
    } else if (addr_type == 0) {
        /* non-resolvable. Cant be equal to public */
        if (!memcmp(g_dev_addr, addr, BLE_DEV_ADDR_LEN)) {
            rc = 0;
        }
    } else {
        /* Invalid upper two bits */
        rc = 0;
    }

    return rc;
}

/**
 * Called from the HCI command parser when the set random address command
 * is received.
 *
 * Context: Link Layer task (HCI command parser)
 *
 * @param addr Pointer to address
 *
 * @return int 0: success
 */
int
ble_ll_set_random_addr(uint8_t *addr)
{
    int rc;

    rc = BLE_ERR_INV_HCI_CMD_PARMS;
    if (ble_ll_is_valid_random_addr(addr)) {
        memcpy(g_random_addr, addr, BLE_DEV_ADDR_LEN);
        rc = BLE_ERR_SUCCESS;
    }

    return rc;
}

/**
 * Checks to see if an address is our device address (either public or
 * random)
 *
 * @param addr
 * @param addr_type
 *
 * @return int
 */
int
ble_ll_is_our_devaddr(uint8_t *addr, int addr_type)
{
    int rc;
    uint8_t *our_addr;

    if (addr_type) {
        our_addr = g_random_addr;
    } else {
        our_addr = g_dev_addr;
    }

    rc = 0;
    if (!memcmp(our_addr, addr, BLE_DEV_ADDR_LEN)) {
        rc = 1;
    }

    return rc;
}

/**
 * Wait for response timeout function
 *
 * Context: interrupt (ble scheduler)
 *
 * @param arg
 */
void
ble_ll_wfr_timer_exp(void *arg)
{
    int rx_start;
    uint8_t lls;

    rx_start = ble_phy_rx_started();
    lls = g_ble_ll_data.ll_state;

    ble_ll_log(BLE_LL_LOG_ID_WFR_EXP, lls, ble_phy_xcvr_state_get(),
               (uint32_t)rx_start);

    /* If we have started a reception, there is nothing to do here */
    if (!rx_start) {
        switch (lls) {
        case BLE_LL_STATE_ADV:
            ble_ll_adv_wfr_timer_exp();
            break;
        case BLE_LL_STATE_CONNECTION:
            ble_ll_conn_wfr_timer_exp();
            break;
        case BLE_LL_STATE_SCANNING:
            ble_ll_scan_wfr_timer_exp();
            break;
        /* Do nothing here. Fall through intentional */
        case BLE_LL_STATE_INITIATING:
        default:
            break;
        }
    }
}

/**
 * Enable the wait for response timer.
 *
 * Context: Interrupt.
 *
 * @param cputime
 * @param wfr_cb
 * @param arg
 */
void
ble_ll_wfr_enable(uint32_t cputime)
{
    cputime_timer_start(&g_ble_ll_data.ll_wfr_timer, cputime);
}

/**
 * Disable the wait for response timer
 */
void
ble_ll_wfr_disable(void)
{
    cputime_timer_stop(&g_ble_ll_data.ll_wfr_timer);
}

/**
 * ll tx pkt in proc
 *
 * Process ACL data packet input from host
 *
 * Context: Link layer task
 *
 */
static void
ble_ll_tx_pkt_in(void)
{
    uint16_t handle;
    uint16_t length;
    uint16_t pb;
    struct os_mbuf_pkthdr *pkthdr;
    struct os_mbuf *om;

    /* Drain all packets off the queue */
    while (STAILQ_FIRST(&g_ble_ll_data.ll_tx_pkt_q)) {
        /* Get mbuf pointer from packet header pointer */
        pkthdr = STAILQ_FIRST(&g_ble_ll_data.ll_tx_pkt_q);
        om = (struct os_mbuf *)((uint8_t *)pkthdr - sizeof(struct os_mbuf));

        /* Remove from queue */
        STAILQ_REMOVE_HEAD(&g_ble_ll_data.ll_tx_pkt_q, omp_next);

        /* Strip HCI ACL header to get handle and length */
        handle = le16toh(om->om_data);
        length = le16toh(om->om_data + 2);
        os_mbuf_adj(om, sizeof(struct hci_data_hdr));

        /* Do some basic error checking */
        pb = handle & 0x3000;
        if ((pkthdr->omp_len != length) || (pb > 0x1000) || (length == 0)) {
            /* This is a bad ACL packet. Count a stat and free it */
            STATS_INC(ble_ll_stats, bad_acl_hdr);
            os_mbuf_free_chain(om);
            continue;
        }

        /* Hand to connection state machine */
        ble_ll_conn_tx_pkt_in(om, handle, length);
    }
}

/**
 * Count Link Layer statistics for received PDUs
 *
 * Context: Link layer task
 *
 * @param hdr
 * @param len
 */
static void
ble_ll_count_rx_stats(struct ble_mbuf_hdr *hdr, uint16_t len, uint8_t pdu_type)
{
    uint8_t crcok;
    uint8_t chan;

    crcok = BLE_MBUF_HDR_CRC_OK(hdr);
    chan = hdr->rxinfo.channel;
    if (crcok) {
        if (chan < BLE_PHY_NUM_DATA_CHANS) {
            STATS_INC(ble_ll_stats, rx_data_pdu_crc_ok);
            STATS_INCN(ble_ll_stats, rx_data_bytes_crc_ok, len);
        } else {
            STATS_INC(ble_ll_stats, rx_adv_pdu_crc_ok);
            STATS_INCN(ble_ll_stats, rx_adv_bytes_crc_ok, len);
            ble_ll_count_rx_adv_pdus(pdu_type);
        }
    } else {
        if (chan < BLE_PHY_NUM_DATA_CHANS) {
            STATS_INC(ble_ll_stats, rx_data_pdu_crc_err);
            STATS_INCN(ble_ll_stats, rx_data_bytes_crc_err, len);
        } else {
            STATS_INC(ble_ll_stats, rx_adv_pdu_crc_err);
            STATS_INCN(ble_ll_stats, rx_adv_bytes_crc_err, len);
        }
    }
}

/**
 * ll rx pkt in
 *
 * Process received packet from PHY.
 *
 * Context: Link layer task
 *
 */
static void
ble_ll_rx_pkt_in(void)
{
    os_sr_t sr;
    uint8_t pdu_type;
    uint8_t *rxbuf;
    struct os_mbuf_pkthdr *pkthdr;
    struct ble_mbuf_hdr *ble_hdr;
    struct os_mbuf *m;

    /* Drain all packets off the queue */
    while (STAILQ_FIRST(&g_ble_ll_data.ll_rx_pkt_q)) {
        /* Get mbuf pointer from packet header pointer */
        pkthdr = STAILQ_FIRST(&g_ble_ll_data.ll_rx_pkt_q);
        m = (struct os_mbuf *)((uint8_t *)pkthdr - sizeof(struct os_mbuf));

        /* Remove from queue */
        OS_ENTER_CRITICAL(sr);
        STAILQ_REMOVE_HEAD(&g_ble_ll_data.ll_rx_pkt_q, omp_next);
        OS_EXIT_CRITICAL(sr);

        /* Note: pdu type wont get used unless this is an advertising pdu */
        ble_hdr = BLE_MBUF_HDR_PTR(m);
        rxbuf = m->om_data;
        pdu_type = rxbuf[0] & BLE_ADV_PDU_HDR_TYPE_MASK;
        ble_ll_count_rx_stats(ble_hdr, pkthdr->omp_len, pdu_type);

        /* Process the data or advertising pdu */
        if (ble_hdr->rxinfo.channel < BLE_PHY_NUM_DATA_CHANS) {
            ble_ll_conn_rx_data_pdu(m, ble_hdr);
        } else {
            /* Process the PDU */
            switch (BLE_MBUF_HDR_RX_STATE(ble_hdr)) {
            case BLE_LL_STATE_ADV:
                ble_ll_adv_rx_pkt_in(pdu_type, rxbuf, ble_hdr);
                break;
            case BLE_LL_STATE_SCANNING:
                ble_ll_scan_rx_pkt_in(pdu_type, rxbuf, ble_hdr);
                break;
            case BLE_LL_STATE_INITIATING:
                ble_ll_init_rx_pkt_in(rxbuf, ble_hdr);
                break;
            default:
                /* Any other state should never occur */
                STATS_INC(ble_ll_stats, bad_ll_state);
                break;
            }

            /* Free the packet buffer */
            os_mbuf_free_chain(m);
        }
    }
}

/**
 * Called to put a packet on the Link Layer receive packet queue.
 *
 * @param rxpdu Pointer to received PDU
 */
void
ble_ll_rx_pdu_in(struct os_mbuf *rxpdu)
{
    struct os_mbuf_pkthdr *pkthdr;

    pkthdr = OS_MBUF_PKTHDR(rxpdu);
    STAILQ_INSERT_TAIL(&g_ble_ll_data.ll_rx_pkt_q, pkthdr, omp_next);
    os_eventq_put(&g_ble_ll_data.ll_evq, &g_ble_ll_data.ll_rx_pkt_ev);
}

/**
 * Called to put a packet on the Link Layer transmit packet queue.
 *
 * @param txpdu Pointer to transmit packet
 */
void
ble_ll_acl_data_in(struct os_mbuf *txpkt)
{
    os_sr_t sr;
    struct os_mbuf_pkthdr *pkthdr;

    pkthdr = OS_MBUF_PKTHDR(txpkt);
    OS_ENTER_CRITICAL(sr);
    STAILQ_INSERT_TAIL(&g_ble_ll_data.ll_tx_pkt_q, pkthdr, omp_next);
    OS_EXIT_CRITICAL(sr);
    os_eventq_put(&g_ble_ll_data.ll_evq, &g_ble_ll_data.ll_tx_pkt_ev);
}

/**
 * Called upon start of received PDU
 *
 * Context: Interrupt
 *
 * @param rxpdu
 *        chan
 *
 * @return int
 *   < 0: A frame we dont want to receive.
 *   = 0: Continue to receive frame. Dont go from rx to tx
 *   > 0: Continue to receive frame and go from rx to tx when done
 */
int
ble_ll_rx_start(struct os_mbuf *rxpdu, uint8_t chan)
{
    int rc;
    uint8_t pdu_type;
    uint8_t *rxbuf;

    ble_ll_log(BLE_LL_LOG_ID_RX_START, chan, 0, (uint32_t)rxpdu);

    /* Check channel type */
    rxbuf = rxpdu->om_data;
    if (chan < BLE_PHY_NUM_DATA_CHANS) {
        /*
         * Data channel pdu. We should be in CONNECTION state with an
         * ongoing connection
         */
        if (g_ble_ll_data.ll_state == BLE_LL_STATE_CONNECTION) {
            /* Call conection pdu rx start function */
            ble_ll_conn_rx_isr_start();

            /* Set up to go from rx to tx */
            rc = 1;
        } else {
            STATS_INC(ble_ll_stats, bad_ll_state);
            rc = 0;
        }
        return rc;
    }

    /* Advertising channel PDU */
    pdu_type = rxbuf[0] & BLE_ADV_PDU_HDR_TYPE_MASK;

    switch (g_ble_ll_data.ll_state) {
    case BLE_LL_STATE_ADV:
        rc = ble_ll_adv_rx_isr_start(pdu_type);
        break;
    case BLE_LL_STATE_INITIATING:
        if ((pdu_type == BLE_ADV_PDU_TYPE_ADV_IND) ||
            (pdu_type == BLE_ADV_PDU_TYPE_ADV_DIRECT_IND)) {
            rc = 1;
        } else {
            rc = 0;
        }
        break;
    case BLE_LL_STATE_SCANNING:
        rc = ble_ll_scan_rx_isr_start(pdu_type, rxpdu);
        break;
    case BLE_LL_STATE_CONNECTION:
        /* Should not occur */
        assert(0);
        rc = 0;
        break;
    default:
        /* Should not be in this state! */
        rc = -1;
        STATS_INC(ble_ll_stats, bad_ll_state);
        break;
    }

    return rc;
}

/**
 * Called by the PHY when a receive packet has ended.
 *
 * NOTE: Called from interrupt context!
 *
 * @param rxpdu Pointer to received PDU
 *        ble_hdr Pointer to BLE header of received mbuf
 *
 * @return int
 *       < 0: Disable the phy after reception.
 *      == 0: Success. Do not disable the PHY.
 *       > 0: Do not disable PHY as that has already been done.
 */
int
ble_ll_rx_end(struct os_mbuf *rxpdu, struct ble_mbuf_hdr *ble_hdr)
{
    int rc;
    int badpkt;
    uint8_t pdu_type;
    uint8_t len;
    uint8_t chan;
    uint8_t crcok;
    uint16_t mblen;
    uint8_t *rxbuf;

    /* Set the rx buffer pointer to the start of the received data */
    rxbuf = rxpdu->om_data;

    /* Get channel and CRC status from BLE header */
    chan = ble_hdr->rxinfo.channel;
    crcok = BLE_MBUF_HDR_CRC_OK(ble_hdr);

    ble_ll_log(BLE_LL_LOG_ID_RX_END, rxbuf[0],
               ((uint16_t)ble_hdr->rxinfo.flags << 8) | rxbuf[1],
               (BLE_MBUF_HDR_PTR(rxpdu))->beg_cputime);

    /* Check channel type */
    if (chan < BLE_PHY_NUM_DATA_CHANS) {
        /* Set length in the received PDU */
        mblen = rxbuf[1] + BLE_LL_PDU_HDR_LEN;
        OS_MBUF_PKTHDR(rxpdu)->omp_len = mblen;
        rxpdu->om_len = mblen;

        /*
         * NOTE: this looks a bit odd, and it is, but for now we place the
         * received PDU on the Link Layer task before calling the rx end
         * function. We do this to guarantee connection event end ordering
         * and receive PDU processing.
         */
        ble_ll_rx_pdu_in(rxpdu);

        /*
         * Data channel pdu. We should be in CONNECTION state with an
         * ongoing connection.
         */
        rc = ble_ll_conn_rx_isr_end(rxpdu, ble_phy_access_addr_get());
        return rc;
    }

    /* Get advertising PDU type and length */
    pdu_type = rxbuf[0] & BLE_ADV_PDU_HDR_TYPE_MASK;
    len = rxbuf[1] & BLE_ADV_PDU_HDR_LEN_MASK;

    /* Setup the mbuf lengths */
    mblen = len + BLE_LL_PDU_HDR_LEN;
    OS_MBUF_PKTHDR(rxpdu)->omp_len = mblen;
    rxpdu->om_len = mblen;

    /* If the CRC checks, make sure lengths check! */
    if (crcok) {
        badpkt = 0;
        switch (pdu_type) {
        case BLE_ADV_PDU_TYPE_SCAN_REQ:
        case BLE_ADV_PDU_TYPE_ADV_DIRECT_IND:
            if (len != BLE_SCAN_REQ_LEN) {
                badpkt = 1;
            }
            break;
        case BLE_ADV_PDU_TYPE_SCAN_RSP:
        case BLE_ADV_PDU_TYPE_ADV_IND:
        case BLE_ADV_PDU_TYPE_ADV_SCAN_IND:
        case BLE_ADV_PDU_TYPE_ADV_NONCONN_IND:
            if ((len < BLE_DEV_ADDR_LEN) || (len > BLE_ADV_SCAN_IND_MAX_LEN)) {
                badpkt = 1;
            }
            break;
        case BLE_ADV_PDU_TYPE_CONNECT_REQ:
            if (len != BLE_CONNECT_REQ_LEN) {
                badpkt = 1;
            }
            break;
        default:
            badpkt = 1;
            break;
        }

        /* If this is a malformed packet, just kill it here */
        if (badpkt) {
            STATS_INC(ble_ll_stats, rx_adv_malformed_pkts);
            os_mbuf_free_chain(rxpdu);
            rxpdu = NULL;
            rc = -1;
        }
    }


    /* Hand packet to the appropriate state machine (if crc ok) */
    switch (BLE_MBUF_HDR_RX_STATE(ble_hdr)) {
    case BLE_LL_STATE_ADV:
        rc = ble_ll_adv_rx_isr_end(pdu_type, rxpdu, crcok);
        break;
    case BLE_LL_STATE_SCANNING:
        rc = ble_ll_scan_rx_isr_end(rxpdu, crcok);
        break;
    case BLE_LL_STATE_INITIATING:
        rc = ble_ll_init_rx_isr_end(rxpdu, crcok);
        break;
    default:
        rc = -1;
        STATS_INC(ble_ll_stats, bad_ll_state);
        break;
    }

    /* Hand packet up to higher layer (regardless of CRC failure) */
    if (rxpdu) {
        ble_ll_rx_pdu_in(rxpdu);
    }

    return rc;
}

/**
 * Link Layer task.
 *
 * This is the task that runs the Link Layer.
 *
 * @param arg
 */
void
ble_ll_task(void *arg)
{
    struct os_event *ev;
    struct os_callout_func *cf;

    /* Init ble phy */
    ble_phy_init();

    /* Set output power to 1mW (0 dBm) */
    ble_phy_txpwr_set(NIMBLE_OPT_LL_TX_PWR_DBM);

    /* Tell the host that we are ready to receive packets */
    ble_ll_hci_send_noop();

    ble_ll_rand_start();

    /* Wait for an event */
    while (1) {
        ev = os_eventq_get(&g_ble_ll_data.ll_evq);
        switch (ev->ev_type) {
        case OS_EVENT_T_TIMER:
            cf = (struct os_callout_func *)ev;
            assert(cf->cf_func);
            cf->cf_func(ev->ev_arg);
            break;
        case BLE_LL_EVENT_HCI_CMD:
            /* Process HCI command */
            ble_ll_hci_cmd_proc(ev);
            break;
        case BLE_LL_EVENT_ADV_EV_DONE:
            ble_ll_adv_event_done(ev->ev_arg);
            break;
        case BLE_LL_EVENT_SCAN:
            ble_ll_scan_event_proc(ev->ev_arg);
            break;
        case BLE_LL_EVENT_RX_PKT_IN:
            ble_ll_rx_pkt_in();
            break;
        case BLE_LL_EVENT_TX_PKT_IN:
            ble_ll_tx_pkt_in();
            break;
        case BLE_LL_EVENT_CONN_SPVN_TMO:
            ble_ll_conn_spvn_timeout(ev->ev_arg);
            break;
        case BLE_LL_EVENT_CONN_EV_END:
            ble_ll_conn_event_end(ev->ev_arg);
            break;
        default:
            assert(0);
            break;
        }
    }
}

/**
 * ble ll state set
 *
 * Called to set the current link layer state.
 *
 * Context: Interrupt and Link Layer task
 *
 * @param ll_state
 */
void
ble_ll_state_set(uint8_t ll_state)
{
    g_ble_ll_data.ll_state = ll_state;
}

/**
 * ble ll state get
 *
 * Called to get the current link layer state.
 *
 * Context: Link Layer task (can be called from interrupt context though).
 *
 * @return ll_state
 */
uint8_t
ble_ll_state_get(void)
{
    return g_ble_ll_data.ll_state;
}

/**
 * ble ll event send
 *
 * Send an event to the Link Layer task
 *
 * @param ev Event to add to the Link Layer event queue.
 */
void
ble_ll_event_send(struct os_event *ev)
{
    os_eventq_put(&g_ble_ll_data.ll_evq, ev);
}

/**
 * Returns the features supported by the link layer
 *
 * @return uint8_t bitmask of supported features.
 */
uint64_t
ble_ll_read_supp_states(void)
{
    return BLE_LL_SUPPORTED_STATES;
}

/**
 * Returns the features supported by the link layer
 *
 * @return uint8_t bitmask of supported features.
 */
uint8_t
ble_ll_read_supp_features(void)
{
    return g_ble_ll_data.ll_supp_features;
}

/**
 * Flush a link layer packet queue.
 *
 * @param pktq
 */
static void
ble_ll_flush_pkt_queue(struct ble_ll_pkt_q *pktq)
{
    struct os_mbuf_pkthdr *pkthdr;
    struct os_mbuf *om;

    /* FLush all packets from Link layer queues */
    while (STAILQ_FIRST(pktq)) {
        /* Get mbuf pointer from packet header pointer */
        pkthdr = STAILQ_FIRST(pktq);
        om = OS_MBUF_PKTHDR_TO_MBUF(pkthdr);

        /* Remove from queue and free the mbuf */
        STAILQ_REMOVE_HEAD(pktq, omp_next);
        os_mbuf_free_chain(om);
    }
}

/**
 * Called to initialize a mbuf used by the controller
 *
 * @param m
 * @param pdulen
 * @param hdr
 */
void
ble_ll_mbuf_init(struct os_mbuf *m, uint8_t pdulen, uint8_t hdr)
{
    struct ble_mbuf_hdr *ble_hdr;

    /* Set mbuf length and packet length */
    m->om_len = pdulen;
    OS_MBUF_PKTHDR(m)->omp_len = pdulen;

    /* Set BLE transmit header */
    ble_hdr = BLE_MBUF_HDR_PTR(m);
    ble_hdr->txinfo.flags = 0;
    ble_hdr->txinfo.offset = 0;
    ble_hdr->txinfo.pyld_len = pdulen;
    ble_hdr->txinfo.hdr_byte = hdr;
}

/**
 * Called to reset the controller. This performs a "software reset" of the link
 * layer; it does not perform a HW reset of the controller nor does it reset
 * the HCI interface.
 *
 * Context: Link Layer task (HCI command)
 *
 * @return int The ble error code to place in the command complete event that
 * is returned when this command is issued.
 */
int
ble_ll_reset(void)
{
    int rc;
    os_sr_t sr;

    /* Stop the phy */
    ble_phy_disable();

    /* Stop any wait for response timer */
    OS_ENTER_CRITICAL(sr);
    ble_ll_wfr_disable();
    ble_ll_sched_stop();
    OS_EXIT_CRITICAL(sr);

    /* Stop any scanning */
    ble_ll_scan_reset();

    /* Stop any advertising */
    ble_ll_adv_reset();

    /* FLush all packets from Link layer queues */
    ble_ll_flush_pkt_queue(&g_ble_ll_data.ll_tx_pkt_q);
    ble_ll_flush_pkt_queue(&g_ble_ll_data.ll_rx_pkt_q);

    /* Reset LL stats */
    memset((uint8_t *)&ble_ll_stats + sizeof(struct stats_hdr), 0,
           sizeof(struct stats_ble_ll_stats) - sizeof(struct stats_hdr));

#ifdef BLE_LL_LOG
    g_ble_ll_log_index = 0;
    memset(&g_ble_ll_log, 0, sizeof(g_ble_ll_log));
#endif

    /* Reset connection module */
    ble_ll_conn_module_reset();

    /* All this does is re-initialize the event masks so call the hci init */
    ble_ll_hci_init();

    /* Set state to standby */
    ble_ll_state_set(BLE_LL_STATE_STANDBY);

    /* Reset our random address */
    memset(g_random_addr, 0, BLE_DEV_ADDR_LEN);

    /* Re-initialize the PHY */
    rc = ble_phy_init();

    return rc;
}

/**
 * Initialize the Link Layer. Should be called only once
 *
 * @return int
 */
int
ble_ll_init(uint8_t ll_task_prio, uint8_t num_acl_pkts, uint16_t acl_pkt_size)
{
    int rc;
    uint8_t features;
    struct ble_ll_obj *lldata;

    /* Get pointer to global data object */
    lldata = &g_ble_ll_data;

    /* Set acl pkt size and number */
    lldata->ll_num_acl_pkts = num_acl_pkts;
    lldata->ll_acl_pkt_size = acl_pkt_size;

    /* Initialize eventq */
    os_eventq_init(&lldata->ll_evq);

    /* Initialize the transmit (from host) and receive (from phy) queues */
    STAILQ_INIT(&lldata->ll_tx_pkt_q);
    STAILQ_INIT(&lldata->ll_rx_pkt_q);

    /* Initialize transmit (from host) and receive packet (from phy) event */
    lldata->ll_rx_pkt_ev.ev_type = BLE_LL_EVENT_RX_PKT_IN;
    lldata->ll_tx_pkt_ev.ev_type = BLE_LL_EVENT_TX_PKT_IN;

    /* Initialize wait for response timer */
    cputime_timer_init(&g_ble_ll_data.ll_wfr_timer, ble_ll_wfr_timer_exp,
                       NULL);

    /* Initialize LL HCI */
    ble_ll_hci_init();

    /* Init the scheduler */
    ble_ll_sched_init();

    /* Initialize advertiser */
    ble_ll_adv_init();

    /* Initialize a scanner */
    ble_ll_scan_init();

    /* Initialize the connection module */
    ble_ll_conn_module_init();

    /* Set the supported features. NOTE: we always support extended reject. */
    features = BLE_LL_FEAT_EXTENDED_REJ;

#if (BLE_LL_CFG_FEAT_DATA_LEN_EXT == 1)
    features |= BLE_LL_FEAT_DATA_LEN_EXT;
#endif
#if (BLE_LL_CFG_FEAT_CONN_PARAM_REQ == 1)
    features |= BLE_LL_FEAT_CONN_PARM_REQ;
#endif
#if (BLE_LL_CFG_FEAT_SLAVE_INIT_FEAT_XCHG == 1)
    features |= BLE_LL_FEAT_SLAVE_INIT;
#endif
#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
    features |= BLE_LL_FEAT_LE_ENCRYPTION;
#endif

    /* Initialize random number generation */
    ble_ll_rand_init();

    lldata->ll_supp_features = features;

    /* Initialize the LL task */
    os_task_init(&g_ble_ll_task, "ble_ll", ble_ll_task, NULL, ll_task_prio,
                 OS_WAIT_FOREVER, g_ble_ll_stack, BLE_LL_STACK_SIZE);

    rc = stats_init_and_reg(STATS_HDR(ble_ll_stats),
                            STATS_SIZE_INIT_PARMS(ble_ll_stats, STATS_SIZE_32),
                            STATS_NAME_INIT_PARMS(ble_ll_stats),
                            "ble_ll");
    return rc;
}

#ifdef BLE_LL_LOG
void
ble_ll_log_dump_index(int i)
{
    struct ble_ll_log *log;

    log = &g_ble_ll_log[i];

    console_printf("cputime=%lu id=%u a8=%u a16=%u a32=%lu\n",
                   log->cputime, log->log_id, log->log_a8,
                   log->log_a16, log->log_a32);
}

void
ble_ll_log_dump(void)
{
    int i;

    for (i = g_ble_ll_log_index; i < BLE_LL_LOG_LEN; ++i) {
        ble_ll_log_dump_index(i);
    }
    for (i = 0; i < g_ble_ll_log_index; ++i) {
        ble_ll_log_dump_index(i);
    }
}
#endif
