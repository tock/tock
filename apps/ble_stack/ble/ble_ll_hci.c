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
#include "nimble/ble.h"
#include "nimble/nimble_opt.h"
#include "nimble/hci_common.h"
#include "nimble/hci_transport.h"
#include "controller/ble_hw.h"
#include "controller/ble_ll_adv.h"
#include "controller/ble_ll_scan.h"
#include "controller/ble_ll.h"
#include "controller/ble_ll_hci.h"
#include "controller/ble_ll_whitelist.h"
#include "ble_ll_conn_priv.h"

/* LE event mask */
static uint8_t g_ble_ll_hci_le_event_mask[BLE_HCI_SET_LE_EVENT_MASK_LEN];
static uint8_t g_ble_ll_hci_event_mask[BLE_HCI_SET_EVENT_MASK_LEN];

/**
 * ll hci get num cmd pkts
 *
 * Returns the number of command packets that the host is allowed to send
 * to the controller.
 *
 * @return uint8_t
 */
static uint8_t
ble_ll_hci_get_num_cmd_pkts(void)
{
    return BLE_LL_CFG_NUM_HCI_CMD_PKTS;
}

/**
 * Send an event to the host.
 *
 * @param evbuf Pointer to event buffer to send
 *
 * @return int 0: success; -1 otherwise.
 */
int
ble_ll_hci_event_send(uint8_t *evbuf)
{
    int rc;

    /* Count number of events sent */
    STATS_INC(ble_ll_stats, hci_events_sent);

    /* Send the event to the host */
    rc = ble_hci_transport_ctlr_event_send(evbuf);

    return rc;
}

/**
 * Created and sends a command complete event with the no-op opcode to the
 * host.
 *
 * @return int 0: ok, ble error code otherwise.
 */
int
ble_ll_hci_send_noop(void)
{
    int rc;
    uint8_t *evbuf;
    uint16_t opcode;

    evbuf = os_memblock_get(&g_hci_cmd_pool);
    if (evbuf) {
        /* Create a command complete event with a NO-OP opcode */
        opcode = 0;
        evbuf[0] = BLE_HCI_EVCODE_COMMAND_COMPLETE;
        evbuf[1] = 3;
        evbuf[2] = ble_ll_hci_get_num_cmd_pkts();
        htole16(evbuf + 3, opcode);
        ble_ll_hci_event_send(evbuf);
        rc = BLE_ERR_SUCCESS;
    } else {
        rc = BLE_ERR_MEM_CAPACITY;
    }

    return rc;
}

#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
/**
 * LE encrypt command
 *
 * @param cmdbuf
 * @param rspbuf
 * @param rsplen
 *
 * @return int
 */
static int
ble_ll_hci_le_encrypt(uint8_t *cmdbuf, uint8_t *rspbuf, uint8_t *rsplen)
{
    int rc;
    struct ble_encryption_block ecb;

    /* Call the link layer to encrypt the data */
    swap_buf(ecb.key, cmdbuf, BLE_ENC_BLOCK_SIZE);
    swap_buf(ecb.plain_text, cmdbuf + BLE_ENC_BLOCK_SIZE, BLE_ENC_BLOCK_SIZE);
    rc = ble_hw_encrypt_block(&ecb);
    if (!rc) {
        swap_buf(rspbuf, ecb.cipher_text, BLE_ENC_BLOCK_SIZE);
        *rsplen = BLE_ENC_BLOCK_SIZE;
        rc = BLE_ERR_SUCCESS;
    } else {
        *rsplen = 0;
        rc = BLE_ERR_CTLR_BUSY;
    }
    return rc;
}
#endif

/**
 * LE rand command
 *
 * @param cmdbuf
 * @param rspbuf
 * @param rsplen
 *
 * @return int
 */
static int
ble_ll_hci_le_rand(uint8_t *rspbuf, uint8_t *rsplen)
{
    int rc;

    rc = ble_ll_rand_data_get(rspbuf, BLE_HCI_LE_RAND_LEN);
    *rsplen = BLE_HCI_LE_RAND_LEN;
    return rc;
}

/**
 * Read local version
 *
 * @param rspbuf
 * @param rsplen
 *
 * @return int
 */
