# Signbus Communication Layers

```
USERLAND
 ↳ app_layer
  ↳ protocol_layer
   ↳ io_layer
    ↳ port_layer
     ↳ I2C DRIVER
```

## app_layer
Userland buffers and callbacks.
Concatenate app information (frame_type, api_type) to message.
 
    pub trait AppLayerClient {
      fn packet_received();
      fn packet_sent();
      fn packet_read_from_slave();
    }

    SignbusAppLayer {
      fn signbus_app_send();
      fn signbus_app_recv();
    }

## protocol_layer
Encrypt/ decrypt message and concatenate HMAC to message.
*Not implemented.*

    pub trait ProtocolLayerClient {
      fn packet_received();
      fn packet_sent();
      fn packet_read_from_slave();
    }

    SignbusProtocolLayer {
      fn signbus_protocol_send();
      fn signbus_protocol_recv();
    }


## io_layer
Send/ receive Signbus packets and concatenate fragmented messages together.

    pub trait IOLayerClient {
      fn packet_received();
      fn packet_sent();
      fn packet_read_from_slave();
    }

    SignbusIOLayer {
      fn signbus_io_init();
      fn signbus_io_send();
      fn signbus_io_recv();
    }


## port_layer
Send/ receive I2C MTU (255 bytes). Communicates with I2C driver.
Ability to use gpio and timer.

    pub trait PortLayerClientI2C {
      fn packet_received();
      fn packet_sent();
      fn packet_read_from_slave();
    }

    pub trait PortLayerClientGPIOTimer {
      fn mod_in_interrupt();
      fn delay_complete();
    }

    SignbusPortLayer {
      fn init();
      fn i2c_master_write();
      fn i2c_slave_listen();
      fn i2c_slave_read_setup();
      fn mod_out_set();
      fn mod_out_clear();
      fn mod_in_read();
      fn mod_in_enable_interrupt();
      fn mod_in_disable_interrupt();
      fn delay_ms();
      fn debug_led_on();
      fn debug_led_off();
    }
