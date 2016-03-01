
#include <stdint.h>
#include <stdbool.h>
#include <string.h>

#include "nrf.h"
#include "ser_phy.h"
#include "ser_config.h"
#include "nrf_error.h"
#include "nordic_common.h"
#include "app_timer.h"

#include "firestorm.h"
#include "delay.h"

// Buffer to receive packets from the nRF51822 in.
// The upper layer also has a buffer, which we could use, but to make
// the timing work out better we just keep a buffer around that the kernel
// can keep a pointer to.
static uint8_t rx[SER_HAL_TRANSPORT_RX_MAX_PKT_SIZE];
// This is a pointer to the RX buffer passed in by the upper seralization
// layer.
static uint8_t* hal_rx_buf = NULL;

// Buffer to create an outgoing packet into.
static uint8_t tx[SER_HAL_TRANSPORT_TX_MAX_PKT_SIZE];
// Length of the outgoing packet.
static uint16_t tx_len;

// Callback that we pass TX done and RX events to
static ser_phy_events_handler_t _ser_phy_event_handler;
// Data structure that we pass for receive events
static ser_phy_evt_t _ser_phy_rx_event;
// Data structure for TX events.
static ser_phy_evt_t _ser_phy_tx_event;


/*******************************************************************************
 * Callback from the UART layer in the kernel
 ******************************************************************************/

CB_TYPE ble_serialization_callback (int callback_type, int rx_len, int c, void* other) {
    UNUSED_PARAMETER(c);
    UNUSED_PARAMETER(other);

	if (callback_type == 1) {
        // TX DONE

        // Reset that we are no longer sending a packet.
        tx_len = 0;

        // Notify the upper layer
        _ser_phy_tx_event.evt_type = SER_PHY_EVT_TX_PKT_SENT;

        if (_ser_phy_event_handler) {
            _ser_phy_event_handler(_ser_phy_tx_event);
        }

	} else if (callback_type == 2) {
        // RX STARTED

        // Need a dummy request for a buffer to keep the state machines
        // in the serialization library happy. We do use this buffer, but
        // we don't block packet receive until we get it.
        _ser_phy_rx_event.evt_type = SER_PHY_EVT_RX_BUF_REQUEST;
        _ser_phy_rx_event.evt_params.rx_buf_request.num_of_bytes = rx_len - SER_PHY_HEADER_SIZE;

        if (_ser_phy_event_handler) {
            _ser_phy_event_handler(_ser_phy_rx_event);
        }

    } else if (callback_type == 3) {
        // RX DONE

        // Check that we actually have a buffer to pass to the upper layers.
        // This buffer MUST be the same buffer that it passed us.
        if (hal_rx_buf) {
            // Copy our buffer into the upper layer's buffer.
            memcpy(hal_rx_buf, rx+2, rx_len - SER_PHY_HEADER_SIZE);

            _ser_phy_rx_event.evt_type = SER_PHY_EVT_RX_PKT_RECEIVED;
            _ser_phy_rx_event.evt_params.rx_pkt_received.num_of_bytes = rx_len - SER_PHY_HEADER_SIZE;
            _ser_phy_rx_event.evt_params.rx_pkt_received.p_buffer = hal_rx_buf;

            if (_ser_phy_event_handler) {
                _ser_phy_event_handler(_ser_phy_rx_event);
            }
        }

	}

    return 0;
}



/*******************************************************************************
 * Main API for upper layers of BLE serialization
 ******************************************************************************/

//
// ser_app_hal_nrf51.c
//

uint32_t ser_app_hal_hw_init() {
    // TODO: configure the pin for the reset pin
    // nrf_gpio_cfg_output(CONN_CHIP_RESET_PIN_NO);

    return NRF_SUCCESS;
}

void ser_app_hal_delay (uint32_t ms)  {
    delay_ms(ms);
}

void ser_app_hal_nrf_reset_pin_clear() {
    // TODO: reset nRF
    // nrf_gpio_pin_clear(CONN_CHIP_RESET_PIN_NO);
}

void ser_app_hal_nrf_reset_pin_set() {
    // TODO: stop resetting nRF
    // nrf_gpio_pin_set(CONN_CHIP_RESET_PIN_NO);
}

