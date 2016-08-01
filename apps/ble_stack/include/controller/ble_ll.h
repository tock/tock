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

#ifndef H_BLE_LL_
#define H_BLE_LL_

#include "stats/stats.h"
#include "hal/hal_cputime.h"
#include "nimble/nimble_opt.h"

/* Controller revision. */
#define BLE_LL_SUB_VERS_NR      (0x0000)

/*
 * The amount of time that we will wait to hear the start of a receive
 * packet after we have transmitted a packet. This time is at least
 * an IFS time plus the time to receive the preamble and access address (which
 * is 40 usecs). We add an additional 32 usecs just to be safe.
 *
 * XXX: move this definition and figure out how we determine the worst-case
 * jitter (spec. should have this).
 */
#define BLE_LL_WFR_USECS    (BLE_LL_IFS + 40 + 32)

/* Packet queue header definition */
STAILQ_HEAD(ble_ll_pkt_q, os_mbuf_pkthdr);

/*
 * Global Link Layer data object. There is only one Link Layer data object
 * per controller although there may be many instances of the link layer state
 * machine running.
 */
struct ble_ll_obj
{
    /* Current Link Layer state */
    uint8_t ll_state;

    /* Supported features */
    uint8_t ll_supp_features;

    /* Number of ACL data packets supported */
    uint8_t ll_num_acl_pkts;
    uint8_t _pad;

    /* ACL data packet size */
    uint16_t ll_acl_pkt_size;
    uint16_t _pad16;

    /* Task event queue */
    struct os_eventq ll_evq;

    /* Wait for response timer */
    struct cpu_timer ll_wfr_timer;

    /* Packet receive queue (and event). Holds received packets from PHY */
    struct os_event ll_rx_pkt_ev;
    struct ble_ll_pkt_q ll_rx_pkt_q;

    /* Packet transmit queue */
    struct os_event ll_tx_pkt_ev;
    struct ble_ll_pkt_q ll_tx_pkt_q;
};
extern struct ble_ll_obj g_ble_ll_data;

/* Link layer statistics */
STATS_SECT_START(ble_ll_stats)
    STATS_SECT_ENTRY(hci_cmds)
    STATS_SECT_ENTRY(hci_cmd_errs)
    STATS_SECT_ENTRY(hci_events_sent)
    STATS_SECT_ENTRY(bad_ll_state)
    STATS_SECT_ENTRY(bad_acl_hdr)
    STATS_SECT_ENTRY(rx_adv_pdu_crc_ok)
    STATS_SECT_ENTRY(rx_adv_pdu_crc_err)
    STATS_SECT_ENTRY(rx_adv_bytes_crc_ok)
    STATS_SECT_ENTRY(rx_adv_bytes_crc_err)
    STATS_SECT_ENTRY(rx_data_pdu_crc_ok)
    STATS_SECT_ENTRY(rx_data_pdu_crc_err)
    STATS_SECT_ENTRY(rx_data_bytes_crc_ok)
    STATS_SECT_ENTRY(rx_data_bytes_crc_err)
    STATS_SECT_ENTRY(rx_adv_malformed_pkts)
    STATS_SECT_ENTRY(rx_adv_ind)
    STATS_SECT_ENTRY(rx_adv_direct_ind)
    STATS_SECT_ENTRY(rx_adv_nonconn_ind)
    STATS_SECT_ENTRY(rx_scan_reqs)
    STATS_SECT_ENTRY(rx_scan_rsps)
    STATS_SECT_ENTRY(rx_connect_reqs)
    STATS_SECT_ENTRY(rx_scan_ind)
    STATS_SECT_ENTRY(adv_txg)
    STATS_SECT_ENTRY(adv_late_starts)
    STATS_SECT_ENTRY(sched_state_conn_errs)
    STATS_SECT_ENTRY(sched_state_adv_errs)
    STATS_SECT_ENTRY(scan_starts)
    STATS_SECT_ENTRY(scan_stops)
    STATS_SECT_ENTRY(scan_req_txf)
    STATS_SECT_ENTRY(scan_req_txg)
    STATS_SECT_ENTRY(scan_rsp_txg)
STATS_SECT_END
extern STATS_SECT_DECL(ble_ll_stats) ble_ll_stats;

/* States */
#define BLE_LL_STATE_STANDBY        (0)
#define BLE_LL_STATE_ADV            (1)
#define BLE_LL_STATE_SCANNING       (2)
#define BLE_LL_STATE_INITIATING     (3)
#define BLE_LL_STATE_CONNECTION     (4)

