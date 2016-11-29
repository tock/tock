#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <stdint.h>
#include <stdbool.h>
#include <math.h>
#include <timer.h>

#include <tock.h>
#include <console.h>

#include "FXOS8700CQ.h"

const double g = -9.8; 

// Step counter/fall detector
int main() {
	printf("Step counter init\n"); 
	unsigned num_measurements = 100; 
	double accel_mags[num_measurements]; 

	for (unsigned ii = 0; ii < num_measurements; ii ++) {
		// read and calculate acceleration 
		// uint16_t accel_x = 1; 
		// uint16_t accel_y = 1; 
		// uint16_t accel_z = 1; 

		double accel_mag = FXOS8700CQ_read_accel_mag(); //sqrt(accel_x * accel_x + accel_y * accel_y + accel_z * accel_z); 
		printf("accel mag = %f\n", accel_mag); 
		accel_mags[ii] = accel_mag + g;
		delay_ms(500); 
	}

	unsigned steps = 0; 
	for (unsigned ii = 0; ii < num_measurements - 1; ii ++) {
		if (accel_mags[ii] < 0 && accel_mags[ii + 1] > 0) {
			// step occurred 
			steps ++; 
		}
	}

	printf("%u steps occurred.\n", steps); 
}