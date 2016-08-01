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

#ifndef _OS_SANITY_H
#define _OS_SANITY_H

#include <stdint.h> 

#include "os/os_time.h"
#include "os/queue.h" 

struct os_sanity_check;
typedef int (*os_sanity_check_func_t)(struct os_sanity_check *, void *);

struct os_sanity_check {
    os_time_t sc_checkin_last;
    os_time_t sc_checkin_itvl;
    os_sanity_check_func_t sc_func;
    void *sc_arg; 

    SLIST_ENTRY(os_sanity_check) sc_next;

};

#define OS_SANITY_CHECK_SETFUNC(__sc, __f, __arg, __itvl)  \
    (__sc)->sc_func = (__f);                               \
    (__sc)->sc_arg = (__arg);                              \
    (__sc)->sc_checkin_itvl = (__itvl) * OS_TICKS_PER_SEC;

int os_sanity_task_init(int);
struct os_task;
int os_sanity_task_checkin(struct os_task *);

int os_sanity_check_init(struct os_sanity_check *);
int os_sanity_check_register(struct os_sanity_check *);
int os_sanity_check_reset(struct os_sanity_check *);

#endif /* _OS_SANITY_H */
