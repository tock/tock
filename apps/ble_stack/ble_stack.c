#include <stdint.h>
#include "controller/ble_phy.h"
int main() {
    ble_phy_init();
	return 0;
}

// Capsule functions
//   - Enable radio interrupts: priority 0
//   - Enable/disable PPI channels 20, 21, 26
//   - Configure timer0 compare[0]
//   - Read timer0 capture[0]