/* BLE LL Task Events */
#define BLE_LL_EVENT_HCI_CMD        (OS_EVENT_T_PERUSER)
#define BLE_LL_EVENT_ADV_EV_DONE    (OS_EVENT_T_PERUSER + 1)
#define BLE_LL_EVENT_RX_PKT_IN      (OS_EVENT_T_PERUSER + 2)
#define BLE_LL_EVENT_SCAN           (OS_EVENT_T_PERUSER + 3)
#define BLE_LL_EVENT_CONN_SPVN_TMO  (OS_EVENT_T_PERUSER + 4)
#define BLE_LL_EVENT_CONN_EV_END    (OS_EVENT_T_PERUSER + 5)
#define BLE_LL_EVENT_TX_PKT_IN      (OS_EVENT_T_PERUSER + 6)

/* LL Features */
#define BLE_LL_FEAT_LE_ENCRYPTION   (0x01)
#define BLE_LL_FEAT_CONN_PARM_REQ   (0x02)
#define BLE_LL_FEAT_EXTENDED_REJ    (0x04)
#define BLE_LL_FEAT_SLAVE_INIT      (0x08)
#define BLE_LL_FEAT_LE_PING         (0x10)
#define BLE_LL_FEAT_DATA_LEN_EXT    (0x20)
#define BLE_LL_FEAT_LL_PRIVACY      (0x40)
#define BLE_LL_FEAT_EXT_SCAN_FILT   (0x80)

/* LL timing */
#define BLE_LL_IFS                  (150)       /* usecs */

/*
 * BLE LL device address. Note that element 0 of the array is the LSB and
 * is sent over the air first. Byte 5 is the MSB and is the last one sent over
 * the air.
 */
#define BLE_DEV_ADDR_LEN            (6)     /* bytes */

struct ble_dev_addr
{
    uint8_t u8[BLE_DEV_ADDR_LEN];
};

#define BLE_IS_DEV_ADDR_STATIC(addr)        ((addr->u8[5] & 0xc0) == 0xc0)
#define BLE_IS_DEV_ADDR_RESOLVABLE(addr)    ((addr->u8[5] & 0xc0) == 0x40)
#define BLE_IS_DEV_ADDR_UNRESOLVABLE(addr)  ((addr->u8[5] & 0xc0) == 0x00)

/*
 * LL packet format
 *
 *  -> Preamble         (1 byte)
 *  -> Access Address   (4 bytes)
 *  -> PDU              (2 to 257 octets)
 *  -> CRC              (3 bytes)
 */
#define BLE_LL_PREAMBLE_LEN     (1)
#define BLE_LL_ACC_ADDR_LEN     (4)
#define BLE_LL_CRC_LEN          (3)
#define BLE_LL_OVERHEAD_LEN     \
    (BLE_LL_CRC_LEN + BLE_LL_ACC_ADDR_LEN + BLE_LL_PREAMBLE_LEN)
#define BLE_LL_PDU_HDR_LEN      (2)
#define BLE_LL_MIN_PDU_LEN      (BLE_LL_PDU_HDR_LEN)
#define BLE_LL_MAX_PDU_LEN      (257)
#define BLE_LL_CRCINIT_ADV      (0x555555)
#define BLE_LL_PDU_OVERHEAD     (BLE_LL_OVERHEAD_LEN + BLE_LL_PDU_HDR_LEN)

/**
 * ll pdu tx time get
 *
 * Returns the number of usecs it will take to transmit a PDU of payload
 * length 'len' bytes. Each byte takes 8 usecs. This routine includes the LL
 * overhead: preamble (1), access addr (4) and crc (3) and the PDU header (2)
 * for a total of 10 bytes.
 *
 * @param len The length of the PDU payload (does not include include header).
 *
 * @return uint16_t The number of usecs it will take to transmit a PDU of
 *                  length 'len' bytes.
 */
#define BLE_TX_DUR_USECS_M(len)     (((len) + BLE_LL_PDU_OVERHEAD) << 3)

/* Calculates the time it takes to transmit 'len' bytes */
#define BLE_TX_LEN_USECS_M(len)     ((len) << 3)

/* Access address for advertising channels */
#define BLE_ACCESS_ADDR_ADV             (0x8E89BED6)

/*
 * Advertising PDU format:
 * -> 2 byte header
 *      -> LSB contains pdu type, txadd and rxadd bits.
 *      -> MSB contains length (6 bits). Length is length of payload. Does
 *         not include the header length itself.
 * -> Payload (max 37 bytes)
 */
