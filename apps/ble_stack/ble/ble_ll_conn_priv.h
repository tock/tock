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

#ifndef H_BLE_LL_CONN_PRIV_
#define H_BLE_LL_CONN_PRIV_

#include "controller/ble_ll_conn.h"

/*
 * Definitions for max rx/tx time/bytes for connections
 *  NOTE: you get 327 usecs from 27 bytes of payload by:
 *      -> adding 4 bytes for MIC
 *      -> adding 2 bytes for PDU header
 *      -> adding 8 bytes for preamble (1), access address (4) and crc (3).
 */
#define BLE_LL_CONN_SUPP_TIME_MIN           (328)   /* usecs */
#define BLE_LL_CONN_SUPP_TIME_MAX           (2120)  /* usecs */
#define BLE_LL_CONN_SUPP_BYTES_MIN          (27)    /* bytes */
#define BLE_LL_CONN_SUPP_BYTES_MAX          (251)   /* bytes */

/* Connection event timing */
#define BLE_LL_CONN_INITIAL_OFFSET          (1250)
#define BLE_LL_CONN_ITVL_USECS              (1250)
#define BLE_LL_CONN_TX_WIN_USECS            (1250)
#define BLE_LL_CONN_CE_USECS                (625)
#define BLE_LL_CONN_TX_WIN_MIN              (1)         /* in tx win units */
#define BLE_LL_CONN_SLAVE_LATENCY_MAX       (499)

/* Connection request duration (in usecs) */
#define BLE_LL_CONN_REQ_DURATION            (352)

/* Connection handle range */
#define BLE_LL_CONN_MAX_CONN_HANDLE         (0x0EFF)

/* Offset (in bytes) of advertising address in connect request */
#define BLE_LL_CONN_REQ_ADVA_OFF    (BLE_LL_PDU_HDR_LEN + BLE_DEV_ADDR_LEN)

/* Global Link Layer connection parameters */
struct ble_ll_conn_global_params
{
    uint8_t master_chan_map[BLE_LL_CONN_CHMAP_LEN];
    uint8_t num_used_chans;
    uint8_t supp_max_tx_octets;
    uint8_t supp_max_rx_octets;
    uint8_t conn_init_max_tx_octets;
    uint8_t sugg_tx_octets;
    uint16_t sugg_tx_time;
    uint16_t conn_init_max_tx_time;
    uint16_t supp_max_tx_time;
    uint16_t supp_max_rx_time;
};
extern struct ble_ll_conn_global_params g_ble_ll_conn_params;

/* Some data structures used by other LL routines */
SLIST_HEAD(ble_ll_conn_active_list, ble_ll_conn_sm);
STAILQ_HEAD(ble_ll_conn_free_list, ble_ll_conn_sm);
extern struct ble_ll_conn_active_list g_ble_ll_conn_active_list;
extern struct ble_ll_conn_free_list g_ble_ll_conn_free_list;

/* Pointer to connection state machine we are trying to create */
extern struct ble_ll_conn_sm *g_ble_ll_conn_create_sm;

/* Generic interface */
struct ble_ll_len_req;
struct hci_create_conn;
struct ble_mbuf_hdr;
void ble_ll_conn_sm_new(struct ble_ll_conn_sm *connsm);
void ble_ll_conn_end(struct ble_ll_conn_sm *connsm, uint8_t ble_err);
void ble_ll_conn_enqueue_pkt(struct ble_ll_conn_sm *connsm, struct os_mbuf *om,
                             uint8_t hdr_byte, uint8_t length);
struct ble_ll_conn_sm *ble_ll_conn_sm_get(void);
void ble_ll_conn_master_init(struct ble_ll_conn_sm *connsm,
                             struct hci_create_conn *hcc);
struct ble_ll_conn_sm *ble_ll_conn_find_active_conn(uint16_t handle);
void ble_ll_conn_datalen_update(struct ble_ll_conn_sm *connsm,
                                struct ble_ll_len_req *req);

/* Advertising interface */
int ble_ll_conn_slave_start(uint8_t *rxbuf, uint32_t conn_req_end);

/* Link Layer interface */
void ble_ll_conn_module_init(void);
void ble_ll_conn_set_global_chanmap(uint8_t num_used_chans, uint8_t *chanmap);
void ble_ll_conn_module_reset(void);
void ble_ll_conn_event_end(void *arg);
void ble_ll_conn_tx_pkt_in(struct os_mbuf *om, uint16_t handle, uint16_t len);
void ble_ll_conn_spvn_timeout(void *arg);
void ble_ll_conn_rx_isr_start(void);
int ble_ll_conn_rx_isr_end(struct os_mbuf *rxpdu, uint32_t aa);
void ble_ll_conn_rx_data_pdu(struct os_mbuf *rxpdu, struct ble_mbuf_hdr *hdr);
void ble_ll_init_rx_pkt_in(uint8_t *rxbuf, struct ble_mbuf_hdr *ble_hdr);
int ble_ll_init_rx_isr_end(struct os_mbuf *rxpdu, uint8_t crcok);
void ble_ll_conn_wfr_timer_exp(void);
int ble_ll_conn_is_lru(struct ble_ll_conn_sm *s1, struct ble_ll_conn_sm *s2);
uint32_t ble_ll_conn_get_ce_end_time(void);
void ble_ll_conn_event_halt(void);
uint8_t ble_ll_conn_calc_used_chans(uint8_t *chmap);

/* HCI */
void ble_ll_disconn_comp_event_send(struct ble_ll_conn_sm *connsm,
                                    uint8_t reason);
int ble_ll_conn_hci_disconnect_cmd(uint8_t *cmdbuf);
int ble_ll_conn_hci_rd_rem_ver_cmd(uint8_t *cmdbuf);
int ble_ll_conn_create(uint8_t *cmdbuf);
int ble_ll_conn_hci_update(uint8_t *cmdbuf);
int ble_ll_conn_hci_set_chan_class(uint8_t *cmdbuf);
int ble_ll_conn_hci_param_reply(uint8_t *cmdbuf, int negative_reply);
int ble_ll_conn_create_cancel(void);
void ble_ll_conn_num_comp_pkts_event_send(void);
void ble_ll_conn_comp_event_send(struct ble_ll_conn_sm *connsm, uint8_t status);
void ble_ll_conn_timeout(struct ble_ll_conn_sm *connsm, uint8_t ble_err);
int ble_ll_conn_hci_chk_conn_params(uint16_t itvl_min, uint16_t itvl_max,
                                    uint16_t latency, uint16_t spvn_tmo);
int ble_ll_conn_hci_read_rem_features(uint8_t *cmdbuf);
int ble_ll_conn_hci_rd_rssi(uint8_t *cmdbuf, uint8_t *rspbuf, uint8_t *rsplen);
int ble_ll_conn_hci_rd_chan_map(uint8_t *cmdbuf, uint8_t *rspbuf,
                                uint8_t *rsplen);
int ble_ll_conn_hci_set_data_len(uint8_t *cmdbuf, uint8_t *rspbuf,
                                 uint8_t *rsplen);
int ble_ll_conn_hci_le_start_encrypt(uint8_t *cmdbuf);
int ble_ll_conn_hci_le_ltk_reply(uint8_t *cmdbuf, uint8_t *rspbuf, uint8_t ocf);
#endif /* H_BLE_LL_CONN_PRIV_ */
