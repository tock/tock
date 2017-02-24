#pragma once

#include <tock.h>

#ifdef __cplusplus
extern "C" {
#endif

#define DRIVER_NUM_CRC 12

enum crc_polynomial {
  CRC_CCIT8023,   // Polynomial 0x04C11DB7
  CRC_CASTAGNOLI, // Polynomial 0x1EDC6F41
  CRC_CCIT16      // Polynomial 0x1021
};

// Does the driver exist?
int crc_exists(void);

// Get the version of the CRC firmware
uint32_t crc_version(void);

// Register a callback to receive CRC results
//
// The callback will receive these parameters, in order:
//    status: SUCCESS if all inputs are valid, else EINVAL
//    result: When status == SUCCESS, the CRC result
int crc_subscribe(subscribe_cb, void *);

// Provide the buffer over which to compute a CRC
int crc_set_buffer(const void*, size_t);

// Request a CRC computation.
//
// The callback and buffer must be provided first.
//
// If SUCCESS is returned, the result will be provided to
// the registered callback.
//
// Returns EBUSY if a computation is already in progress.
// Returns ESIZE if the buffer is too big for the unit.
int crc_compute(enum crc_polynomial);

#ifdef __cplusplus
}
#endif
