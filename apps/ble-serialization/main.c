/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include <stdlib.h>
#include <unistd.h>

#include <firestorm.h>

#include "ser_phy.h"
#include "ser_config.h"
#include "nrf_error.h"

#include "simple_ble.h"
#include "simple_adv.h"

#include "nrf.h"


// char txdone[] = "TX DONE!\r\n";
char txdone[] = "T\n";
char hello[] = "Done!\r\n";
char am[] = "am!\r\n";

CB_TYPE nop(int x, int y, int z, void *ud) { return ASYNC; }

// char tx[] = {0x08, 0x00, 0x00, 0x60, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00};
uint8_t* rx_buf = NULL;
char rx[256] = {0x0};
int  _rx_len = 0;

int tx_num = 0;

// int stop = 0;





static uint8_t mp_tx_stream[512];                         /**< Pointer to Tx data */
static uint16_t  m_tx_stream_length;                   /**< Length of Tx data including SER_PHY
                                                        *   header */
// static uint16_t  m_tx_stream_index;                    /**< Byte index in Tx data */
static uint8_t   m_tx_length_buf[SER_PHY_HEADER_SIZE]; /**< Buffer for header of Tx packet */

// static uint8_t * mp_rx_stream;                         /**< Pointer to Rx buffer */
// static uint16_t  m_rx_stream_length;                   /**< Length of Rx data including SER_PHY
                                                        // *   header*/
// static uint16_t  m_rx_stream_index;                    /**< Byte index in Rx data */
// static uint8_t   m_rx_length_buf[SER_PHY_HEADER_SIZE]; /**< Buffer for header of Rx packet */
// static uint8_t   m_rx_drop_buf[1];                     /**< 1-byte buffer used to trash incoming
                                                        // *   data */
// static uint8_t   m_rx_byte;                            /**< Rx byte passed from low-level driver */

static ser_phy_events_handler_t m_ser_phy_event_handler; /**< Event handler for upper layer */
static ser_phy_evt_t            m_ser_phy_rx_event;      /**< Rx event for upper layer
                                                          *   notification */
static ser_phy_evt_t            m_ser_phy_tx_event;      /**< Tx event for upper layer
                                                          *   notification */



// Callback from the UART layer
CB_TYPE ble_serialization_callback (int callback_type, int rx_len, int c, void* other) {
    // gpio_set(LED_0);
	if (callback_type == 1) {
        // TX DONE

        // gpio_set(LED_0);

        // mp_tx_stream       = NULL;
        m_tx_stream_length = 0;
        // m_tx_stream_index  = 0;

        m_ser_phy_tx_event.evt_type = SER_PHY_EVT_TX_PKT_SENT;

        if (m_ser_phy_event_handler) {
            m_ser_phy_event_handler(m_ser_phy_tx_event);
        }



		// putnstr_async(txdone, sizeof(txdone), nop, NULL);
		// putnstr_async(am, sizeof(am), nop, NULL);




	} else if (callback_type == 2) {

        // if (rx[3] == 0x77) gpio_set(LED_0);


// gpio_set(LED_0);


        // need a dummy request for a buffer to keep the state machines
        // in the serialization library happy
        m_ser_phy_rx_event.evt_type = SER_PHY_EVT_RX_BUF_REQUEST;
        m_ser_phy_rx_event.evt_params.rx_buf_request.num_of_bytes = rx_len - SER_PHY_HEADER_SIZE;

        // callback_ser_phy_event(m_ser_phy_rx_event);
        if (m_ser_phy_event_handler) {
            m_ser_phy_event_handler(m_ser_phy_rx_event);
        }



    } else if (callback_type == 3) {

    // gpio_set(LED_0);

        if (rx_buf) {
            //gpio_set(LED_0);

        _rx_len = rx_len;

        memcpy(rx_buf, rx+2, rx_len - SER_PHY_HEADER_SIZE);

        m_ser_phy_rx_event.evt_type = SER_PHY_EVT_RX_PKT_RECEIVED;
        m_ser_phy_rx_event.evt_params.rx_pkt_received.num_of_bytes = rx_len - SER_PHY_HEADER_SIZE;
        m_ser_phy_rx_event.evt_params.rx_pkt_received.p_buffer = rx_buf;
        // m_ser_phy_rx_event.evt_params.rx_pkt_received.p_buffer = rx;



        if (m_ser_phy_event_handler) {
            m_ser_phy_event_handler(m_ser_phy_rx_event);
        }

    }


        // gpio_set(LED_0);







		// char buf[25];
		// snprintf(buf, 25, "Read %d ok\r\n", b);
		// putnstr_async(buf, strlen(buf), nop, NULL);
		// // wait();

		// if (stop == 0) {
		// 	nrf51822_serialization_write(tx, 10, rx, 128);
		// 	stop = 1;
		// }

		// putstr(buf);
		// putnstr_async(hello, sizeof(hello), nop, NULL);
	}


}



