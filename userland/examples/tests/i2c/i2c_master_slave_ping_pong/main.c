#include <stdbool.h>

#include <button.h>
#include <i2c_master_slave.h>
#include <timer.h>

#define BUF_SIZE 16
#define LEADER_ADDRESS 0x40
#define FOLLOW_ADDRESS 0x41

uint8_t master_write_buf[BUF_SIZE];
uint8_t master_read_buf[BUF_SIZE];
uint8_t slave_write_buf[BUF_SIZE];
uint8_t slave_read_buf[BUF_SIZE];
bool    is_leader = false;


// In response to a
static void i2c_callback(int callback_type,
              __attribute__ ((unused)) int length,
              __attribute__ ((unused)) int arg2,
              __attribute__ ((unused)) void* userdata) {
  if (callback_type == TOCK_I2C_CB_MASTER_WRITE) {
    printf("CB: Master write\n");
    i2c_master_slave_listen();
  } else if (callback_type == TOCK_I2C_CB_SLAVE_WRITE) {
    printf("CB: Slave write\n");
    delay_ms(2500);

    printf("%s sending\n", is_leader ? "Leader" : "Follower");
    i2c_master_slave_write(is_leader ? FOLLOW_ADDRESS : LEADER_ADDRESS, BUF_SIZE);
  } else {
    printf("ERROR: Unexepected callback: type %d\n", callback_type);
  }
}

// This is the callback for the button press.
// A button press indicates that this device should start the ping-pong
// exchange. First, change the address to the BUTTON_ADDRESS to avoid
// conflict with the other node, then send a message.
static void button_cb(__attribute__((unused)) int btn_num,
              __attribute__ ((unused)) int arg1,
              __attribute__ ((unused)) int arg2,
              __attribute__ ((unused)) void* userdata) {
  // Only the first press is meaningfull
  static bool pressed = false;

  if (!pressed) {
    pressed = true;
    is_leader = true;

    printf("Switching to master\n");

    i2c_master_slave_set_slave_address(LEADER_ADDRESS);
    i2c_master_slave_write(FOLLOW_ADDRESS, BUF_SIZE);
  }
}

// This function sets up the I2C peripheral with needed buffers and prepares
// callbacks for I2C and button presses. Normal operation of this test takes
// place in the subsequent callbacks.
int main(void) {
  printf("I2C Master/Slave Ping-Pong\n");

  // Prepare buffers
  strcpy((char*) master_write_buf, "Hello friend.\n");

  // Set up I2C peripheral
  i2c_master_slave_set_callback(i2c_callback, NULL);
  i2c_master_slave_set_master_write_buffer(master_write_buf, BUF_SIZE);
  i2c_master_slave_set_master_read_buffer(master_read_buf, BUF_SIZE);
  i2c_master_slave_set_slave_write_buffer(slave_write_buf, BUF_SIZE);
  i2c_master_slave_set_slave_read_buffer(slave_read_buf, BUF_SIZE);

  i2c_master_slave_set_slave_address(FOLLOW_ADDRESS);
  i2c_master_slave_listen();

  // Set up button peripheral to grab any button press
  button_subscribe(button_cb, NULL);

  int nbuttons = button_count();
  int j;
  for (j = 0; j < nbuttons; j++) {
    button_enable_interrupt(j);
  }
}
