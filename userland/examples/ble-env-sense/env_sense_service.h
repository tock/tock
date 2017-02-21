#include <stdint.h>

#define ENVIRONMENTAL_SENSING_SERVICE_UUID      0x181A
#define TEMPERATURE_MEASUREMENT_CHAR_UUID       0x2A6E
#define IRRADIANCE_CHAR_UUID                    0x2A77

void env_sense_service_init(void);

uint32_t env_sense_update_irradiance(uint16_t conn, uint16_t irradiance_n);

uint32_t env_sense_update_temperature(uint16_t conn, int16_t temperature_n);
