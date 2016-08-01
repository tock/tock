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

#ifndef H_BLE_
#define H_BLE_

#include <inttypes.h>
/* XXX: some or all of these should not be here */
#include "os/os.h"

/* BLE encryption block definitions */
#define BLE_ENC_BLOCK_SIZE       (16)

struct ble_encryption_block
{
    uint8_t     key[BLE_ENC_BLOCK_SIZE];
    uint8_t     plain_text[BLE_ENC_BLOCK_SIZE];
    uint8_t     cipher_text[BLE_ENC_BLOCK_SIZE];
};

/* Shared command pool for transort between host and controller */
extern struct os_mempool g_hci_cmd_pool;
extern struct os_mempool g_hci_os_event_pool;

/*
 * BLE MBUF structure:
 *
 * The BLE mbuf structure is as follows. Note that this structure applies to
 * the packet header mbuf (not mbufs that are part of a "packet chain"):
 *      struct os_mbuf          (16)
 *      struct os_mbuf_pkthdr   (8)
 *      struct ble_mbuf_hdr     (8)
 *      Data buffer             (BLE_MBUF_PAYLOAD_SIZE)
 *
 * The BLE mbuf header contains the following:
 *  flags: bitfield with the following values
 *      0x01:   Set if there was a match on the whitelist
 *      0x02:   Set if a connect request was transmitted upon receiving pdu
 *      0x04:   Set the first time we transmit the PDU (used to detect retry).
 *  channel: The logical BLE channel PHY channel # (0 - 39)
 *  crcok: flag denoting CRC check passed (1) or failed (0).
 *  rssi: RSSI, in dBm.
 */
struct ble_mbuf_hdr_rxinfo
{
    uint8_t flags;
    uint8_t channel;
    uint8_t handle;
    int8_t rssi;
};

/* Flag definitions for rxinfo  */
#define BLE_MBUF_HDR_F_CRC_OK           (0x80)
#define BLE_MBUF_HDR_F_DEVMATCH         (0x40)
#define BLE_MBUF_HDR_F_MIC_FAILURE      (0x20)
#define BLE_MBUF_HDR_F_SCAN_RSP_TXD     (0x10)
#define BLE_MBUF_HDR_F_SCAN_RSP_CHK     (0x08)
#define BLE_MBUF_HDR_F_RXSTATE_MASK     (0x07)

/* Transmit info. NOTE: no flags defined */
struct ble_mbuf_hdr_txinfo
{
    uint8_t flags;
    uint8_t offset;
    uint8_t pyld_len;
    uint8_t hdr_byte;
};

struct ble_mbuf_hdr
{
    union {
        struct ble_mbuf_hdr_rxinfo rxinfo;
        struct ble_mbuf_hdr_txinfo txinfo;
    };
    uint32_t beg_cputime;
};

/*
 * The payload size for BLE MBUFs. NOTE: this needs to accommodate a max size
 * PHY pdu of 257 bytes.
 */
#define BLE_MBUF_PAYLOAD_SIZE           (260)

#define BLE_MBUF_HDR_CRC_OK(hdr)        \
    ((hdr)->rxinfo.flags & BLE_MBUF_HDR_F_CRC_OK)

#define BLE_MBUF_HDR_MIC_FAILURE(hdr)   \
    ((hdr)->rxinfo.flags & BLE_MBUF_HDR_F_MIC_FAILURE)

#define BLE_MBUF_HDR_RX_STATE(hdr)      \
    ((hdr)->rxinfo.flags & BLE_MBUF_HDR_F_RXSTATE_MASK)

#define BLE_MBUF_HDR_PTR(om)            \
    (struct ble_mbuf_hdr *)((uint8_t *)om + sizeof(struct os_mbuf) + \
                            sizeof(struct os_mbuf_pkthdr))

/* BLE mbuf overhead per packet header mbuf */
#define BLE_MBUF_PKTHDR_OVERHEAD        \
    (sizeof(struct os_mbuf_pkthdr) + sizeof(struct ble_mbuf_hdr))

#define BLE_MBUF_MEMBLOCK_OVERHEAD      \
    (sizeof(struct os_mbuf) + BLE_MBUF_PKTHDR_OVERHEAD)

#define BLE_DEV_ADDR_LEN        (6)
extern uint8_t g_dev_addr[BLE_DEV_ADDR_LEN];
extern uint8_t g_random_addr[BLE_DEV_ADDR_LEN];

#undef htole16
#undef htole32
#undef htole64
#undef le16toh
#undef le32toh
#undef le64toh
#undef htobe16
#undef htobe32
#undef htobe64
#undef be16toh
#undef be32toh
#undef be64toh
void htole16(void *buf, uint16_t x);
void htole32(void *buf, uint32_t x);
void htole64(void *buf, uint64_t x);
uint16_t le16toh(void *buf);
uint32_t le32toh(void *buf);
uint64_t le64toh(void *buf);
void htobe16(void *buf, uint16_t x);
void htobe32(void *buf, uint32_t x);
void htobe64(void *buf, uint64_t x);
void swap_in_place(void *buf, int len);
void swap_buf(uint8_t *dst, uint8_t *src, int len);
/* XXX */

