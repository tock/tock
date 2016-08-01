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

#ifndef H_HAL_FLASH_
#define H_HAL_FLASH_

#ifdef __cplusplus
extern "C" {
#endif

#include <inttypes.h>

int hal_flash_read(uint8_t flash_id, uint32_t address, void *dst,
  uint32_t num_bytes);
int hal_flash_write(uint8_t flash_id, uint32_t address, const void *src,
  uint32_t num_bytes);
int hal_flash_erase_sector(uint8_t flash_id, uint32_t sector_address);
int hal_flash_erase(uint8_t flash_id, uint32_t address, uint32_t num_bytes);
uint8_t hal_flash_align(uint8_t flash_id);
int hal_flash_init(void);


#ifdef __cplusplus
}
#endif

#endif /* H_HAL_FLASH_ */