static int
ble_ll_hci_rd_local_version(uint8_t *rspbuf, uint8_t *rsplen)
{
    uint16_t hci_rev;
    uint16_t lmp_subver;
    uint16_t mfrg;

    hci_rev = 0;
    lmp_subver = 0;
    mfrg = NIMBLE_OPT_LL_MFRG_ID;

    /* Place the data packet length and number of packets in the buffer */
    rspbuf[0] = BLE_HCI_VER_BCS_4_2;
    htole16(rspbuf + 1, hci_rev);
    rspbuf[3] = BLE_LMP_VER_BCS_4_2;
    htole16(rspbuf + 4, mfrg);
    htole16(rspbuf + 6, lmp_subver);
    *rsplen = BLE_HCI_RD_LOC_VER_INFO_RSPLEN;
    return BLE_ERR_SUCCESS;
}

/**
 * Read local supported features
 *
 * @param rspbuf
 * @param rsplen
 *
 * @return int
 */
static int
ble_ll_hci_rd_local_supp_feat(uint8_t *rspbuf, uint8_t *rsplen)
{
    /*
     * The only two bits we set here currently are:
     *      BR/EDR not supported        (bit 5)
     *      LE supported (controller)   (bit 6)
     */
    memset(rspbuf, 0, BLE_HCI_RD_LOC_SUPP_FEAT_RSPLEN);
    rspbuf[4] = 0x60;
    *rsplen = BLE_HCI_RD_LOC_SUPP_FEAT_RSPLEN;
    return BLE_ERR_SUCCESS;
}

/**
 * Read local supported commands
 *
 * @param rspbuf
 * @param rsplen
 *
 * @return int
 */
static int
ble_ll_hci_rd_local_supp_cmd(uint8_t *rspbuf, uint8_t *rsplen)
{
    memset(rspbuf, 0, BLE_HCI_RD_LOC_SUPP_CMD_RSPLEN);
    memcpy(rspbuf, g_ble_ll_supp_cmds, sizeof(g_ble_ll_supp_cmds));
    *rsplen = BLE_HCI_RD_LOC_SUPP_CMD_RSPLEN;
    return BLE_ERR_SUCCESS;
}

/**
 * Called to read the public device address of the device
 *
 *
 * @param rspbuf
 * @param rsplen
 *
 * @return int
 */
static int
ble_ll_hci_rd_bd_addr(uint8_t *rspbuf, uint8_t *rsplen)
{
    /*
     * XXX: for now, assume we always have a public device address. If we
     * dont, we should set this to zero
     */
    memcpy(rspbuf, g_dev_addr, BLE_DEV_ADDR_LEN);
    *rsplen = BLE_DEV_ADDR_LEN;
    return BLE_ERR_SUCCESS;
}

/**
 * ll hci set le event mask
 *
 * Called when the LL controller receives a set LE event mask command.
 *
 * Context: Link Layer task (HCI command parser)
 *
 * @param cmdbuf Pointer to command buf.
 *
 * @return int BLE_ERR_SUCCESS. Does not return any errors.
 */
static int
ble_ll_hci_set_le_event_mask(uint8_t *cmdbuf)
{
    /* Copy the data into the event mask */
    memcpy(g_ble_ll_hci_le_event_mask, cmdbuf, BLE_HCI_SET_LE_EVENT_MASK_LEN);
    return BLE_ERR_SUCCESS;
}

/**
 * HCI read buffer size command. Returns the ACL data packet length and
 * num data packets.
 *
 * @param rspbuf Pointer to response buffer
 * @param rsplen Length of response buffer
 *
 * @return int BLE error code
 */
static int
ble_ll_hci_le_read_bufsize(uint8_t *rspbuf, uint8_t *rsplen)
{
    /* Place the data packet length and number of packets in the buffer */
    htole16(rspbuf, g_ble_ll_data.ll_acl_pkt_size);
    rspbuf[2] = g_ble_ll_data.ll_num_acl_pkts;
    *rsplen = BLE_HCI_RD_BUF_SIZE_RSPLEN;
    return BLE_ERR_SUCCESS;
}

#if (BLE_LL_CFG_FEAT_DATA_LEN_EXT == 1)
/**
 * HCI write suggested default data length command.
 *
 * This command is used by the host to change the initial max tx octets/time
 * for all connections. Note that if the controller does not support the
 * requested times no error is returned; the controller simply ignores the
 * request (but remembers what the host requested for the read suggested
 * default data length command). The spec allows for the controller to
 * disregard the host.
 *
 * @param rspbuf Pointer to response buffer
 * @param rsplen Length of response buffer
 *
 * @return int BLE error code
 */