/* BLE Error Codes (Core v4.2 Vol 2 part D) */
enum ble_error_codes
{
    /* An "error" code of 0 means success */
    BLE_ERR_SUCCESS             = 0,
    BLE_ERR_UNKNOWN_HCI_CMD     = 1,
    BLE_ERR_UNK_CONN_ID         = 2,
    BLE_ERR_HW_FAIL             = 3,
    BLE_ERR_PAGE_TMO            = 4,
    BLE_ERR_AUTH_FAIL           = 5,
    BLE_ERR_PINKEY_MISSING      = 6,
    BLE_ERR_MEM_CAPACITY        = 7,
    BLE_ERR_CONN_SPVN_TMO       = 8,
    BLE_ERR_CONN_LIMIT          = 9,
    BLE_ERR_SYNCH_CONN_LIMIT    = 10,
    BLE_ERR_ACL_CONN_EXISTS     = 11,
    BLE_ERR_CMD_DISALLOWED      = 12,
    BLE_ERR_CONN_REJ_RESOURCES  = 13,
    BLE_ERR_CONN_REJ_SECURITY   = 14,
    BLE_ERR_CONN_REJ_BD_ADDR    = 15,
    BLE_ERR_CONN_ACCEPT_TMO     = 16,
    BLE_ERR_UNSUPPORTED         = 17,
    BLE_ERR_INV_HCI_CMD_PARMS   = 18,
    BLE_ERR_REM_USER_CONN_TERM  = 19,
    BLE_ERR_RD_CONN_TERM_RESRCS = 20,
    BLE_ERR_RD_CONN_TERM_PWROFF = 21,
    BLE_ERR_CONN_TERM_LOCAL     = 22,
    BLE_ERR_REPEATED_ATTEMPTS   = 23,
    BLE_ERR_NO_PAIRING          = 24,
    BLE_ERR_UNK_LMP             = 25,
    BLE_ERR_UNSUPP_REM_FEATURE  = 26,
    BLE_ERR_SCO_OFFSET          = 27,
    BLE_ERR_SCO_ITVL            = 28,
    BLE_ERR_SCO_AIR_MODE        = 29,
    BLE_ERR_INV_LMP_LL_PARM     = 30,
    BLE_ERR_UNSPECIFIED         = 31,
    BLE_ERR_UNSUPP_LMP_LL_PARM  = 32,
    BLE_ERR_NO_ROLE_CHANGE      = 33,
    BLE_ERR_LMP_LL_RSP_TMO      = 34,
    BLE_ERR_LMP_COLLISION       = 35,
    BLE_ERR_LMP_PDU             = 36,
    BLE_ERR_ENCRYPTION_MODE     = 37,
    BLE_ERR_LINK_KEY_CHANGE     = 38,
    BLE_ERR_UNSUPP_QOS          = 39,
    BLE_ERR_INSTANT_PASSED      = 40,
    BLE_ERR_UNIT_KEY_PAIRING    = 41,
    BLE_ERR_DIFF_TRANS_COLL     = 42,
    /* BLE_ERR_RESERVED         = 43 */
    BLE_ERR_QOS_PARM            = 44,
    BLE_ERR_QOS_REJECTED        = 45,
    BLE_ERR_CHAN_CLASS          = 46,
    BLE_ERR_INSUFFICIENT_SEC    = 47,
    BLE_ERR_PARM_OUT_OF_RANGE   = 48,
    /* BLE_ERR_RESERVED         = 49 */
    BLE_ERR_PENDING_ROLE_SW     = 50,
    /* BLE_ERR_RESERVED         = 51 */
    BLE_ERR_RESERVED_SLOT       = 52,
    BLE_ERR_ROLE_SW_FAIL        = 53,
    BLE_ERR_INQ_RSP_TOO_BIG     = 54,
    BLE_ERR_SEC_SIMPLE_PAIR     = 55,
    BLE_ERR_HOST_BUSY_PAIR      = 56,
    BLE_ERR_CONN_REJ_CHANNEL    = 57,
    BLE_ERR_CTLR_BUSY           = 58,
    BLE_ERR_CONN_PARMS          = 59,
    BLE_ERR_DIR_ADV_TMO         = 60,
    BLE_ERR_CONN_TERM_MIC       = 61,
    BLE_ERR_CONN_ESTABLISHMENT  = 62,
    BLE_ERR_MAC_CONN_FAIL       = 63,
    BLE_ERR_COARSE_CLK_ADJ      = 64,
    BLE_ERR_ATTR_NOT_FOUND      = 65,
    BLE_ERR_MAX                 = 255
};

/* Address types */
#define BLE_ADDR_TYPE_PUBLIC    (0)
#define BLE_ADDR_TYPE_RANDOM    (1)

#endif /* H_BLE_ */