void ser_app_hal_nrf_evt_irq_priority_set () {
    // Since we aren't using an actual interrupt, not needed
}

void ser_app_hal_nrf_evt_pending() {
    // Not sure if we can do software interrupts, so try just doing a
    // function call.
    TOCK_EVT_IRQHandler();
}


//
// ser_phy_nrf51_uart.c
//

uint32_t ser_phy_open (ser_phy_events_handler_t events_handler) {
    if (events_handler == NULL) {
        return NRF_ERROR_NULL;
    }

    // Check that we haven't already opened the phy layer
    if (_ser_phy_event_handler != NULL) {
        return NRF_ERROR_INVALID_STATE;
    }

    // Configure the serialization layer in the kernel
    nrf51822_serialization_subscribe(ble_serialization_callback);
    nrf51822_serialization_setup_rx_buffer((char*) rx, SER_HAL_TRANSPORT_RX_MAX_PKT_SIZE);

    // Save the callback handler
    _ser_phy_event_handler = events_handler;

    return NRF_SUCCESS;
}

uint32_t ser_phy_tx_pkt_send (const uint8_t* p_buffer, uint16_t num_of_bytes) {

    // Error checks
    if (p_buffer == NULL) {
        return NRF_ERROR_NULL;
    } else if (num_of_bytes == 0) {
        return NRF_ERROR_INVALID_PARAM;
    }

    // Check if there is no ongoing transmission at the moment
    if (tx_len == 0) {
        // Encode the number of bytes as the first two bytes of the outgoing
        // packet.
        tx[0] = num_of_bytes & 0xFF;
        tx[1] = (num_of_bytes >> 8) & 0xFF;

        // Copy in the outgoing data
        memcpy(tx+2, p_buffer, num_of_bytes);

        // Add in that we added the header (2 length bytes)
        tx_len = num_of_bytes + SER_PHY_HEADER_SIZE;

        // Call tx procedure to start transmission of a packet
        nrf51822_serialization_write((char*) tx, tx_len);
    } else {
        return NRF_ERROR_BUSY;
    }

    return NRF_SUCCESS;
}


uint32_t ser_phy_rx_buf_set (uint8_t* p_buffer) {
    // Save a pointer to the buffer we can use.
    hal_rx_buf = p_buffer;

    return NRF_SUCCESS;
}

void ser_phy_close () {
    _ser_phy_event_handler = NULL;
}

void ser_phy_interrupts_enable () { }

void ser_phy_interrupts_disable () { }


// TODO: implement timers!

uint32_t app_timer_init (uint32_t                      prescaler,
                         uint8_t                       op_queues_size,
                         void *                        p_buffer,
                         app_timer_evt_schedule_func_t evt_schedule_func) {
    UNUSED_PARAMETER(prescaler);
    UNUSED_PARAMETER(op_queues_size);
    UNUSED_PARAMETER(p_buffer);
    UNUSED_PARAMETER(evt_schedule_func);
    return NRF_SUCCESS;
}

/**@brief Function for creating a timer instance.
 *
 * @param[in]  p_timer_id        Pointer to timer identifier.
 * @param[in]  mode              Timer mode.
 * @param[in]  timeout_handler   Function to be executed when the timer expires.
 *
 * @retval     NRF_SUCCESS               If the timer was successfully created.
 * @retval     NRF_ERROR_INVALID_PARAM   If a parameter was invalid.
 * @retval     NRF_ERROR_INVALID_STATE   If the application timer module has not been initialized or
 *                                       the timer is running.
 *
 * @note This function does the timer allocation in the caller's context. It is also not protected
 *       by a critical region. Therefore care must be taken not to call it from several interrupt
 *       levels simultaneously.
 * @note The function can be called again on the timer instance and will re-initialize the instance if
 *       the timer is not running.
 * @attention The FreeRTOS and RTX app_timer implementation does not allow app_timer_create to
 *       be called on the previously initialized instance.
 */
uint32_t app_timer_create (app_timer_id_t const *      p_timer_id,
                           app_timer_mode_t            mode,
                           app_timer_timeout_handler_t timeout_handler) {
    UNUSED_PARAMETER(p_timer_id);
    UNUSED_PARAMETER(mode);
    UNUSED_PARAMETER(timeout_handler);
    return NRF_SUCCESS;
}

