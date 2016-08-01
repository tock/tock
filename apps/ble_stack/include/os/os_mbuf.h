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

#ifndef _OS_MBUF_H 
#define _OS_MBUF_H 

#include "os/queue.h"
#include "os/os_eventq.h"

/**
 * A mbuf pool from which to allocate mbufs. This contains a pointer to the os 
 * mempool to allocate mbufs out of, the total number of elements in the pool, 
 * and the amount of "user" data in a non-packet header mbuf. The total pool 
 * size, in bytes, should be: 
 *  os_mbuf_count * (omp_databuf_len + sizeof(struct os_mbuf))
 */
struct os_mbuf_pool {
    /** 
     * Total length of the databuf in each mbuf.  This is the size of the 
     * mempool block, minus the mbuf header
     */
    uint16_t omp_databuf_len;
    /**
     * Total number of memblock's allocated in this mempool.
     */
    uint16_t omp_mbuf_count;
    /**
     * The memory pool which to allocate mbufs out of 
     */
    struct os_mempool *omp_pool;

    /**
     * Link to the next mbuf pool for system memory pools.
     */
    STAILQ_ENTRY(os_mbuf_pool) omp_next;
};


/**
 * A packet header structure that preceeds the mbuf packet headers.
 */
struct os_mbuf_pkthdr {
    /**
     * Overall length of the packet. 
     */
    uint16_t omp_len;
    /**
     * Flags
     */
    uint16_t omp_flags;
    /**
     * Next packet in the mbuf chain.
     */
    STAILQ_ENTRY(os_mbuf_pkthdr) omp_next;
};

/**
 * Chained memory buffer.
 */
struct os_mbuf {
    /**
     * Current pointer to data in the structure
     */
    uint8_t *om_data;
    /**
     * Flags associated with this buffer, see OS_MBUF_F_* defintions
     */
    uint8_t om_flags;
    /**
     * Length of packet header
     */
    uint8_t om_pkthdr_len;
    /**
     * Length of data in this buffer 
     */
    uint16_t om_len;

    /**
     * The mbuf pool this mbuf was allocated out of 
     */
    struct os_mbuf_pool *om_omp;

    /**
     * Pointer to next entry in the chained memory buffer
     */
    SLIST_ENTRY(os_mbuf) om_next;

    /**
     * Pointer to the beginning of the data, after this buffer
     */
    uint8_t om_databuf[0];
};

struct os_mqueue {
    STAILQ_HEAD(, os_mbuf_pkthdr) mq_head;
    struct os_event mq_ev;
};

/*
 * Given a flag number, provide the mask for it
 *
 * @param __n The number of the flag in the mask 
 */
#define OS_MBUF_F_MASK(__n) (1 << (__n))

/* 
 * Checks whether a given mbuf is a packet header mbuf 
 *
 * @param __om The mbuf to check 
 */
#define OS_MBUF_IS_PKTHDR(__om) \
    ((__om)->om_pkthdr_len >= sizeof (struct os_mbuf_pkthdr))

/* Get a packet header pointer given an mbuf pointer */
#define OS_MBUF_PKTHDR(__om) ((struct os_mbuf_pkthdr *)     \
    ((uint8_t *)&(__om)->om_data + sizeof(struct os_mbuf)))

/* Given a mbuf packet header pointer, return a pointer to the mbuf */
#define OS_MBUF_PKTHDR_TO_MBUF(__hdr)   \
     (struct os_mbuf *)((uint8_t *)(__hdr) - sizeof(struct os_mbuf))

/**
 * Gets the length of an entire mbuf chain.  The specified mbuf must have a
 * packet header.
 */
#define OS_MBUF_PKTLEN(__om) (OS_MBUF_PKTHDR(__om)->omp_len)

/*
 * Access the data of a mbuf, and cast it to type
 *
 * @param __om The mbuf to access, and cast 
 * @param __type The type to cast it to 
 */
#define OS_MBUF_DATA(__om, __type) \
     (__type) ((__om)->om_data)

/**
 * Access the "user header" in the head of an mbuf chain.
 *
 * @param om                    Pointer to the head of an mbuf chain.
 */
#define OS_MBUF_USRHDR(om)                              \
    (void *)((uint8_t *)om + sizeof (struct os_mbuf) +  \
             sizeof (struct os_mbuf_pkthdr))

/**
 * Retrieves the length of the user header in an mbuf.
 *
 * @param om                    Pointer to the mbuf to query.
 */
#define OS_MBUF_USRHDR_LEN(om) \
    ((om)->om_pkthdr_len - sizeof (struct os_mbuf_pkthdr))

