#include <stdbool.h>
#include <stdio.h>

#include <led.h>
#include <timer.h>
#include <spi.h>

#include <stdint.h>
#include "xmodem.h"

#define NUM_PIXELS 150
#define PIXEL_BUFFER_SIZE ((NUM_PIXELS*4) + 8)

static char pixels[PIXEL_BUFFER_SIZE];
static char rbuf[PIXEL_BUFFER_SIZE];
static uint32_t RED_OFFSET = 3;
static uint32_t GREEN_OFFSET = 2;
static uint32_t BLUE_OFFSET = 1;

void xmodem_callback(char* buf, int len, int error);

static void initialize_strip(void) {
    spi_set_chip_select(0);
    spi_set_rate(12e6);
    spi_set_polarity(false);
    spi_set_phase(false);

    int i;
    for (i = 0; i < 4; i++) {
        pixels[i] = 0x0;
    }

    for (i = 4; i < PIXEL_BUFFER_SIZE; i++) {
        pixels[i] = 0xFF;
    }
}

static uint8_t red(uint32_t color) {
    return (uint8_t)((color >> (8*(3 - RED_OFFSET))) & 0xFF);
}

static uint8_t green(uint32_t color) {
    return (uint8_t)((color >> (8*(3 - GREEN_OFFSET))) & 0xFF);
}

static uint8_t blue(uint32_t color) {
    return (uint8_t)((color >> (8*(3 - BLUE_OFFSET))) & 0xFF);
}

static uint32_t color(uint8_t r, uint8_t g, uint8_t b) {
    return 0 | (r << (8*(3 - RED_OFFSET))) | (g << (8*(3 - GREEN_OFFSET))) | (b << (8*(3 - BLUE_OFFSET)));
}

static void set_pixel(uint32_t pixel, uint32_t color) {
    pixels[pixel*4 + 4 + RED_OFFSET] = red(color);
    pixels[pixel*4 + 4 + BLUE_OFFSET] = blue(color);
    pixels[pixel*4 + 4 + GREEN_OFFSET] = green(color);
}


static uint32_t __attribute__((unused)) get_pixel(uint32_t pixel) {
    return color(pixels[pixel*4 + 4 + RED_OFFSET],
                 pixels[pixel*4 + 4 + GREEN_OFFSET],
                 pixels[pixel*4 + 4 + BLUE_OFFSET]);
}

static void write_cb(__attribute__ ((unused)) int arg0,
                     __attribute__ ((unused)) int arg2,
                     __attribute__ ((unused)) int arg3,
                     __attribute__ ((unused)) void* userdata) {
}


static void update_strip(void) {
    spi_read_write(pixels, rbuf, PIXEL_BUFFER_SIZE, write_cb, NULL);
}

bool update = false;
void xmodem_callback(__attribute__ ((unused)) char* buf, 
		     __attribute__ ((unused)) int len, 
		     __attribute__ ((unused)) int error) {
    update = true;
}

int main(void) {
    initialize_strip();
    xmodem_init();
    xmodem_set_buffer(pixels, PIXEL_BUFFER_SIZE);
    xmodem_set_callback(xmodem_callback);
    int i;
    for (i = 0; i < NUM_PIXELS; i++) {
        set_pixel(i, 0);
        update_strip();
    }
    set_pixel(8, color(32, 32, 32));
    update_strip();
    set_pixel(9, color(32, 32, 32));
    set_pixel(11, color(32, 32, 32));
    while (1) {
        yield();
	if (update) {
            update = false;
            update_strip();
	}
    }
}
