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

#ifndef H_HAL_FLASH_INT_
#define H_HAL_FLASH_INT_

#ifdef __cplusplus
extern "C" {
#endif

#include <inttypes.h>

/*
 * API that flash driver has to implement.
 */
struct hal_flash_funcs {
    int (*hff_read)(uint32_t address, void *dst, uint32_t num_bytes);
    int (*hff_write)(uint32_t address, const void *src, uint32_t num_bytes);
    int (*hff_erase_sector)(uint32_t sector_address);
    int (*hff_sector_info)(int idx, uint32_t *address, uint32_t *size);
    int (*hff_init)(void);
};

struct hal_flash {
    const struct hal_flash_funcs *hf_itf;
    uint32_t hf_base_addr;
    uint32_t hf_size;
    int hf_sector_cnt;
    int hf_align;		/* Alignment requirement. 1 if unrestricted. */
};

/*
 * Return size of the flash sector. sec_idx is index to hf_sectors array.
 */
uint32_t hal_flash_sector_size(const struct hal_flash *hf, int sec_idx);

/* External function prototype supplied by BSP */
const struct hal_flash *bsp_flash_dev(uint8_t flash_id);


#ifdef __cplusplus
}
#endif

#endif /* H_HAL_FLASH_INT_ */