static int
ble_ll_hci_le_wr_sugg_data_len(uint8_t *cmdbuf)
{
    int rc;
    uint16_t tx_oct;
    uint16_t tx_time;

    /* Get suggested octets and time */
    tx_oct = le16toh(cmdbuf);
    tx_time = le16toh(cmdbuf + 2);

    /* If valid, write into suggested and change connection initial times */
    if (ble_ll_chk_txrx_octets(tx_oct) && ble_ll_chk_txrx_time(tx_time)) {
        g_ble_ll_conn_params.sugg_tx_octets = (uint8_t)tx_oct;
        g_ble_ll_conn_params.sugg_tx_time = tx_time;

        if ((tx_time <= g_ble_ll_conn_params.supp_max_tx_time) &&
            (tx_oct <= g_ble_ll_conn_params.supp_max_tx_octets)) {
            g_ble_ll_conn_params.conn_init_max_tx_octets = tx_oct;
            g_ble_ll_conn_params.conn_init_max_tx_time = tx_time;
        }
        rc = BLE_ERR_SUCCESS;
    } else {
        rc = BLE_ERR_INV_HCI_CMD_PARMS;
    }

    return rc;
}

/**
 * HCI read suggested default data length command. Returns the controllers
 * initial max tx octet/time.
 *
 * @param rspbuf Pointer to response buffer
 * @param rsplen Length of response buffer
 *
 * @return int BLE error code
 */
static int
ble_ll_hci_le_rd_sugg_data_len(uint8_t *rspbuf, uint8_t *rsplen)
{
    /* Place the data packet length and number of packets in the buffer */
    htole16(rspbuf, g_ble_ll_conn_params.sugg_tx_octets);
    htole16(rspbuf + 2, g_ble_ll_conn_params.sugg_tx_time);
    *rsplen = BLE_HCI_RD_SUGG_DATALEN_RSPLEN;
    return BLE_ERR_SUCCESS;
}
#endif

/**
 * HCI read maximum data length command. Returns the controllers max supported
 * rx/tx octets/times.
 *
 * @param rspbuf Pointer to response buffer
 * @param rsplen Length of response buffer
 *
 * @return int BLE error code
 */
static int
ble_ll_hci_le_rd_max_data_len(uint8_t *rspbuf, uint8_t *rsplen)
{
    /* Place the data packet length and number of packets in the buffer */
    htole16(rspbuf, g_ble_ll_conn_params.supp_max_tx_octets);
    htole16(rspbuf + 2, g_ble_ll_conn_params.supp_max_tx_time);
    htole16(rspbuf + 4, g_ble_ll_conn_params.supp_max_rx_octets);
    htole16(rspbuf + 6, g_ble_ll_conn_params.supp_max_rx_time);
    *rsplen = BLE_HCI_RD_MAX_DATALEN_RSPLEN;
    return BLE_ERR_SUCCESS;
}

/**
 * HCI read local supported features command. Returns the features
 * supported by the controller.
 *
 * @param rspbuf Pointer to response buffer
 * @param rsplen Length of response buffer
 *
 * @return int BLE error code
 */
static int
ble_ll_hci_le_read_local_features(uint8_t *rspbuf, uint8_t *rsplen)
{
    /* Add list of supported features. */
    memset(rspbuf, 0, BLE_HCI_RD_LOC_SUPP_FEAT_RSPLEN);
    rspbuf[0] = ble_ll_read_supp_features();
    *rsplen = BLE_HCI_RD_LOC_SUPP_FEAT_RSPLEN;
    return BLE_ERR_SUCCESS;
}

/**
 * HCI read local supported states command. Returns the states
 * supported by the controller.
 *
 * @param rspbuf Pointer to response buffer
 * @param rsplen Length of response buffer
 *
 * @return int BLE error code
 */
static int
ble_ll_hci_le_read_supp_states(uint8_t *rspbuf, uint8_t *rsplen)
{
    uint64_t supp_states;

    /* Add list of supported states. */
    supp_states = ble_ll_read_supp_states();
    htole64(rspbuf, supp_states);
    *rsplen = BLE_HCI_RD_SUPP_STATES_RSPLEN;
    return BLE_ERR_SUCCESS;
}