ble_address_set() {
    __asm("nop;");
}





/******************************************************************************/
/* Helper functions that should be somewhere more generic
/******************************************************************************/


// copied from spi_byte app

/* FIXME: These delay functions are Cortex-M0 specific (and calibrated for a
 * 16MHz CPU clock), therefore should be moved to platform specific location.
 * */

/* Delay for for the given microseconds (approximately).
 *
 * For a 16 MHz CPU, 1us == 16 instructions (assuming each instruction takes
 * one cycle). */
static void delay_us(int duration)
{
    // The inner loop instructions are: 14 NOPs + 1 SUBS/ADDS + 1 CMP
    while (duration-- != 0) {
        __asm volatile (
            "nop\n"
            "nop\n"
            "nop\n"
            "nop\n"
            "nop\n"
            "nop\n"
            "nop\n"
            "nop\n"
            "nop\n"
            "nop\n"
            "nop\n"
            "nop\n"
            "nop\n"
            "nop\n"
        );
    }
}

/* Delay for for the given milliseconds (approximately).
 *
 * Note that this is not precise as there are 2 extra instructions on the inner
 * loop. Therefore, there is 1us added every 8 iterations. */
static void delay_ms(int duration)
{
    while (duration-- != 0) {
        delay_us(1000);
    }
}






/******************************************************************************/
/* Main API for upper layers of BLE serialization
/******************************************************************************/

//
// ser_app_hal_nrf51.c
//

uint32_t ser_app_hal_hw_init() {
    // gpio_set(LED_0);

    // nrf_gpio_cfg_output(CONN_CHIP_RESET_PIN_NO);

    // NRF_CLOCK->LFCLKSRC            = (CLOCK_LFCLKSRC_SRC_Xtal << CLOCK_LFCLKSRC_SRC_Pos);
    // NRF_CLOCK->EVENTS_LFCLKSTARTED = 0;
    // NRF_CLOCK->TASKS_LFCLKSTART    = 1;

    // while (NRF_CLOCK->EVENTS_LFCLKSTARTED == 0)
    // {
    //     //No implementation needed.
    // }

    // NRF_CLOCK->EVENTS_LFCLKSTARTED = 0;

    return NRF_SUCCESS;

}

void ser_app_hal_delay (uint32_t ms)  {
    delay_ms(ms);
}

void ser_app_hal_nrf_reset_pin_clear() {
    // Don't think we have this pin wired up. If needed, will add.

    // nrf_gpio_pin_clear(CONN_CHIP_RESET_PIN_NO);
}

void ser_app_hal_nrf_reset_pin_set() {
    // nrf_gpio_pin_set(CONN_CHIP_RESET_PIN_NO);
}

void ser_app_hal_nrf_evt_irq_priority_set () {
    // We aren't using an actual interrupt....

    // NVIC_SetPriority(SOFTDEVICE_EVT_IRQ, APP_IRQ_PRIORITY_LOW);
}

void ser_app_hal_nrf_evt_pending() {
    // gpio_set(LED_0);
    // NVIC_SetPendingIRQ(SOFTDEVICE_EVT_IRQ);

    // Not sure if we can do software interrupts, so try just doing a
    // function call.
    TOCK_EVT_IRQHandler();
}

// These three functions don't appear to be called.
uint32_t sd_ppi_channel_enable_get(uint32_t * p_channel_enable) {
    // *p_channel_enable = NRF_PPI->CHEN;
    return NRF_SUCCESS;
}

uint32_t sd_ppi_channel_enable_set(uint32_t channel_enable_set_msk) {
    // NRF_PPI->CHEN = channel_enable_set_msk;
    return NRF_SUCCESS;
}

