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

#ifndef H_BLE_LL_CONN_
#define H_BLE_LL_CONN_

#include "os/os.h"
#include "nimble/ble.h"
#include "nimble/hci_common.h"
#include "controller/ble_ll_sched.h"
#include "controller/ble_ll_ctrl.h"
#include "hal/hal_cputime.h"

/* Roles */
#define BLE_LL_CONN_ROLE_NONE           (0)
#define BLE_LL_CONN_ROLE_MASTER         (1)
#define BLE_LL_CONN_ROLE_SLAVE          (2)

/* Connection states */
#define BLE_LL_CONN_STATE_IDLE          (0)
#define BLE_LL_CONN_STATE_CREATED       (1)
#define BLE_LL_CONN_STATE_ESTABLISHED   (2)

/* Channel map size */
#define BLE_LL_CONN_CHMAP_LEN           (5)

/* Definitions for source clock accuracy */
#define BLE_MASTER_SCA_251_500_PPM      (0)
#define BLE_MASTER_SCA_151_250_PPM      (1)
#define BLE_MASTER_SCA_101_150_PPM      (2)
#define BLE_MASTER_SCA_76_100_PPM       (3)
#define BLE_MASTER_SCA_51_75_PPM        (4)
#define BLE_MASTER_SCA_31_50_PPM        (5)
#define BLE_MASTER_SCA_21_30_PPM        (6)
#define BLE_MASTER_SCA_0_20_PPM         (7)

/* Definition for RSSI when the RSSI is unknown */
#define BLE_LL_CONN_UNKNOWN_RSSI        (127)

#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
/*
 * Encryption states for a connection
 *
 * NOTE: the states are ordered so that we can check to see if the state
 * is greater than ENCRYPTED. If so, it means that the start or pause
 * encryption procedure is running and we should not send data pdu's.
 */
enum conn_enc_state {
    CONN_ENC_S_UNENCRYPTED = 1,
    CONN_ENC_S_ENCRYPTED,
    CONN_ENC_S_ENC_RSP_WAIT,
    CONN_ENC_S_START_ENC_REQ_WAIT,
    CONN_ENC_S_START_ENC_RSP_WAIT,
    CONN_ENC_S_PAUSE_ENC_RSP_WAIT,
    CONN_ENC_S_LTK_REQ_WAIT,
    CONN_ENC_S_LTK_NEG_REPLY
};

/*
 * Note that the LTK is the key, the SDK is the plain text, and the
 * session key is the cipher text portion of the encryption block.
 */
struct ble_ll_conn_enc_data
{
    uint8_t enc_state;
    uint8_t tx_encrypted;
    uint16_t enc_div;
    uint16_t tx_pkt_cntr;
    uint16_t rx_pkt_cntr;
    uint64_t host_rand_num;
    uint8_t iv[8];
    struct ble_encryption_block enc_block;
};
#endif

/* Connection state machine flags. */
union ble_ll_conn_sm_flags {
    struct {
        uint32_t pkt_rxd:1;
        uint32_t terminate_ind_txd:1;
        uint32_t terminate_ind_rxd:1;
        uint32_t allow_slave_latency:1;
        uint32_t slave_set_last_anchor:1;
        uint32_t awaiting_host_reply:1;
        uint32_t send_conn_upd_event:1;
        uint32_t conn_update_sched:1;
        uint32_t host_expects_upd_event:1;
        uint32_t version_ind_sent:1;
        uint32_t rxd_version_ind:1;
        uint32_t chanmap_update_scheduled:1;
        uint32_t conn_empty_pdu_txd:1;
        uint32_t last_txd_md:1;
        uint32_t conn_req_txd:1;
        uint32_t send_ltk_req:1;
        uint32_t encrypted:1;
        uint32_t encrypt_chg_sent:1;
    } cfbit;
    uint32_t conn_flags;
} __attribute__((packed));

/* Connection state machine */
struct ble_ll_conn_sm
{
    /* Connection state machine flags */
    union ble_ll_conn_sm_flags csmflags;

    /* Current connection handle, state and role */
    uint16_t conn_handle;
    uint8_t conn_state;
    uint8_t conn_role;          /* Can possibly be 1 bit */

    /* Connection data length management */
    uint8_t max_tx_octets;
    uint8_t max_rx_octets;
    uint8_t rem_max_tx_octets;
    uint8_t rem_max_rx_octets;
    uint8_t eff_max_tx_octets;
    uint8_t eff_max_rx_octets;
    uint16_t max_tx_time;
    uint16_t max_rx_time;
    uint16_t rem_max_tx_time;
    uint16_t rem_max_rx_time;
    uint16_t eff_max_tx_time;
    uint16_t eff_max_rx_time;

    /* Used to calculate data channel index for connection */
    uint8_t chanmap[BLE_LL_CONN_CHMAP_LEN];
    uint8_t req_chanmap[BLE_LL_CONN_CHMAP_LEN];
    uint16_t chanmap_instant;
    uint8_t hop_inc;
    uint8_t data_chan_index;
    uint8_t unmapped_chan;
    uint8_t last_unmapped_chan;
    uint8_t num_used_chans;

