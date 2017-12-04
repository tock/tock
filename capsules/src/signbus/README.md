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

Usage
-----


```rust
// Signbus virtual alarm
    let signbus_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, sam4l::ast::Ast>,
        VirtualMuxAlarm::new(mux_alarm));

    // Signbus port_layer
    let port_layer = static_init!(
        capsules::signbus::port_layer::SignbusPortLayer<'static,
        VirtualMuxAlarm<'static, sam4l::ast::Ast>>,
        capsules::signbus::port_layer::SignbusPortLayer::new(
            &sam4l::i2c::I2C1,
            &mut capsules::signbus::port_layer::I2C_SEND,
            &mut capsules::signbus::port_layer::I2C_RECV,
            &sam4l::gpio::PB[14], // D0 mod_in
            &sam4l::gpio::PB[15], // D1 mod_out
            signbus_virtual_alarm,
            Some(&sam4l::gpio::PA[13]), // RED LED
		));

    sam4l::i2c::I2C1.set_master_client(port_layer);
    sam4l::i2c::I2C1.set_slave_client(port_layer);
    signbus_virtual_alarm.set_client(port_layer);
    sam4l::gpio::PB[14].set_client(port_layer);


    // Signbus IO Interface
    let io_layer = static_init!(
        capsules::signbus::io_layer::SignbusIOLayer<'static>,
        capsules::signbus::io_layer::SignbusIOLayer::new(port_layer,
              	&mut capsules::signbus::io_layer::BUFFER0,
                &mut capsules::signbus::io_layer::BUFFER1,
                &mut capsules::signbus::io_layer::BUFFER2
     ));

    port_layer.set_io_client(io_layer);

    // Signbus Protocol Layer
    let protocol_layer = static_init!(
        capsules::signbus::protocol_layer::SignbusProtocolLayer<'static>,
        capsules::signbus::protocol_layer::SignbusProtocolLayer::new(io_layer,
    ));

    io_layer.set_client(protocol_layer);

    // Signbus App Layer
    let app_layer = static_init!(
        capsules::signbus::app_layer::SignbusAppLayer<'static>,
        capsules::signbus::app_layer::SignbusAppLayer::new(protocol_layer,
            &mut capsules::signbus::app_layer::BUFFER0,
              &mut capsules::signbus::app_layer::BUFFER1
    ));

protocol_layer.set_client(app_layer);
```
