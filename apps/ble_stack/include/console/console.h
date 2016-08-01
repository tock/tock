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
#ifndef __CONSOLE_H__
#define __CONSOLE_H__

#include <stdarg.h>

typedef void (*console_rx_cb)(int full_line);

// Because these are dummy functions, gcc issues a lot of warnings
// These pragma disable the warnings. -pal
#pragma GCC diagnostic ignored "-Wunused"
#pragma GCC diagnostic ignored "-Wunused-function"
#pragma GCC diagnostic ignored "-Wunused-parameter"
static int 
console_is_init(void)
{
    return 0;
}

static int 
console_init(console_rx_cb rxcb)
{
    return 0;
}

static int 
console_read(char *str, int cnt)
{
    return 0;
}

static void 
console_blocking_mode(void)
{
}

static void 
console_write(const char *str, int cnt)
{
}

static void  console_printf(const char *fmt, ...)
    __attribute__ ((format (printf, 1, 2)));

static void 
console_printf(const char *fmt, ...)
{
}

static void 
console_echo(int on)
{
}

#define console_is_midline  (0)

#endif /* __CONSOLE__ */