/**
 * Checks to see if a LE event has been disabled by the host.
 *
 * @param subev Sub-event code of the LE Meta event. Note that this can
 * be a value from 0 to 63, inclusive.
 *
 * @return uint8_t 0: event is not enabled; otherwise event is enabled.
 */
uint8_t
ble_ll_hci_is_le_event_enabled(int subev)
{
    uint8_t enabled;
    uint8_t bytenum;
    uint8_t bitmask;
    int bitpos;

    /* The LE meta event must be enabled for any LE event to be enabled */
    enabled = 0;
    bitpos = subev - 1;
    if (g_ble_ll_hci_event_mask[7] & 0x20) {
        bytenum = bitpos / 8;
        bitmask = 1 << (bitpos & 0x7);
        enabled = g_ble_ll_hci_le_event_mask[bytenum] & bitmask;
    }

    return enabled;
}

/**
 * Checks to see if an event has been disabled by the host.
 *
 * @param evcode This is the event code for the event (0 - 63).
 *
 * @return uint8_t 0: event is not enabled; otherwise event is enabled.
 */
uint8_t
ble_ll_hci_is_event_enabled(int evcode)
{
    uint8_t enabled;
    uint8_t bytenum;
    uint8_t bitmask;
    int bitpos;

    bitpos = evcode - 1;
    bytenum = bitpos / 8;
    bitmask = 1 << (bitpos & 0x7);
    enabled = g_ble_ll_hci_event_mask[bytenum] & bitmask;

    return enabled;
}

/**
 * Called to determine if the reply to the command should be a command complete
 * event or a command status event.
 *
 * @param ocf
 *
 * @return int 0: return command complete; 1: return command status event
 */
static int
ble_ll_hci_le_cmd_send_cmd_status(uint16_t ocf)
{
    int rc;

    switch (ocf) {
    case BLE_HCI_OCF_LE_RD_REM_FEAT:
    case BLE_HCI_OCF_LE_CREATE_CONN:
    case BLE_HCI_OCF_LE_CONN_UPDATE:
    case BLE_HCI_OCF_LE_START_ENCRYPT:
    case BLE_HCI_OCF_LE_RD_P256_PUBKEY:
    case BLE_HCI_OCF_LE_GEN_DHKEY:
        rc = 1;
        break;
    default:
        rc = 0;
        break;
    }
    return rc;
}

/**
 * Process a LE command sent from the host to the controller. The HCI command
 * has a 3 byte command header followed by data. The header is:
 *  -> opcode (2 bytes)
 *  -> Length of parameters (1 byte; does include command header bytes).
 *
 * @param cmdbuf Pointer to command buffer. Points to start of command header.
 * @param ocf    Opcode command field.
 * @param *rsplen Pointer to length of response
 *
 * @return int  This function returns a BLE error code. If a command status
 *              event should be returned as opposed to command complete,
 *              256 gets added to the return value.
 */
