#include <led.h>
#include <timer.h>
#include <stdio.h>

static char digits[] = "0000";

void update_display(char *d) {
    printf("%c%c%c%c", d[0], d[1], d[2], d[3]);
    fflush(stdout);
}

void reset_display() {
    printf("\x81");
    fflush(stdout);
}

int main(void) {
    reset_display();

    while (1) {
        // This delay uses an underlying timer in the kernel.
        delay_ms(50);

        update_display(digits);

        if (digits[3] == '9') {
            digits[3] = '0';
            if (digits[2] == '9') {
                digits[2] = '0';
                if (digits[1] == '9') {
                    digits[1] = '0';
                    if (digits[0] == '9') {
                        digits[0] = '0';
                    } else {
                        digits[0]++;
                    }
                } else {
                    digits[1]++;
                }
            } else {
                digits[2]++;
            }
        } else {
            digits[3]++;
        }
    }
}