/**@brief Function for starting a timer.
 *
 * @param[in]       timer_id      Timer identifier.
 * @param[in]       timeout_ticks Number of ticks (of RTC1, including prescaling) to time-out event
 *                                (minimum 5 ticks).
 * @param[in]       p_context     General purpose pointer. Will be passed to the time-out handler when
 *                                the timer expires.
 *
 * @retval     NRF_SUCCESS               If the timer was successfully started.
 * @retval     NRF_ERROR_INVALID_PARAM   If a parameter was invalid.
 * @retval     NRF_ERROR_INVALID_STATE   If the application timer module has not been initialized or the timer
 *                                       has not been created.
 * @retval     NRF_ERROR_NO_MEM          If the timer operations queue was full.
 *
 * @note The minimum timeout_ticks value is 5.
 * @note For multiple active timers, time-outs occurring in close proximity to each other (in the
 *       range of 1 to 3 ticks) will have a positive jitter of maximum 3 ticks.
 * @note When calling this method on a timer that is already running, the second start operation
 *       is ignored.
 */
uint32_t app_timer_start (app_timer_id_t timer_id,
                          uint32_t timeout_ticks,
                          void* p_context) {
    UNUSED_PARAMETER(timer_id);
    UNUSED_PARAMETER(timeout_ticks);
    UNUSED_PARAMETER(p_context);
    return NRF_SUCCESS;
}

/**@brief Function for stopping the specified timer.
 *
 * @param[in]  timer_id                  Timer identifier.
 *
 * @retval     NRF_SUCCESS               If the timer was successfully stopped.
 * @retval     NRF_ERROR_INVALID_PARAM   If a parameter was invalid.
 * @retval     NRF_ERROR_INVALID_STATE   If the application timer module has not been initialized or the timer
 *                                       has not been created.
 * @retval     NRF_ERROR_NO_MEM          If the timer operations queue was full.
 */
uint32_t app_timer_stop (app_timer_id_t timer_id) {
    UNUSED_PARAMETER(timer_id);
    return NRF_SUCCESS;
}

/**@brief Function for stopping all running timers.
 *
 * @retval     NRF_SUCCESS               If all timers were successfully stopped.
 * @retval     NRF_ERROR_INVALID_STATE   If the application timer module has not been initialized.
 * @retval     NRF_ERROR_NO_MEM          If the timer operations queue was full.
 */
uint32_t app_timer_stop_all () {
    return NRF_SUCCESS;
}

/**@brief Function for returning the current value of the RTC1 counter.
 *
 * @param[out] p_ticks   Current value of the RTC1 counter.
 *
 * @retval     NRF_SUCCESS   If the counter was successfully read.
 */
uint32_t app_timer_cnt_get (uint32_t* p_ticks) {
    UNUSED_PARAMETER(p_ticks);
    return NRF_SUCCESS;
}

/**@brief Function for computing the difference between two RTC1 counter values.
 *
 * @param[in]  ticks_to       Value returned by app_timer_cnt_get().
 * @param[in]  ticks_from     Value returned by app_timer_cnt_get().
 * @param[out] p_ticks_diff   Number of ticks from ticks_from to ticks_to.
 *
 * @retval     NRF_SUCCESS   If the counter difference was successfully computed.
 */
uint32_t app_timer_cnt_diff_compute (uint32_t ticks_to,
                                     uint32_t ticks_from,
                                     uint32_t* p_ticks_diff) {
    UNUSED_PARAMETER(ticks_to);
    UNUSED_PARAMETER(ticks_from);
    UNUSED_PARAMETER(p_ticks_diff);
    return NRF_SUCCESS;
}





void ser_app_power_system_off_set () {

}

bool ser_app_power_system_off_get () {
    return false;
}

void ser_app_power_system_off_enter () {

}

// Essentially sleep this process
uint32_t sd_app_evt_wait () {
    wait();
    return NRF_SUCCESS;
}

void critical_region_enter () {

}

void critical_region_exit () {

}

uint32_t sd_nvic_EnableIRQ (IRQn_Type IRQn) {
    UNUSED_PARAMETER(IRQn);
    return NRF_SUCCESS;
}