static int
ble_ll_hci_le_cmd_proc(uint8_t *cmdbuf, uint16_t ocf, uint8_t *rsplen)
{
    int rc;
    uint8_t cmdlen;
    uint8_t len;
    uint8_t *rspbuf;

    /* Assume error; if all pass rc gets set to 0 */
    rc = BLE_ERR_INV_HCI_CMD_PARMS;

    /* Get length from command */
    len = cmdbuf[sizeof(uint16_t)];

    /* Check the length to make sure it is valid */
    cmdlen = g_ble_hci_le_cmd_len[ocf];
    if ((cmdlen != 0xFF) && (len != cmdlen)) {
        goto ll_hci_le_cmd_exit;
    }

    /*
     * The command response pointer points into the same buffer as the
     * command data itself. That is fine, as each command reads all the data
     * before crafting a response.
     */
    rspbuf = cmdbuf + BLE_HCI_EVENT_CMD_COMPLETE_MIN_LEN;

    /* Move past HCI command header */
    cmdbuf += BLE_HCI_CMD_HDR_LEN;

    switch (ocf) {
    case BLE_HCI_OCF_LE_SET_EVENT_MASK:
        rc = ble_ll_hci_set_le_event_mask(cmdbuf);
        break;
    case BLE_HCI_OCF_LE_RD_BUF_SIZE:
        rc = ble_ll_hci_le_read_bufsize(rspbuf, rsplen);
        break;
    case BLE_HCI_OCF_LE_RD_LOC_SUPP_FEAT:
        rc = ble_ll_hci_le_read_local_features(rspbuf, rsplen);
        break;
    case BLE_HCI_OCF_LE_SET_RAND_ADDR:
        rc = ble_ll_set_random_addr(cmdbuf);
        break;
    case BLE_HCI_OCF_LE_SET_ADV_PARAMS:
        /* Length should be one byte */
        rc = ble_ll_adv_set_adv_params(cmdbuf);
        break;
    case BLE_HCI_OCF_LE_RD_ADV_CHAN_TXPWR:
        rc = ble_ll_adv_read_txpwr(rspbuf, rsplen);
        break;
    case BLE_HCI_OCF_LE_SET_ADV_DATA:
        if (len > 0) {
            --len;
            rc = ble_ll_adv_set_adv_data(cmdbuf, len);
        }
        break;
    case BLE_HCI_OCF_LE_SET_SCAN_RSP_DATA:
        if (len > 0) {
            --len;
            rc = ble_ll_adv_set_scan_rsp_data(cmdbuf, len);
        }
        break;
    case BLE_HCI_OCF_LE_SET_ADV_ENABLE:
        /* Length should be one byte */
        rc = ble_ll_adv_set_enable(cmdbuf);
        break;
    case BLE_HCI_OCF_LE_SET_SCAN_ENABLE:
        rc = ble_ll_scan_set_enable(cmdbuf);
        break;
    case BLE_HCI_OCF_LE_SET_SCAN_PARAMS:
        rc = ble_ll_scan_set_scan_params(cmdbuf);
        break;
    case BLE_HCI_OCF_LE_CREATE_CONN:
        rc = ble_ll_conn_create(cmdbuf);
        break;
    case BLE_HCI_OCF_LE_CREATE_CONN_CANCEL:
        rc = ble_ll_conn_create_cancel();
        break;
    case BLE_HCI_OCF_LE_CLEAR_WHITE_LIST:
        rc = ble_ll_whitelist_clear();
        break;
    case BLE_HCI_OCF_LE_RD_WHITE_LIST_SIZE:
        rc = ble_ll_whitelist_read_size(rspbuf, rsplen);
        break;
    case BLE_HCI_OCF_LE_ADD_WHITE_LIST:
        rc = ble_ll_whitelist_add(cmdbuf + 1, cmdbuf[0]);
        break;
    case BLE_HCI_OCF_LE_RMV_WHITE_LIST:
        rc = ble_ll_whitelist_rmv(cmdbuf + 1, cmdbuf[0]);
        break;
    case BLE_HCI_OCF_LE_CONN_UPDATE:
        rc = ble_ll_conn_hci_update(cmdbuf);
        break;
    case BLE_HCI_OCF_LE_SET_HOST_CHAN_CLASS:
        rc = ble_ll_conn_hci_set_chan_class(cmdbuf);
        break;
    case BLE_HCI_OCF_LE_RD_CHAN_MAP:
        rc = ble_ll_conn_hci_rd_chan_map(cmdbuf, rspbuf, rsplen);
        break;
    case BLE_HCI_OCF_LE_RD_REM_FEAT:
        rc = ble_ll_conn_hci_read_rem_features(cmdbuf);
        break;
#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
    case BLE_HCI_OCF_LE_ENCRYPT:
        rc = ble_ll_hci_le_encrypt(cmdbuf, rspbuf, rsplen);
        break;
#endif
    case BLE_HCI_OCF_LE_RAND:
        rc = ble_ll_hci_le_rand(rspbuf, rsplen);
        break;
#if (BLE_LL_CFG_FEAT_LE_ENCRYPTION == 1)
    case BLE_HCI_OCF_LE_START_ENCRYPT:
        rc = ble_ll_conn_hci_le_start_encrypt(cmdbuf);
        break;
    case BLE_HCI_OCF_LE_LT_KEY_REQ_REPLY:
    case BLE_HCI_OCF_LE_LT_KEY_REQ_NEG_REPLY:
        rc = ble_ll_conn_hci_le_ltk_reply(cmdbuf, rspbuf, ocf);
        *rsplen = sizeof(uint16_t);
        break;
#endif
    case BLE_HCI_OCF_LE_RD_SUPP_STATES :
        rc = ble_ll_hci_le_read_supp_states(rspbuf, rsplen);
        break;
    case BLE_HCI_OCF_LE_REM_CONN_PARAM_NRR:
        rc = ble_ll_conn_hci_param_reply(cmdbuf, 0);
        break;
    case BLE_HCI_OCF_LE_REM_CONN_PARAM_RR:
        rc = ble_ll_conn_hci_param_reply(cmdbuf, 1);
        break;
#if (BLE_LL_CFG_FEAT_DATA_LEN_EXT == 1)
    case BLE_HCI_OCF_LE_SET_DATA_LEN:
        rc = ble_ll_conn_hci_set_data_len(cmdbuf, rspbuf, rsplen);
        break;
    case BLE_HCI_OCF_LE_RD_SUGG_DEF_DATA_LEN:
        rc = ble_ll_hci_le_rd_sugg_data_len(rspbuf, rsplen);
        break;
    case BLE_HCI_OCF_LE_WR_SUGG_DEF_DATA_LEN:
        rc = ble_ll_hci_le_wr_sugg_data_len(cmdbuf);
        break;
#endif
    case BLE_HCI_OCF_LE_RD_MAX_DATA_LEN:
        rc = ble_ll_hci_le_rd_max_data_len(rspbuf, rsplen);
        break;
    default:
        rc = BLE_ERR_UNKNOWN_HCI_CMD;
        break;
    }

    /*
     * This code is here because we add 256 to the return code to denote
     * that the reply to this command should be command status (as opposed to
     * command complete).
     */
ll_hci_le_cmd_exit:
    if (ble_ll_hci_le_cmd_send_cmd_status(ocf)) {
        rc += (BLE_ERR_MAX + 1);
    }

    return rc;
}