    /* RSSI */
    int8_t conn_rssi;

    /* Ack/Flow Control */
    uint8_t tx_seqnum;          /* note: can be 1 bit */
    uint8_t next_exp_seqnum;    /* note: can be 1 bit */
    uint8_t cons_rxd_bad_crc;   /* note: can be 1 bit */
    uint8_t last_rxd_sn;        /* note: cant be 1 bit given current code */
    uint8_t last_rxd_hdr_byte;  /* note: possibly can make 1 bit since we
                                   only use the MD bit now */

    /* connection event mgmt */
    uint8_t reject_reason;
    uint8_t host_reply_opcode;
    uint8_t master_sca;
    uint8_t tx_win_size;
    uint8_t cur_ctrl_proc;
    uint8_t disconnect_reason;
    uint8_t rxd_disconnect_reason;
    uint8_t common_features;        /* Just a uint8 for now */
    uint8_t vers_nr;
    uint16_t pending_ctrl_procs;
    uint16_t event_cntr;
    uint16_t completed_pkts;
    uint16_t comp_id;
    uint16_t sub_vers_nr;

    uint32_t access_addr;
    uint32_t crcinit;               /* only low 24 bits used */
    /* XXX: do we need ce_end_time? Cant this be sched end time? */
    uint32_t ce_end_time;   /* cputime at which connection event should end */
    uint32_t terminate_timeout;
    uint32_t last_scheduled;

    /* Connection timing */
    uint16_t conn_itvl_min;
    uint16_t conn_itvl_max;
    uint16_t conn_itvl;
    uint16_t slave_latency;
    uint16_t supervision_tmo;
    uint16_t min_ce_len;
    uint16_t max_ce_len;
    uint16_t tx_win_off;
    uint32_t anchor_point;
    uint32_t last_anchor_point;
    uint32_t slave_cur_tx_win_usecs;
    uint32_t slave_cur_window_widening;

    /* address information */
    uint8_t own_addr_type;
    uint8_t peer_addr_type;
    uint8_t peer_addr[BLE_DEV_ADDR_LEN];

    /* connection supervisor timer */
    struct cpu_timer conn_spvn_timer;

    /* connection supervision timeout event */
    struct os_event conn_spvn_ev;

    /* Connection end event */
    struct os_event conn_ev_end;

    /* Packet transmit queue */
    struct os_mbuf *cur_tx_pdu;
    STAILQ_HEAD(conn_txq_head, os_mbuf_pkthdr) conn_txq;

    /* List entry for active/free connection pools */
    union {
        SLIST_ENTRY(ble_ll_conn_sm) act_sle;
        STAILQ_ENTRY(ble_ll_conn_sm) free_stqe;
    };

    /* LL control procedure response timer */
    struct os_callout_func ctrl_proc_rsp_timer;

    /* For scheduling connections */
    struct ble_ll_sched_item conn_sch;

    /*
     * XXX: a note on all these structures for control procedures. First off,
     * all of these need to be ifdef'd to save memory. Another thing to
     * consider is this: since most control procedures can only run when no
     * others are running, can I use just one structure (a union)? Should I
     * allocate these from a pool? Not sure what to do. For now, I just use
     * a large chunk of memory per connection.
     */
#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
    struct ble_ll_conn_enc_data enc_data;
#endif
    /*
     * For connection update procedure. XXX: can make this a pointer and
     * malloc it if we want to save space.
     */
    struct hci_conn_update conn_param_req;

    /* For connection update procedure */
    struct ble_ll_conn_upd_req conn_update_req;

    /* XXX: for now, just store them all */
    struct ble_ll_conn_params conn_cp;
};

/* Flags */
#define CONN_F_UPDATE_SCHED(csm)    ((csm)->csmflags.cfbit.conn_update_sched)
#define CONN_F_EMPTY_PDU_TXD(csm)   ((csm)->csmflags.cfbit.conn_empty_pdu_txd)
#define CONN_F_LAST_TXD_MD(csm)     ((csm)->csmflags.cfbit.last_txd_md)
#define CONN_F_CONN_REQ_TXD(csm)    ((csm)->csmflags.cfbit.conn_req_txd)
#define CONN_F_ENCRYPTED(csm)       ((csm)->csmflags.cfbit.encrypted)
#define CONN_F_ENC_CHANGE_SENT(csm) ((csm)->csmflags.cfbit.encrypt_chg_sent)

/* Role */
#define CONN_IS_MASTER(csm)         (csm->conn_role == BLE_LL_CONN_ROLE_MASTER)
#define CONN_IS_SLAVE(csm)          (csm->conn_role == BLE_LL_CONN_ROLE_SLAVE)

/*
 * Given a handle, returns an active connection state machine (or NULL if the
 * handle does not exist
 *
 */
struct ble_ll_conn_sm *ble_ll_conn_find_active_conn(uint16_t handle);

#endif /* H_BLE_LL_CONN_ */