#define BLE_ADV_PDU_HDR_TYPE_MASK           (0x0F)
#define BLE_ADV_PDU_HDR_TXADD_MASK          (0x40)
#define BLE_ADV_PDU_HDR_RXADD_MASK          (0x80)
#define BLE_ADV_PDU_HDR_LEN_MASK            (0x3F)

/* Advertising channel PDU types */
#define BLE_ADV_PDU_TYPE_ADV_IND            (0)
#define BLE_ADV_PDU_TYPE_ADV_DIRECT_IND     (1)
#define BLE_ADV_PDU_TYPE_ADV_NONCONN_IND    (2)
#define BLE_ADV_PDU_TYPE_SCAN_REQ           (3)
#define BLE_ADV_PDU_TYPE_SCAN_RSP           (4)
#define BLE_ADV_PDU_TYPE_CONNECT_REQ        (5)
#define BLE_ADV_PDU_TYPE_ADV_SCAN_IND       (6)

/*
 * TxAdd and RxAdd bit definitions. A 0 is a public address; a 1 is a
 * random address.
 */
#define BLE_ADV_PDU_HDR_TXADD_RAND          (0x40)
#define BLE_ADV_PDU_HDR_RXADD_RAND          (0x80)

/*
 * Data Channel format
 *
 *  -> Header (2 bytes)
 *      -> LSB contains llid, nesn, sn and md
 *      -> MSB contains length (8 bits)
 *  -> Payload (0 to 251)
 *  -> MIC (0 or 4 bytes)
 */
#define BLE_LL_DATA_HDR_LLID_MASK       (0x03)
#define BLE_LL_DATA_HDR_NESN_MASK       (0x04)
#define BLE_LL_DATA_HDR_SN_MASK         (0x08)
#define BLE_LL_DATA_HDR_MD_MASK         (0x10)
#define BLE_LL_DATA_HDR_RSRVD_MASK      (0xE0)
#define BLE_LL_DATA_PDU_MAX_PYLD        (251)
#define BLE_LL_DATA_MIC_LEN             (4)

/* LLID definitions */
#define BLE_LL_LLID_RSRVD               (0)
#define BLE_LL_LLID_DATA_FRAG           (1)
#define BLE_LL_LLID_DATA_START          (2)
#define BLE_LL_LLID_CTRL                (3)

/*
 * CONNECT_REQ
 *      -> InitA        (6 bytes)
 *      -> AdvA         (6 bytes)
 *      -> LLData       (22 bytes)
 *          -> Access address (4 bytes)
 *          -> CRC init (3 bytes)
 *          -> WinSize (1 byte)
 *          -> WinOffset (2 bytes)
 *          -> Interval (2 bytes)
 *          -> Latency (2 bytes)
 *          -> Timeout (2 bytes)
 *          -> Channel Map (5 bytes)
 *          -> Hop Increment (5 bits)
 *          -> SCA (3 bits)
 *
 *  InitA is the initiators public (TxAdd=0) or random (TxAdd=1) address.
 *  AdvaA is the advertisers public (RxAdd=0) or random (RxAdd=1) address.
 *  LLData contains connection request data.
 *      aa: Link Layer's access address
 *      crc_init: The CRC initialization value used for CRC calculation.
 *      winsize: The transmit window size = winsize * 1.25 msecs
 *      winoffset: The transmit window offset =  winoffset * 1.25 msecs
 *      interval: The connection interval = interval * 1.25 msecs.
 *      latency: connection slave latency = latency
 *      timeout: Connection supervision timeout = timeout * 10 msecs.
 *      chanmap: contains channel mapping indicating used and unused data
 *               channels. Only bits that are 1 are usable. LSB is channel 0.
 *      hop_inc: Hop increment used for frequency hopping. Random value in
 *               range of 5 to 16.
 */
#define BLE_CONNECT_REQ_LEN         (34)
#define BLE_CONNECT_REQ_PDU_LEN     (BLE_CONNECT_REQ_LEN + BLE_LL_PDU_HDR_LEN)

/*--- External API ---*/
/* Initialize the Link Layer */
int
ble_ll_init(uint8_t ll_task_prio, uint8_t num_acl_pkts, uint16_t acl_pkt_size);

/* Reset the Link Layer */
int ble_ll_reset(void);