/*
 * Called by OS_MBUF_LEADINGSPACE() macro
 */
static inline uint16_t 
_os_mbuf_leadingspace(struct os_mbuf *om)
{
    uint16_t startoff;
    uint16_t leadingspace;

    startoff = 0;
    if (OS_MBUF_IS_PKTHDR(om)) {
        startoff = om->om_pkthdr_len;
    }

    leadingspace = (uint16_t) (OS_MBUF_DATA(om, uint8_t *) - 
        ((uint8_t *) &om->om_databuf[0] + startoff));

    return (leadingspace);
}

/**
 * Returns the leading space (space at the beginning) of the mbuf. 
 * Works on both packet header, and regular mbufs, as it accounts 
 * for the additional space allocated to the packet header.
 * 
 * @param __omp Is the mbuf pool (which contains packet header length.)
 * @param __om  Is the mbuf in that pool to get the leadingspace for 
 *
 * @return Amount of leading space available in the mbuf 
 */
#define OS_MBUF_LEADINGSPACE(__om) _os_mbuf_leadingspace(__om)

/* Called by OS_MBUF_TRAILINGSPACE() macro. */
static inline uint16_t 
_os_mbuf_trailingspace(struct os_mbuf *om)
{
    struct os_mbuf_pool *omp;

    omp = om->om_omp;

    return (&om->om_databuf[0] + omp->omp_databuf_len) -
      (om->om_data + om->om_len);
}

/**
 * Returns the trailing space (space at the end) of the mbuf.
 * Works on both packet header and regular mbufs.
 *
 * @param __omp The mbuf pool for this mbuf 
 * @param __om  Is the mbuf in that pool to get trailing space for 
 *
 * @return The amount of trailing space available in the mbuf 
 */
#define OS_MBUF_TRAILINGSPACE(__om) _os_mbuf_trailingspace(__om)

/* Mbuf queue functions */

/* Initialize a mbuf queue */
int os_mqueue_init(struct os_mqueue *, void *arg);

/* Get an element from a mbuf queue */
struct os_mbuf *os_mqueue_get(struct os_mqueue *);

/* Put an element in a mbuf queue */
int os_mqueue_put(struct os_mqueue *, struct os_eventq *, struct os_mbuf *);

/* Register an mbuf pool with the system pool registry */
int os_msys_register(struct os_mbuf_pool *);

/* Return a mbuf from the system pool, given an indicative mbuf size */
struct os_mbuf *os_msys_get(uint16_t dsize, uint16_t leadingspace);

/* De-registers all mbuf pools from msys. */
void os_msys_reset(void);

/* Return a packet header mbuf from the system pool */
struct os_mbuf *os_msys_get_pkthdr(uint16_t dsize, uint16_t user_hdr_len);

/* Initialize a mbuf pool */
int os_mbuf_pool_init(struct os_mbuf_pool *, struct os_mempool *mp, 
        uint16_t, uint16_t);

/* Allocate a new mbuf out of the os_mbuf_pool */ 
struct os_mbuf *os_mbuf_get(struct os_mbuf_pool *omp, uint16_t);

/* Allocate a new packet header mbuf out of the os_mbuf_pool */ 
struct os_mbuf *os_mbuf_get_pkthdr(struct os_mbuf_pool *omp, 
        uint8_t pkthdr_len);

/* Duplicate a mbuf from the pool */
struct os_mbuf *os_mbuf_dup(struct os_mbuf *m);

struct os_mbuf * os_mbuf_off(struct os_mbuf *om, int off, int *out_off);

/* Copy data from an mbuf to a flat buffer. */
int os_mbuf_copydata(const struct os_mbuf *m, int off, int len, void *dst);

/* Append data onto a mbuf */
int os_mbuf_append(struct os_mbuf *m, const void *, uint16_t);

/* Free a mbuf */
int os_mbuf_free(struct os_mbuf *mb);

/* Free a mbuf chain */
int os_mbuf_free_chain(struct os_mbuf *om);

void os_mbuf_adj(struct os_mbuf *mp, int req_len);
int os_mbuf_memcmp(const struct os_mbuf *om, int off, const void *data,
                   int len);

struct os_mbuf *os_mbuf_prepend(struct os_mbuf *om, int len);
int os_mbuf_copyinto(struct os_mbuf *om, int off, const void *src, int len);
void os_mbuf_concat(struct os_mbuf *first, struct os_mbuf *second);
void *os_mbuf_extend(struct os_mbuf *om, uint16_t len);
struct os_mbuf *os_mbuf_pullup(struct os_mbuf *om, uint16_t len);

#endif /* _OS_MBUF_H */ 
