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

#ifndef _OS_SCHED_H
#define _OS_SCHED_H

#include "os/os_task.h"

void os_sched_ctx_sw_hook(struct os_task *);
struct os_task *os_sched_get_current_task(void);
void os_sched_set_current_task(struct os_task *);
struct os_task *os_sched_next_task(void);
void os_sched(struct os_task *);
void os_sched_os_timer_exp(void);
os_error_t os_sched_insert(struct os_task *);
int os_sched_sleep(struct os_task *, os_time_t nticks);
int os_sched_wakeup(struct os_task *);
void os_sched_resort(struct os_task *);
os_time_t os_sched_wakeup_ticks(os_time_t now);

#endif /* _OS_SCHED_H */