/* 'Boolean' function returning true if address is a valid random address */
int ble_ll_is_valid_random_addr(uint8_t *addr);

/* Calculate the amount of time a pdu of 'len' bytes will take to transmit */
uint16_t ble_ll_pdu_tx_time_get(uint16_t len);

/* Is this address a resolvable private address? */
int ble_ll_is_resolvable_priv_addr(uint8_t *addr);

/* Is 'addr' our device address? 'addr_type' is public (0) or random (!=0) */
int ble_ll_is_our_devaddr(uint8_t *addr, int addr_type);

/**
 * Called to put a packet on the Link Layer transmit packet queue.
 *
 * @param txpdu Pointer to transmit packet
 */
void ble_ll_acl_data_in(struct os_mbuf *txpkt);

/*--- PHY interfaces ---*/
/* Called by the PHY when a packet has started */
int ble_ll_rx_start(struct os_mbuf *rxpdu, uint8_t chan);

/* Called by the PHY when a packet reception ends */
struct ble_mbuf_hdr;
int ble_ll_rx_end(struct os_mbuf *rxpdu, struct ble_mbuf_hdr *ble_hdr);

/*--- Controller API ---*/
void ble_ll_mbuf_init(struct os_mbuf *m, uint8_t pdulen, uint8_t hdr);

/* Set the link layer state */
void ble_ll_state_set(uint8_t ll_state);

/* Get the link layer state */
uint8_t ble_ll_state_get(void);

/* Send an event to LL task */
void ble_ll_event_send(struct os_event *ev);

/* Set random address */
int ble_ll_set_random_addr(uint8_t *addr);

/* Enable wait for response timer */
void ble_ll_wfr_enable(uint32_t cputime);

/* Disable wait for response timer */
void ble_ll_wfr_disable(void);

/* Read set of features supported by the Link Layer */
uint8_t ble_ll_read_supp_features(void);

/* Read set of states supported by the Link Layer */
uint64_t ble_ll_read_supp_states(void);

/* Check if octets and time are valid. Returns 0 if not valid */
int ble_ll_chk_txrx_octets(uint16_t octets);
int ble_ll_chk_txrx_time(uint16_t time);

/* Random numbers */
int ble_ll_rand_init(void);
void ble_ll_rand_sample(uint8_t rnum);
int ble_ll_rand_data_get(uint8_t *buf, uint8_t len);
int ble_ll_rand_start(void);

/*
 * XXX: temporary LL debug log. Will get removed once we transition to real
 * log
 */
#undef BLE_LL_LOG
#include "console/console.h"

#define BLE_LL_LOG_ID_PHY_SETCHAN       (1)
#define BLE_LL_LOG_ID_RX_START          (2)
#define BLE_LL_LOG_ID_RX_END            (3)
#define BLE_LL_LOG_ID_WFR_EXP           (4)
#define BLE_LL_LOG_ID_PHY_TXEND         (5)
#define BLE_LL_LOG_ID_PHY_TX            (6)
#define BLE_LL_LOG_ID_PHY_RX            (7)
#define BLE_LL_LOG_ID_PHY_DISABLE       (9)
#define BLE_LL_LOG_ID_CONN_EV_START     (10)
#define BLE_LL_LOG_ID_CONN_TX           (15)
#define BLE_LL_LOG_ID_CONN_RX           (16)
#define BLE_LL_LOG_ID_CONN_TX_RETRY     (17)
#define BLE_LL_LOG_ID_CONN_RX_ACK       (18)
#define BLE_LL_LOG_ID_LL_CTRL_RX        (19)
#define BLE_LL_LOG_ID_CONN_EV_END       (20)
#define BLE_LL_LOG_ID_CONN_END          (30)
#define BLE_LL_LOG_ID_ADV_TXBEG         (50)
#define BLE_LL_LOG_ID_ADV_TXDONE        (60)

#ifdef BLE_LL_LOG
void ble_ll_log(uint8_t id, uint8_t arg8, uint16_t arg16, uint32_t arg32);
#else
#define ble_ll_log(m,n,o,p)
#endif

#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
/* LTK 0x4C68384139F574D836BCF34E9DFB01BF */
extern const uint8_t g_bletest_LTK[];
extern uint16_t g_bletest_EDIV;
extern uint64_t g_bletest_RAND;
extern uint64_t g_bletest_SKDm;
extern uint64_t g_bletest_SKDs;
extern uint32_t g_bletest_IVm;
extern uint32_t g_bletest_IVs;
#endif

#endif /* H_LL_ */
