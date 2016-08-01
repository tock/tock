#include <stdint.h>
#include "controller/ble_phy.h"

// Capsule functions
//   - Enable radio interrupts: priority 0
//   - Enable/disable PPI channels 20, 21, 26
//   - Run Timer0
//   - Configure timer0 compare[0]
//   - Read timer0 capture[0]

struct ble_mbuf_hdr {
   uint32_t len;
};

/* Enable the wait for response timer */
#pragma GCC diagnostic ignored "-Wunused-parameter"
void ble_ll_wfr_enable(uint32_t cputime) {}

/**
 * Called by the PHY when a receive packet has ended.
 *
 * NOTE: Called from interrupt context!
 *
 * @param rxpdu Pointer to received PDU
 *        ble_hdr Pointer to BLE header of received mbuf
 *
 * @return int
 *       < 0: Disable the phy after reception.
 *      == 0: Success. Do not disable the PHY.
 *       > 0: Do not disable PHY as that has already been done.
 */

#pragma GCC diagnostic ignored "-Wunused-parameter"
int ble_ll_rx_end(struct os_mbuf *rxpdu, struct ble_mbuf_hdr *ble_hdr) {
    return 0;
}

/**
 * ble ll state get
 *
 * Called to get the current link layer state.
 *
 * Context: Link Layer task (can be called from interrupt context though).
 *
 * @return ll_state
 */
uint8_t ble_ll_state_get(void) {
    return 0;
}

/**
 * Called upon start of received PDU
 *
 * Context: Interrupt
 *
 * @param rxpdu
 *        chan
 *
 * @return int
 *   < 0: A frame we dont want to receive.
 *   = 0: Continue to receive frame. Dont go from rx to tx
 *   > 0: Continue to receive frame and go from rx to tx when done
 */
#pragma GCC diagnostic ignored "-Wunused-parameter"
int ble_ll_rx_start(struct os_mbuf *rxpdu, uint8_t chan) {
  return 0;
}

#pragma GCC diagnostic ignored "-Wunused-parameter"
void NVIC_SetVector(uint32_t vec, uint32_t isr) {
   
}

int main() {
    ble_phy_init();
	return 0;
}

