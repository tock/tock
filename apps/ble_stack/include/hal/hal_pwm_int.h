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

#ifndef H_HAL_PWM_INT_
#define H_HAL_PWM_INT_

#ifdef __cplusplus
extern "C" {
#endif

#include <hal/hal_pwm.h>
#include <inttypes.h>

/* when you are implementing a driver for the hal_pwm. This is the interface
 * you must provide.
 */

struct hal_pwm;

struct hal_pwm_funcs {
    /* the low level hal API */
    int     (*hpwm_get_bits)        (struct hal_pwm *ppwm);
    int     (*hpwm_get_clk)         (struct hal_pwm *ppwm);
    int     (*hpwm_disable)         (struct hal_pwm *ppwm);
    int     (*hpwm_ena_duty)  (struct hal_pwm *ppwm, uint16_t frac_duty);
    int     (*hpwm_set_freq)  (struct hal_pwm *ppwm, uint32_t freq_hz);

};

struct hal_pwm {
    const struct hal_pwm_funcs *driver_api;
};

struct hal_pwm *
bsp_get_hal_pwm_driver(enum system_device_id sysid);


#ifdef __cplusplus
}
#endif

#endif /* H_HAL_PWM_INT_ */

