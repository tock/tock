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

#ifndef H_HAL_SYSTEM_
#define H_HAL_SYSTEM_

#ifdef __cplusplus
extern "C" {
#endif

/*
 * System reset.
 */
void system_reset(void) __attribute((noreturn));

/*
 * Called by bootloader to start loaded program.
 */
void system_start(void *img_start) __attribute((noreturn));


#ifdef __cplusplus
}
#endif

#endif /* H_HAL_SYSTEM_ */
