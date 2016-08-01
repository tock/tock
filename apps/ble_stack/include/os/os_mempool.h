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

#ifndef _OS_MEMPOOL_H_
#define _OS_MEMPOOL_H_

#include "os/os.h"
#include "os/queue.h"

/* 
 * A memory block structure. This simply contains a pointer to the free list 
 * chain and is only used when the block is on the free list. When the block 
 * has been removed from the free list the entire memory block is usable by the 
 * caller. 
 */
struct os_memblock {
    SLIST_ENTRY(os_memblock) mb_next;
};

/* XXX: Change this structure so that we keep the first address in the pool? */
/* XXX: add memory debug structure and associated code */
/* XXX: Change how I coded the SLIST_HEAD here. It should be named:
   SLIST_HEAD(,os_memblock) mp_head; */

/* Memory pool */
struct os_mempool {
    int mp_block_size;          /* Size of the memory blocks, in bytes. */
    int mp_num_blocks;          /* The number of memory blocks. */
    int mp_num_free;            /* The number of free blocks left */
    uint32_t mp_membuf_addr;    /* Address of memory buffer used by pool */
    STAILQ_ENTRY(os_mempool) mp_list;
    SLIST_HEAD(,os_memblock);   /* Pointer to list of free blocks */
    char *name;                 /* Name for memory block */
};

#define OS_MEMPOOL_INFO_NAME_LEN (32)

struct os_mempool_info {
    int omi_block_size;
    int omi_num_blocks;
    int omi_num_free;
    char omi_name[OS_MEMPOOL_INFO_NAME_LEN];
};

struct os_mempool *os_mempool_info_get_next(struct os_mempool *, 
        struct os_mempool_info *);

/* 
 * To calculate size of the memory buffer needed for the pool. NOTE: This size 
 * is NOT in bytes! The size is the number of os_membuf_t elements required for 
 * the memory pool.
 */
#if (OS_CFG_ALIGNMENT == OS_CFG_ALIGN_4)
#define OS_MEMPOOL_SIZE(n,blksize)      ((((blksize) + 3) / 4) * (n))
typedef uint32_t os_membuf_t;
#else
#define OS_MEMPOOL_SIZE(n,blksize)      ((((blksize) + 7) / 8) * (n)) 
typedef uint64_t os_membuf_t;
#endif

/** Calculates the number of bytes required to initialize a memory pool. */
#define OS_MEMPOOL_BYTES(n,blksize)     \
    (sizeof (os_membuf_t) * OS_MEMPOOL_SIZE((n), (blksize)))

/* Initialize a memory pool */
os_error_t os_mempool_init(struct os_mempool *mp, int blocks, int block_size, 
                           void *membuf, char *name);

/* Get a memory block from the pool */
void *os_memblock_get(struct os_mempool *mp);

/* Put the memory block back into the pool */
os_error_t os_memblock_put(struct os_mempool *mp, void *block_addr);

#endif  /* _OS_MEMPOOL_H_ */
