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


#ifndef _OS_CALLOUT_H
#define _OS_CALLOUT_H

#define OS_CALLOUT_F_QUEUED (0x01)

struct os_callout {
    struct os_event c_ev;
    struct os_eventq *c_evq;
    uint32_t c_ticks;
    TAILQ_ENTRY(os_callout) c_next;
};

typedef void (*os_callout_func_t)(void *);

struct os_callout_func {
    /* Must be the first element in the structure for casting
     * purposes.
     */
    struct os_callout cf_c;
    os_callout_func_t cf_func;
};
#define CF_ARG(__cf) ((__cf)->cf_c.c_ev.ev_arg)

void os_callout_func_init(struct os_callout_func *cf, struct os_eventq *evq,
  os_callout_func_t timo_func, void *ev_arg);
void os_callout_stop(struct os_callout *);
int os_callout_reset(struct os_callout *, int32_t);
void os_callout_tick(void);
os_time_t os_callout_wakeup_ticks(os_time_t now);

static inline int
os_callout_queued(struct os_callout *c)
{
    return c->c_next.tqe_prev != NULL;
}

#endif /* _OS_CALLOUT_H */