/**
 * Process a link control command sent from the host to the controller. The HCI
 * command has a 3 byte command header followed by data. The header is:
 *  -> opcode (2 bytes)
 *  -> Length of parameters (1 byte; does include command header bytes).
 *
 * @param cmdbuf Pointer to command buffer. Points to start of command header.
 * @param ocf    Opcode command field.
 * @param *rsplen Pointer to length of response
 *
 * @return int  This function returns a BLE error code. If a command status
 *              event should be returned as opposed to command complete,
 *              256 gets added to the return value.
 */
static int
ble_ll_hci_link_ctrl_cmd_proc(uint8_t *cmdbuf, uint16_t ocf, uint8_t *rsplen)
{
    int rc;
    uint8_t len;

    /* Assume error; if all pass rc gets set to 0 */
    rc = BLE_ERR_INV_HCI_CMD_PARMS;

    /* Get length from command */
    len = cmdbuf[sizeof(uint16_t)];

    /* Move past HCI command header */
    cmdbuf += BLE_HCI_CMD_HDR_LEN;

    switch (ocf) {
    case BLE_HCI_OCF_DISCONNECT_CMD:
        if (len == BLE_HCI_DISCONNECT_CMD_LEN) {
            rc = ble_ll_conn_hci_disconnect_cmd(cmdbuf);
        }
        /* Send command status instead of command complete */
        rc += (BLE_ERR_MAX + 1);
        break;

    case BLE_HCI_OCF_RD_REM_VER_INFO:
        if (len == sizeof(uint16_t)) {
            rc = ble_ll_conn_hci_rd_rem_ver_cmd(cmdbuf);
        }
        /* Send command status instead of command complete */
        rc += (BLE_ERR_MAX + 1);
        break;

    default:
        rc = BLE_ERR_UNKNOWN_HCI_CMD;
        break;
    }

    return rc;
}