uint32_t sd_ppi_channel_assign(uint8_t               channel_num,
                               const volatile void * evt_endpoint,
                               const volatile void * task_endpoint) {
    // NRF_PPI->CH[channel_num].TEP = (uint32_t)task_endpoint;
    // NRF_PPI->CH[channel_num].EEP = (uint32_t)evt_endpoint;
    return NRF_SUCCESS;
}






uint32_t ser_phy_open (ser_phy_events_handler_t events_handler) {
    // uint32_t err_code = NRF_SUCCESS;

    // gpio_set(LED_0);

    if (events_handler == NULL) {
        return NRF_ERROR_NULL;
    }

    // Check that we haven't already opened the phy layer
    if (m_ser_phy_event_handler != NULL) {
        return NRF_ERROR_INVALID_STATE;
    }

    //Configure UART and register handler
    //uart_evt_handler is used to handle events produced by low-level uart driver
///    APP_UART_INIT(&comm_params, ser_phy_uart_evt_callback, UART_IRQ_PRIORITY, err_code);

//    //Pull down Rx pin until another side gets up to avoid receiving false bytes due to glitches
//    //on Rx line
//    nrf_gpio_cfg_input(comm_params.rx_pin_no, NRF_GPIO_PIN_PULLDOWN);

    // Save the callback handler
    m_ser_phy_event_handler = events_handler;

    return NRF_SUCCESS;
}

uint32_t ser_phy_tx_pkt_send (const uint8_t * p_buffer, uint16_t num_of_bytes) {

    tx_num++;

    if (tx_num >= 3) {
        // gpio_set(0);
    }


    // gpio_toggle(LED_0);
    if (p_buffer == NULL) {
        return NRF_ERROR_NULL;
    } else if (num_of_bytes == 0) {
        return NRF_ERROR_INVALID_PARAM;
    }

    //Check if there is no ongoing transmission at the moment
    if (m_tx_stream_length == 0) {
        // if (num_of_bytes == 3) gpio_set(0);
        (void) uint16_encode(num_of_bytes, m_tx_length_buf);
        mp_tx_stream[0] = num_of_bytes & 0xFF;
        mp_tx_stream[1] = (num_of_bytes >> 8) & 0xFF;

        memcpy(mp_tx_stream+2, p_buffer, num_of_bytes);

        // mp_tx_stream       = (uint8_t*) p_buffer;
        m_tx_stream_length = num_of_bytes + SER_PHY_HEADER_SIZE;

        //Call tx procedure to start transmission of a packet
        // ser_phy_uart_tx();
        nrf51822_serialization_write(mp_tx_stream, m_tx_stream_length);
    } else {
        return NRF_ERROR_BUSY;
    }

    return NRF_SUCCESS;
}


uint32_t ser_phy_rx_buf_set (uint8_t* p_buffer) {
// gpio_set(0);
    // if (m_ser_phy_rx_event.evt_type != SER_PHY_EVT_RX_BUF_REQUEST) {
    //     return NRF_ERROR_INVALID_STATE;
    // }

    // if (p_buffer != NULL) {
    //     mp_rx_stream = p_buffer;
    // } else {
    //     mp_rx_stream = m_rx_drop_buf;
    // }

    // //Unblock RXRDY interrupts as higher layer has responded (with a valid or NULL pointer)
    // NRF_UART0->INTENSET = (UART_INTENSET_RXDRDY_Set << UART_INTENSET_RXDRDY_Pos);



    // m_ser_phy_rx_event.evt_type = SER_PHY_EVT_RX_PKT_RECEIVED;
    // m_ser_phy_rx_event.evt_params.rx_pkt_received.num_of_bytes = _rx_len - SER_PHY_HEADER_SIZE;
    // m_ser_phy_rx_event.evt_params.rx_pkt_received.p_buffer = rx+2;

    // if (m_ser_phy_event_handler) {
    //     m_ser_phy_event_handler(m_ser_phy_rx_event);
    // }

    rx_buf = p_buffer;


    return NRF_SUCCESS;
}

void ser_phy_close (void) {
    m_ser_phy_event_handler = NULL;
    // (void)app_uart_close();
}

void ser_phy_interrupts_enable (void) {
    // NVIC_EnableIRQ(SER_UART_IRQ);
}

void ser_phy_interrupts_disable (void) {
    // NVIC_DisableIRQ(SER_UART_IRQ);
}




