#include <stdbool.h>
#include <stdio.h>

#include <led.h>
#include <timer.h>
#include <spi.h>

#include <stdint.h>

#define NUM_PIXELS 150
#define PIXEL_BUFFER_SIZE ((NUM_PIXELS*4) + 8)

static char pixels[PIXEL_BUFFER_SIZE];
static char rbuf[PIXEL_BUFFER_SIZE];
static uint32_t RED_OFFSET = 3;
static uint32_t GREEN_OFFSET = 2;
static uint32_t BLUE_OFFSET = 1;

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

static uint32_t wheel(uint8_t wheelpos) {
    wheelpos = 255 - wheelpos;
    if (wheelpos < 85) return color(255 - wheelpos * 3, 0, wheelpos * 3);

    if (wheelpos < 170) {
        wheelpos -= 85;
        return color(0, wheelpos*3, 255 - wheelpos * 3);
    }

    wheelpos -= 170;
    return color(wheelpos*3, 255 - wheelpos * 3, 0);
}

static void write_cb(__attribute__ ((unused)) int arg0,
                     __attribute__ ((unused)) int arg2,
                     __attribute__ ((unused)) int arg3,
                     __attribute__ ((unused)) void* userdata) {
}


static void update_strip(void) {
    spi_read_write(pixels, rbuf, PIXEL_BUFFER_SIZE, write_cb, NULL);
}

int main(void) {
    initialize_strip();

    int i;
    for (i = 0; i < NUM_PIXELS; i++) {
        set_pixel(i, 0);
        update_strip();
    }

    uint8_t w = 0;
    uint8_t wb = 255;
    int which = 0;
    int dir = 1;
    while (1) {
        delay_ms(20);
        set_pixel(which, color(0, 0, 0));
        which = which + dir;
        set_pixel(which, color(32, 32, 32));
        if (which == NUM_PIXELS) {
           dir = -1;
        } else if (which == 0) {
           dir = 1;
        }
        update_strip();
    }
}
