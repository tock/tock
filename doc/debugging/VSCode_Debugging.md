VSCode Debugging
========

This is a guide how to perform remote debugging via JTAG in Tock using VSCode (at the moment nRF51-DK and nRF52-DK are supported)

## Requirements
1. [VSCode](https://code.visualstudio.com)
2. [VSCode Native Debug Extension](https://github.com/WebFreak001/code-debug)
3. [VSCode Rust Extension](https://github.com/editor-rs/vscode-rust)

## Installation
1. Install VSCode for your platform
2. Open VSCode
3. Enter the extensions menu by pressing `View/Extensions` 

4. Install `Native Debug` and `Rust` in that view by searching for them

You are now good to run the debugger and the debugging configurations is already configured for you.
But, if you want change the configuration for example to run some special gdb 
commands before starting you can do that [here](../../.vscode/launch.json). 

## Enabling breakpoints
Let's now test if this works by configuring some breakpoints:

1. Enter `Explorer mode` by pressing `View/Explorer`
2. Browse and open a file where you want to enable a breakpoint
3. In my case I want to have a breakpoint in the `reset_handler` in main.rs
4. Click left of the line number to enable a breakpoint. You should see red dot now as the figure below:
 
![Enable breakpoint VSCode](figures/vsc_breakpoint.png) 

## Running the debugger
1. You need to start the `GDB Server` before launching a debugging session in VSCode (check out the instructions how to do that for your board).
2. Enter `Debug mode` in VSCode by pressing `View/Debug`. You should now see a debug view somewhere on your screen as the figure below:
 
![VSCode Debug mode](figures/vsc_debug_view.png)

3. Choose your board in the scroll bar and then click on green arrow or `Debug/Start Debugging`  
4. You should now see that program stopped at the breakpoint as the figure below:
![Running](figures/vsc_running.png)
5. Finally, if want to use specific gdb commands you can use the debug console in VSCode which is very useful.


## Issues
1. Sometimes gdb behaves unpredictably and stops at the wrong source line. For example sometimes we have noticed that debugger stops at `/kernel/src/support/arm.rs` instead of the `reset_handler` if that occurs just press `step over` and it should hopefully jump to correct location.
2. Rust in `release mode` is optimizing things such as inlining and mangling which makes debugging harder and values may not be visible. To perform more reliable debugging mark the important functions with:
```
#[no_mangle]
#[inline(never)]
```
3. Enable `rust-pretty printer` or something similar because viewing variables are very limited in VSCode