uint32_t app_timer_init(uint32_t                      prescaler,
                        uint8_t                       op_queues_size,
                        void *                        p_buffer,
                        app_timer_evt_schedule_func_t evt_schedule_func){ return NRF_SUCCESS; }

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
uint32_t app_timer_create(app_timer_id_t const *      p_timer_id,
                          app_timer_mode_t            mode,
                          app_timer_timeout_handler_t timeout_handler){ return NRF_SUCCESS; }

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
uint32_t app_timer_start(app_timer_id_t timer_id, uint32_t timeout_ticks, void * p_context){ return NRF_SUCCESS; }

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
uint32_t app_timer_stop(app_timer_id_t timer_id){ return NRF_SUCCESS; }

/**@brief Function for stopping all running timers.
 *
 * @retval     NRF_SUCCESS               If all timers were successfully stopped.
 * @retval     NRF_ERROR_INVALID_STATE   If the application timer module has not been initialized.
 * @retval     NRF_ERROR_NO_MEM          If the timer operations queue was full.
 */
uint32_t app_timer_stop_all(void){ return NRF_SUCCESS; }

/**@brief Function for returning the current value of the RTC1 counter.
 *
 * @param[out] p_ticks   Current value of the RTC1 counter.
 *
 * @retval     NRF_SUCCESS   If the counter was successfully read.
 */
uint32_t app_timer_cnt_get(uint32_t * p_ticks){ return NRF_SUCCESS; }

/**@brief Function for computing the difference between two RTC1 counter values.
 *
 * @param[in]  ticks_to       Value returned by app_timer_cnt_get().
 * @param[in]  ticks_from     Value returned by app_timer_cnt_get().
 * @param[out] p_ticks_diff   Number of ticks from ticks_from to ticks_to.
 *
 * @retval     NRF_SUCCESS   If the counter difference was successfully computed.
 */
uint32_t app_timer_cnt_diff_compute(uint32_t   ticks_to,
                                    uint32_t   ticks_from,
                                    uint32_t * p_ticks_diff){ return NRF_SUCCESS; }





void ser_app_power_system_off_set(void)
{
    // m_power_system_off = true;
}

bool ser_app_power_system_off_get(void)
{
    return false;
}

void ser_app_power_system_off_enter(void)
{
    // NRF_POWER->SYSTEMOFF = POWER_SYSTEMOFF_SYSTEMOFF_Enter;

    // Only for debugging purpose, will not be reached without connected debugger
    // while(1);
}


uint32_t sd_app_evt_wait(void)
{
    // __WFE();
    wait();

    // gpio_set(0);

    return NRF_SUCCESS;
}


void critical_region_enter () {

}

void critical_region_exit () {

}

uint32_t sd_nvic_EnableIRQ(IRQn_Type IRQn) {
    return NRF_SUCCESS;
}



// char* name = "FSTORM";


// Intervals for advertising and connections
simple_ble_config_t ble_config = {
    .platform_id       = 0x00,              // used as 4th octect in device BLE address
    .device_id         = DEVICE_ID_DEFAULT,
    .adv_name          = "FSTORM",
    .adv_interval      = MSEC_TO_UNITS(500, UNIT_0_625_MS),
    .min_conn_interval = MSEC_TO_UNITS(500, UNIT_1_25_MS),
    .max_conn_interval = MSEC_TO_UNITS(1000, UNIT_1_25_MS)
};


void main() {

    gpio_enable(LED_0);
    gpio_clear(LED_0);

    // gpio_toggle(LED_0);
    // delay_ms(200);
    // gpio_toggle(LED_0);
    // delay_ms(200);

    // gpio_toggle(LED_0);
    // delay_ms(200);
    // gpio_toggle(LED_0);
    // delay_ms(200);

    // gpio_toggle(LED_0);
    // delay_ms(200);
    // gpio_toggle(LED_0);
    // delay_ms(200);


  // putnstr_async(hello, sizeof(hello), nop, NULL);

    nrf51822_serialization_subscribe(ble_serialization_callback);
    nrf51822_serialization_setup_rx_buffer(rx, 256);
  // nrf51822_serialization_write(tx, 10);



  // Setup BLE
    simple_ble_init(&ble_config);


    // gpio_set(LED_0);

    // Advertise because why not
    simple_adv_only_name();





}