static int
ble_ll_hci_ctlr_bb_cmd_proc(uint8_t *cmdbuf, uint16_t ocf, uint8_t *rsplen)
{
    int rc;
    uint8_t len;

    /* Assume error; if all pass rc gets set to 0 */
    rc = BLE_ERR_INV_HCI_CMD_PARMS;

    /* Get length from command */
    len = cmdbuf[sizeof(uint16_t)];

    /* Move past HCI command header */
    cmdbuf += BLE_HCI_CMD_HDR_LEN;

    switch (ocf) {
    case BLE_HCI_OCF_CB_SET_EVENT_MASK:
        if (len == BLE_HCI_SET_EVENT_MASK_LEN) {
            memcpy(g_ble_ll_hci_event_mask, cmdbuf, len);
            rc = BLE_ERR_SUCCESS;
        }
        break;
    case BLE_HCI_OCF_CB_RESET:
        if (len == 0) {
            rc = ble_ll_reset();
        }
        break;
    default:
        rc = BLE_ERR_UNKNOWN_HCI_CMD;
        break;
    }

    return rc;
}

static int
ble_ll_hci_info_params_cmd_proc(uint8_t *cmdbuf, uint16_t ocf, uint8_t *rsplen)
{
    int rc;
    uint8_t len;
    uint8_t *rspbuf;

    /* Assume error; if all pass rc gets set to 0 */
    rc = BLE_ERR_INV_HCI_CMD_PARMS;

    /* Get length from command */
    len = cmdbuf[sizeof(uint16_t)];

    /*
     * The command response pointer points into the same buffer as the
     * command data itself. That is fine, as each command reads all the data
     * before crafting a response.
     */
    rspbuf = cmdbuf + BLE_HCI_EVENT_CMD_COMPLETE_MIN_LEN;

    /* Move past HCI command header */
    cmdbuf += BLE_HCI_CMD_HDR_LEN;

    switch (ocf) {
    case BLE_HCI_OCF_IP_RD_LOCAL_VER:
        if (len == 0) {
            rc = ble_ll_hci_rd_local_version(rspbuf, rsplen);
        }
        break;
    case BLE_HCI_OCF_IP_RD_LOC_SUPP_CMD:
        if (len == 0) {
            rc = ble_ll_hci_rd_local_supp_cmd(rspbuf, rsplen);
        }
        break;
    case BLE_HCI_OCF_IP_RD_LOC_SUPP_FEAT:
        if (len == 0) {
            rc = ble_ll_hci_rd_local_supp_feat(rspbuf, rsplen);
        }
        break;
    case BLE_HCI_OCF_IP_RD_BD_ADDR:
        if (len == 0) {
            rc = ble_ll_hci_rd_bd_addr(rspbuf, rsplen);
        }
        break;
    default:
        rc = BLE_ERR_UNKNOWN_HCI_CMD;
        break;
    }

    return rc;
}

static int
ble_ll_hci_status_params_cmd_proc(uint8_t *cmdbuf, uint16_t ocf, uint8_t *rsplen)
{
    int rc;
    uint8_t len;
    uint8_t *rspbuf;

    /* Assume error; if all pass rc gets set to 0 */
    rc = BLE_ERR_INV_HCI_CMD_PARMS;

    /* Get length from command */
    len = cmdbuf[sizeof(uint16_t)];

    /*
     * The command response pointer points into the same buffer as the
     * command data itself. That is fine, as each command reads all the data
     * before crafting a response.
     */
    rspbuf = cmdbuf + BLE_HCI_EVENT_CMD_COMPLETE_MIN_LEN;

    /* Move past HCI command header */
    cmdbuf += BLE_HCI_CMD_HDR_LEN;

    switch (ocf) {
    case BLE_HCI_OCF_RD_RSSI:
        if (len == sizeof(uint16_t)) {
            rc = ble_ll_conn_hci_rd_rssi(cmdbuf, rspbuf, rsplen);
        }
        break;
    default:
        rc = BLE_ERR_UNKNOWN_HCI_CMD;
        break;
    }

    return rc;
}

/**
 * Called to process an HCI command from the host.
 *
 * @param ev Pointer to os event containing a pointer to command buffer
 */
