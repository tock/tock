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

#ifndef _OS_TASK_H
#define _OS_TASK_H

#include "os/os.h"
#include "os/os_sanity.h" 
#include "os/queue.h"

/* The highest and lowest task priorities */
#define OS_TASK_PRI_HIGHEST (0)
#define OS_TASK_PRI_LOWEST  (0xff)

/* 
 * Generic "object" structure. All objects that a task can wait on must
 * have a SLIST_HEAD(, os_task) head_name as the first element in the object 
 * structure. The element 'head_name' can be any name. See os_mutex.h or
 * os_sem.h for an example.
 */
struct os_task_obj
{
    SLIST_HEAD(, os_task) obj_head;     /* chain of waiting tasks */
};

/* Task states */
typedef enum os_task_state {
    OS_TASK_READY = 1, 
    OS_TASK_SLEEP = 2
} os_task_state_t;

/* Task flags */
#define OS_TASK_FLAG_NO_TIMEOUT     (0x01U)
#define OS_TASK_FLAG_SEM_WAIT       (0x02U)
#define OS_TASK_FLAG_MUTEX_WAIT     (0x04U)

typedef void (*os_task_func_t)(void *);

#define OS_TASK_MAX_NAME_LEN (32)

struct os_task {
    os_stack_t *t_stackptr;
    os_stack_t *t_stacktop;
    
    uint16_t t_stacksize;
    uint16_t t_pad;

    uint8_t t_taskid;
    uint8_t t_prio;
    uint8_t t_state;
    uint8_t t_flags;

    char *t_name;
    os_task_func_t t_func;
    void *t_arg;

    void *t_obj;

    struct os_sanity_check t_sanity_check; 

    os_time_t t_next_wakeup;
    os_time_t t_run_time;
    uint32_t t_ctx_sw_cnt;
   
    /* Global list of all tasks, irrespective of run or sleep lists */
    STAILQ_ENTRY(os_task) t_os_task_list;

    /* Used to chain task to either the run or sleep list */ 
    TAILQ_ENTRY(os_task) t_os_list;

    /* Used to chain task to an object such as a semaphore or mutex */
    SLIST_ENTRY(os_task) t_obj_list;
};

int os_task_init(struct os_task *, char *, os_task_func_t, void *, uint8_t,
        os_time_t, os_stack_t *, uint16_t);

uint8_t os_task_count(void);

struct os_task_info {
    uint8_t oti_prio;
    uint8_t oti_taskid;
    uint8_t oti_state;
    uint8_t oti_flags;
    uint16_t oti_stkusage;
    uint16_t oti_stksize;
    uint32_t oti_cswcnt;
    uint32_t oti_runtime;
    os_time_t oti_last_checkin;
    os_time_t oti_next_checkin;

    char oti_name[OS_TASK_MAX_NAME_LEN];
};
struct os_task *os_task_info_get_next(const struct os_task *, 
        struct os_task_info *);


#endif /* _OS_TASK_H */
