OTA app
========
This document explains how `ota app` code works in microbit_v2 and provides a guide to write an application with `ota app`. In addition, it describes the design overview.

<!-- npm i -g markdown-toc; markdown-toc -i ota_app_system_document.md -->
<!-- toc -->

- [Designe overview of ota app](#design-overview-of-ota-app)
  *[Update Scenario](#update-scenario)
  *[Module Dependency](#module-dependency)
  *[Key points](#key points)
  *[State Machine](#state machine)
- [Guide for demo](#guide-for-demo)
- [To do list](#to-do-list)

<!-- tocstop -->

## Designe overview of ota app

OTA (On The Air) app project starts with the aim to make OTA as a general standard independent to specific operating systems. As IoT devices (e.g., smart watch, smart home appliance, smart farm, autonomous driving, smart blending) are getting increased, we need to consider "How to manage tons of its devices in terms of cost, maintenance, and security".

IoT industry has become time-to-market. This property makes 3rd parties difficult to build their reliable IoT device. Thus, they choose to update the software after launching their IoT device. Normally, there are two ways to update IoT devices. First is to update the device manually. In this case, Flash Boot Loader will be in charge of flashing software. But this manual update is time-consuming and it is hard to track all of the update history of IoT devices. Second is to update the device by OTA. Since the software update by OTA is executed from a web server wirelessly, it is convenient to update their software, to add new features, and to improving security issues. However, 3rd parties adopt their own OTA policy as well as a specific operating system. Such diversity causes that some IoT devices holds brand new features and up-to-date security, whereas other devices stay old version software and vulnerable security in the fully connected IoT device network.

If we can standardize OTA and entrench it in IoT industry, we can overcome such problems. To do this, I choose to implement OTA at the application layer. There are two reasons. First, most of devices adopt operating systems instead of bare metal code, and modern operating systems pursue POSIX system. If an OTA app standardizes APIs to update software, that APIs can be built in the POSIX-based operating system (e.g., Linux), then this standard can proliferate to other operating systems. It means that OTA can be general and independent to a specific operating system. Second, If OTA is implemented in the application layer, developers don't need to be limited to a specific programming language, because modern operating systems can run applications programmed with diverse programming language. Thus, regardless of programming language, if programmers follow OTA policy and use APIs provided by the operating system, they can easily implement OTA.

### Update Scenario

[2022-07-22] OTA app proivdes updating a new application (not driver component) at runtime currently. Since flashing applications should follow `MPU alignment rule`, it is only possible to update an application which has the size smaller or equal than the size of OTA app (8198 bytes). If an application which don't follow `MPU alignment rule` is flahsed, the loaded application will be erased. Furtuermore, if the number of application loaded on the target board reaches to the maximum number of application that the target board can run, OTA app doesn't execute update.

### Module Dependency
[2022-07-22] The following image describes `ota app` module dependency implemented on Tock. We assume that `ota_uart.send.py` act as web server which send data to IoT device. From the external tool to `console.rs`, binary data move through `①`. When receiving the specified size of data (517), console callback function `②` is triggered. Then `ota_app` parses the receiving data and do actions according to a command which is positioned at index 0 of the data. After completing actions corresponding commands, `ota_app` send `ota_uart_send.py` a coressponding response for next sequence. An application (.tbf) binary is written to flash memory via `③`. When writing the binary is done, `ota_send_uart.py` delivers crc32 value which it sends to `ota_app`, and `ota_app` also calculates the data that it received. Then the app request `process_load_utilities.rs` to calculate the written binary data into flash memory, and return the resulting value to `ota-app` via `④ and ⑤`. `ota_app` checks whether or not the three crc32 values are same. If there is incorrect crc32 consistency, `ota_app` erases the loaded data. When the update procedure passes the crc32 consistency state, `ota_app` requests loading the loaded application through ⑥. If the loaded app doesn't follow `MPU alignment rule`, `ota_app` erases the loaded data and don't load the entry point of the loaded application into `PROCESS global array` at main.rs.
  

![ota app module dependency]](ota_app_module_dependency.png)

### Key points
 
[2022-07-22] Dynamically changing start address of flash memory and sram.
When we update an application at runtime, we don't have to interfere flash and sram memory region which is occupied by the kernel and other apps. If we commit such memory access violation, the system is going to be crushed. To prevent this issue, there are three key variables that save the dynamically changing unused flash memory and sram memory start address at "process_load_utilities.rs". 

At `find_dynamic_start_address_of_writable_flash_advanced`, it parses an address immediately next to the last application in flash memory and an index used to save the entry point of the loaded application into PROCESS global array. Then it saves the address and index to `dynamic_flash_start_addr` and `index` at grant region separately.


```c
struct ProcLoaderData{
    index: usize,
    dynamic_flash_start_addr: usize,
    dynamic_unsued_sram_start_addr: usize,
}
```

```c
fn find_dynamic_start_address_of_writable_flash_advanced(
        &self,
        proc_data: &mut ProcLoaderData,
    ) -> Result<(), ProcessLoadError>
```

The most tricky part is to find out `unused sram start address`. Since we load applications by using tockloader and then the system executes reset, we can figure out what is `the unused sram start address` by receiving it from `kernel::process::load_processes` at main.rs. This returned address is saved to `dynamic_unsued_sram_start_addr` as the initial value, when OTA app calls a command at init stage (Only one-time). After parsing `dynamic_flash_start_addr`, `index`, and `dynamic_unsued_sram_start_addr`, when we first attempt to load a new application by `ota app` at `load_processes_advanced_air`, such three variables are used. After loading the new application. If there is no loading error caused by alignment, the entry point of the loaded app is saved to PROCESS global array, and `dynamic_unsued_sram_start_addr` is updated to next unused sram start address. 

```c
 fn load_processes_advanced_air(
        &self,
        proc_data: &mut ProcLoaderData,
    ) -> Result<(usize, Option<&'static dyn Process>), ProcessLoadError>
```

### State Machine
[2022-07-22] `ota_app` follows the below state machine.
0) [Init stage]
    - Init stage is executed in main function only one time. In this stage, constant value (e.g., app start address, Rom end address, the number of supported process) are saved. 

When receving commands, the below state machine is executed.
1) [COMMAND_SET_INIT]
    - The size of an app which will be loaded are saved.
    - Request to find dynamically changing flash start address, and get the address.
    - Get an index to write the entry point of the app
    - Check whether or not the index is greater or equal than 4 (the number of supported process) and there is enough flash region to write the app
    
2) [COMMAND_WRITE_BINARY_DATA]
    - Write the app binary into flash memory (512 bytes)
    - Repeat writing the binary
    
3) [COMMAND_WRITE_PADDING_DATA]
    - Write 01, 01, 01.. (512 bytes) padding data in order to make boundaries between apps.
    
4) [COMMAND_SEND_CRC]
    - Check whether or not three CRC32 values are same. If not, send the external tool fail response. Then, the loaded app will be erased. 
    
5) [COMMAND_APP_LOAD ]
    - Request loading the entry point of the loaded app. If the flashed app doesn't meet `MPU alignment rule`, `ota app` sends the external tool fail response. Then, the loaded app will be erased. 
    
6) [COMMAND_APP_ERASE]
    - When receiving the erase request, it erases the loaded app.

## Guide for demo
[2022-07-22] In the directory(tock/tool/ota_app), there is `ota_uart_send.py` tool and a couple of test tbf files. After copying, and merging OTA app project code into your local work folder. Then, it is necessary to disable the below code snippet at main.rs, because undesired strings (e.g., $tock) interrupt the communication protocol between the tool and ota_app. After compiling the kernel code and loading it, do run the python tool by entering `python ota_uart_send.py [file name]`. Then you will see the update by `ota app`.

```c
let process_printer =
        components::process_printer::ProcessPrinterTextComponent::new().finalize(());
    PROCESS_PRINTER = Some(process_printer);

    let _process_console = components::process_console::ProcessConsoleComponent::new(
        board_kernel,
        uart_mux,
        mux_alarm,
        process_printer,
    )
    .finalize(components::process_console_component_helper!(
        nrf52833::rtc::Rtc
    ));
    let _ = _process_console.start();
```

## To do list
1) Adding security features (i.e., system call filter, permission header)
2) Need to come up with an idea to meet the `MPU alignment rule`
3) Document dynamic view of `ota app`
4) Erase function and etc..