void
ble_ll_hci_cmd_proc(struct os_event *ev)
{
    int rc;
    uint8_t ogf;
    uint8_t rsplen;
    uint8_t *cmdbuf;
    uint16_t opcode;
    uint16_t ocf;
    os_error_t err;

    /* The command buffer is the event argument */
    cmdbuf = (uint8_t *)ev->ev_arg;
    assert(cmdbuf != NULL);

    /* Free the event */
    err = os_memblock_put(&g_hci_os_event_pool, ev);
    assert(err == OS_OK);

    /* Get the opcode from the command buffer */
    opcode = le16toh(cmdbuf);
    ocf = BLE_HCI_OCF(opcode);
    ogf = BLE_HCI_OGF(opcode);

    /* Assume response length is zero */
    rsplen = 0;

    switch (ogf) {
    case BLE_HCI_OGF_LINK_CTRL:
        rc = ble_ll_hci_link_ctrl_cmd_proc(cmdbuf, ocf, &rsplen);
        break;
    case BLE_HCI_OGF_CTLR_BASEBAND:
        rc = ble_ll_hci_ctlr_bb_cmd_proc(cmdbuf, ocf, &rsplen);
        break;
    case BLE_HCI_OGF_INFO_PARAMS:
        rc = ble_ll_hci_info_params_cmd_proc(cmdbuf, ocf, &rsplen);
        break;
    case BLE_HCI_OGF_STATUS_PARAMS:
        rc = ble_ll_hci_status_params_cmd_proc(cmdbuf, ocf, &rsplen);
        break;
    case BLE_HCI_OGF_LE:
        rc = ble_ll_hci_le_cmd_proc(cmdbuf, ocf, &rsplen);
        break;
    default:
        /* XXX: Need to support other OGF. For now, return unsupported */
        rc = BLE_ERR_UNKNOWN_HCI_CMD;
        break;
    }

    /* If no response is generated, we free the buffers */
    assert(rc >= 0);
    if (rc <= BLE_ERR_MAX) {
        /* Create a command complete event with status from command */
        cmdbuf[0] = BLE_HCI_EVCODE_COMMAND_COMPLETE;
        cmdbuf[1] = 4 + rsplen;
        cmdbuf[2] = ble_ll_hci_get_num_cmd_pkts();
        htole16(cmdbuf + 3, opcode);
        cmdbuf[5] = (uint8_t)rc;
    } else {
        /* Create a command status event */
        rc -= (BLE_ERR_MAX + 1);
        cmdbuf[0] = BLE_HCI_EVCODE_COMMAND_STATUS;
        cmdbuf[1] = 4;
        cmdbuf[2] = (uint8_t)rc;
        cmdbuf[3] = ble_ll_hci_get_num_cmd_pkts();
        htole16(cmdbuf + 4, opcode);
    }

    /* Count commands and those in error */
    if (rc) {
        STATS_INC(ble_ll_stats, hci_cmd_errs);
    } else {
        STATS_INC(ble_ll_stats, hci_cmds);
    }

    /* Send the event (events cannot be masked) */
    ble_ll_hci_event_send(cmdbuf);
}

/* XXX: For now, put this here */
int
ble_hci_transport_host_cmd_send(uint8_t *cmd)
{
    os_error_t err;
    struct os_event *ev;

    /* Get an event structure off the queue */
    ev = (struct os_event *)os_memblock_get(&g_hci_os_event_pool);
    if (!ev) {
        err = os_memblock_put(&g_hci_cmd_pool, cmd);
        assert(err == OS_OK);
        return -1;
    }

    /* Fill out the event and post to Link Layer */
    ev->ev_queued = 0;
    ev->ev_type = BLE_LL_EVENT_HCI_CMD;
    ev->ev_arg = cmd;
    os_eventq_put(&g_ble_ll_data.ll_evq, ev);

    return 0;
}

/* Send ACL data from host to contoller */
int
ble_hci_transport_host_acl_data_send(struct os_mbuf *om)
{
    ble_ll_acl_data_in(om);
    return 0;
}

/**
 * Initalize the LL HCI.
 *
 * NOTE: This function is called by the HCI RESET command so if any code
 * is added here it must be OK to be executed when the reset command is used.
 */
void
ble_ll_hci_init(void)
{
    /* Set defaults for LE events: Vol 2 Part E 7.8.1 */
    g_ble_ll_hci_le_event_mask[0] = 0x1f;

    /* Set defaults for controller/baseband events: Vol 2 Part E 7.3.1 */
    g_ble_ll_hci_event_mask[0] = 0xff;
    g_ble_ll_hci_event_mask[1] = 0xff;
    g_ble_ll_hci_event_mask[2] = 0xff;
    g_ble_ll_hci_event_mask[3] = 0xff;
    g_ble_ll_hci_event_mask[4] = 0xff;
    g_ble_ll_hci_event_mask[5] = 0x1f;
}
