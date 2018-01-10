#include <stdbool.h>
#include <stdio.h>
#include <stdint.h>

// Tock API
#include <led.h>
#include <timer.h>
#include <spi.h>

// Number of active pixels on the Dotstar LED strip.
// You may need to decrease this number if your power supply is not strong
// enough.
#define NUM_PIXELS 150
#define PIXEL_BUFFER_SIZE ((NUM_PIXELS*4) + 8)

// Global buffer which holds all of the pixels.
static char pixels[PIXEL_BUFFER_SIZE];

// Needed for the spi_read_write call; unused.
static char rbuf[PIXEL_BUFFER_SIZE];

// Dotstar strips expect the pixel colors to be set blue first, green second,
// and red third.
static const uint32_t BLUE_OFFSET = 1;
static const uint32_t GREEN_OFFSET = 2;
static const uint32_t RED_OFFSET = 3;

// Constants used when converting between a Color and its RGB components.
#define COLOR_SHIFT(x) ((8 * (3 - x)))
#define RED_SHIFT COLOR_SHIFT(RED_OFFSET)
#define BLUE_SHIFT COLOR_SHIFT(BLUE_OFFSET)
#define GREEN_SHIFT COLOR_SHIFT(GREEN_OFFSET)

// Constants used when getting and setting pixel values.
#define PIXEL_INDEX(i, c) (((i * 4) + 4 + c))
#define RED_INDEX(i) (PIXEL_INDEX(i, RED_OFFSET))
#define BLUE_INDEX(i) (PIXEL_INDEX(i, BLUE_OFFSET))
#define GREEN_INDEX(i) (PIXEL_INDEX(i, GREEN_OFFSET))

// A Color is a 'packed' representation of its RGB components.
typedef uint32_t Color;

/**
 * Extract the color components from a Color.
 * @{
 */
static uint8_t red(Color color) {
    return (uint8_t)((color >> RED_SHIFT) & 0xFF);
}

static uint8_t green(Color color) {
    return (uint8_t)((color >> GREEN_SHIFT) & 0xFF);
}

static uint8_t blue(Color color) {
    return (uint8_t)((color >> BLUE_SHIFT) & 0xFF);
}
/**
 * @}
 */

/**
 * Create a Color from its individual components.
 */
static Color color(uint8_t r, uint8_t g, uint8_t b) {
    return (r << RED_SHIFT) | (g << GREEN_SHIFT) | (b << BLUE_SHIFT);
}

/**
 * Calculate a color along a color wheel, which cycles from red to blue to green
 * and back.
 *
 * @param wheelpos A value from 0-255 representing the position along the color
 *                 wheel.
 * @returns The Color corresponding to the given position in the color wheel.
 */
static Color wheel(uint8_t wheelpos) {
    wheelpos = 255 - wheelpos;
    if (wheelpos < 85) return color(255 - wheelpos * 3, 0, wheelpos * 3);

    if (wheelpos < 170) {
        wheelpos -= 85;
        return color(0, wheelpos*3, 255 - wheelpos * 3);
    }

    wheelpos -= 170;
    return color(wheelpos*3, 255 - wheelpos * 3, 0);
}

/**
 * Callback for the SPI write operation in update_strip().
 */
static void write_cb(__attribute__ ((unused)) int arg0,
                     __attribute__ ((unused)) int arg2,
                     __attribute__ ((unused)) int arg3,
                     __attribute__ ((unused)) void* userdata) {
}

/**
 * Set the color of the pixel at the given index.
 *
 * This does not actually update the LED strip, but instead just modifies the
 * pixel's color in the pixel buffer. To display the new color on the strip, you
 * must call update_strip().
 */
static void set_pixel(uint32_t pixel, Color color) {
    pixels[RED_INDEX(pixel)] = red(color);
    pixels[GREEN_INDEX(pixel)] = green(color);
    pixels[BLUE_INDEX(pixel)] = blue(color);
}

/**
 * Get the color of the pixel at the given index.
 *
 * This does not return the pixel color as it is currently displayed on the
 * strip, but rather the color in the pixel buffer that has been set by set_pixel().
 */
static Color __attribute__((unused)) get_pixel(uint32_t pixel) {
    return color(pixels[RED_INDEX(pixel)],
                 pixels[GREEN_INDEX(pixel)],
                 pixels[BLUE_INDEX(pixel)]);
}

/**
 * Initialize the SPI and the pixel buffer in order to use the LED strip.
 *
 * This function must be called once before ever calling update_strip().
 * All pixels in the buffer will be initially set to zero (off).
 */
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

    for (i = 0; i < NUM_PIXELS; i++) {
        set_pixel(i, 0);
    }
}

/**
 * Write all pixel values to the Dotstar LED strip.
 *
 * This function is what actually displays the new pixel colors onto the strip.
 * Before using this function, you must first have called initialize_strip().
 */
static void update_strip(void) {
    spi_read_write(pixels, rbuf, PIXEL_BUFFER_SIZE, write_cb, NULL);
}

/**
 * Simple animation which cycles two simultaneous color wheels on the Dotstar LED strip.
 */
int main(void) {
    // Needed before calling update_strip(). All pixels in the buffer are
    // initialized to zero.
    initialize_strip();

    // Position in the forward color wheel.
    uint8_t wf = 0;

    // Position in the backward color wheel.
    uint8_t wb = 255;

    while (1) {
        // Set the forward color wheel pixels.
        int i;
        for (i = 0; i < NUM_PIXELS; i+=4) {
            set_pixel(i, wheel(wf+(i/4)));
        }

        // Set the backward color wheel pixels.
        for (i = 2; i < NUM_PIXELS; i+=4) {
            set_pixel(i, wheel(wb-(i/4)));
        }

        // Advance along the two wheels at different rates to make the pattern
        // more interesting.
        wf++;
        wb-=2;

        // Display the new pixel values.
        update_strip();

        // Wait 20 milliseconds before displaying the next update.
        delay_ms(20);
    }
}
