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
#ifndef __UTIL_STATS_H__ 
#define __UTIL_STATS_H__ 

#include <os/queue.h>
#include <stdint.h>

struct stats_name_map {
    uint16_t snm_off;
    char *snm_name;
};

struct stats_hdr {
    char *s_name;
    uint8_t s_size;
    uint8_t s_cnt;
    uint16_t s_pad1;
#ifdef STATS_NAME_ENABLE
    struct stats_name_map *s_map;
    int s_map_cnt;
#endif
    STAILQ_ENTRY(stats_hdr) s_next;
};

#define STATS_SECT_DECL(__name)             \
    struct stats_ ## __name

#define STATS_SECT_START(__name)            \
STATS_SECT_DECL(__name) {                   \
    struct stats_hdr s_hdr;

#define STATS_SECT_END };

#define STATS_SECT_VAR(__var) \
    s##__var

#define STATS_HDR(__sectname) &(__sectname).s_hdr

#define STATS_SIZE_16 (sizeof(uint16_t))
#define STATS_SIZE_32 (sizeof(uint32_t))
#define STATS_SIZE_64 (sizeof(uint64_t))

#define STATS_SECT_ENTRY(__var) uint32_t STATS_SECT_VAR(__var);
#define STATS_SECT_ENTRY16(__var) uint16_t STATS_SECT_VAR(__var);
#define STATS_SECT_ENTRY32(__var) uint32_t STATS_SECT_VAR(__var);
#define STATS_SECT_ENTRY64(__var) uint64_t STATS_SECT_VAR(__var);

#define STATS_SIZE_INIT_PARMS(__sectvarname, __size)                        \
    (__size),                                                               \
    ((sizeof (__sectvarname)) - sizeof (struct stats_hdr)) / (__size)

#define STATS_INC(__sectvarname, __var)        \
    ((__sectvarname).STATS_SECT_VAR(__var)++)

#define STATS_INCN(__sectvarname, __var, __n)  \
    ((__sectvarname).STATS_SECT_VAR(__var) += (__n))

#ifdef STATS_NAME_ENABLE

#define STATS_NAME_MAP_NAME(__sectname) g_stats_map_ ## __sectname

#define STATS_NAME_START(__sectname)                                        \
struct stats_name_map STATS_NAME_MAP_NAME(__sectname)[] = {

#define STATS_NAME(__sectname, __entry)                                     \
    { offsetof(STATS_SECT_DECL(__sectname), STATS_SECT_VAR(__entry)),       \
      #__entry },

#define STATS_NAME_END(__sectname)                                          \
};

#define STATS_NAME_INIT_PARMS(__name)                                       \
    &(STATS_NAME_MAP_NAME(__name)[0]),                                      \
    (sizeof(STATS_NAME_MAP_NAME(__name)) / sizeof(struct stats_name_map))

#else /* STATS_NAME_ENABLE */

#define STATS_NAME_START(__name)
#define STATS_NAME(__name, __entry)
#define STATS_NAME_END(__name)
#define STATS_NAME_INIT_PARMS(__name) NULL, 0

#endif /* STATS_NAME_ENABLE */

int stats_module_init(void);
void stats_module_reset(void);
int stats_init(struct stats_hdr *shdr, uint8_t size, uint8_t cnt, 
    struct stats_name_map *map, uint8_t map_cnt);
int stats_register(char *name, struct stats_hdr *shdr);
int stats_init_and_reg(struct stats_hdr *shdr, uint8_t size, uint8_t cnt,
                       struct stats_name_map *map, uint8_t map_cnt,
                       char *name);

typedef int (*stats_walk_func_t)(struct stats_hdr *, void *, char *, 
        uint16_t);
int stats_walk(struct stats_hdr *, stats_walk_func_t, void *);

typedef int (*stats_group_walk_func_t)(struct stats_hdr *, void *);
int stats_group_walk(stats_group_walk_func_t, void *);

struct stats_hdr *stats_group_find(char *name);

/* Private */
#ifdef NEWTMGR_PRESENT 
int stats_nmgr_register_group(void);
#endif 
#ifdef SHELL_PRESENT
int stats_shell_register(void);
#endif

#endif /* __UTIL_STATS_H__ */
