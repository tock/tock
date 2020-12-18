use core::fmt::Write;

use kernel;
use kernel::syscall::ContextSwitchReason;

use crate::process::UnixProcess;
use crate::syscall_transport::SyscallTransport;
use crate::Result;

use crate::ipc_syscalls as ipc;

use std::path::PathBuf;

#[derive(Default, Copy, Clone)]
pub struct HostStoredState {
    process: Option<&'static UnixProcess<'static>>,
    syscall_ret: ipc::KernelReturn,
}

#[derive(Default)]
pub struct SysCall {
    transport: SyscallTransport,
}

impl HostStoredState {
    pub fn new(process: &'static UnixProcess<'static>) -> Self {
        HostStoredState {
            process: Some(process),
            ..Default::default()
        }
    }
}

impl SysCall {
    pub fn try_new(syscall_rx: PathBuf, syscall_tx: PathBuf) -> Result<SysCall> {
        Ok(SysCall {
            transport: SyscallTransport::open(syscall_tx, syscall_rx)?,
        })
    }

    pub fn get_transport(&self) -> &SyscallTransport {
        &self.transport
    }
}

impl kernel::syscall::UserspaceKernelBoundary for SysCall {
    type StoredState = HostStoredState;

    unsafe fn initialize_process(
        &self,
        stack_pointer: *const usize,
        _stack_size: usize,
        _state: &mut Self::StoredState,
    ) -> core::result::Result<*const usize, ()> {
        // Do nothing as Unix process will be started on first switch_to_process
        // This is good place for synchronize libtock-rs startup
        // Right now this is not needed b/c libtock-rs not require
        // anything special right now
        Ok(stack_pointer as *mut usize)
    }

    unsafe fn set_syscall_return_value(
        &self,
        _stack_pointer: *const usize,
        state: &mut Self::StoredState,
        return_value: isize,
    ) {
        state.syscall_ret = ipc::KernelReturn::new_ret(return_value);
    }

    unsafe fn set_process_function(
        &self,
        stack_pointer: *const usize,
        _remaining_stack_memory: usize,
        state: &mut Self::StoredState,
        callback: kernel::procs::FunctionCall,
    ) -> core::result::Result<*mut usize, *mut usize> {
        // Do nothing as app 'Unix process' will be started on first switch_to_process
        state.syscall_ret = ipc::KernelReturn::new_cb(ipc::Callback::new(
            callback.pc,
            callback.argument0,
            callback.argument1,
            callback.argument2,
            callback.argument3,
        ));

        Ok(stack_pointer as *mut usize)
    }

    unsafe fn switch_to_process(
        &self,
        stack_pointer: *const usize,
        state: &mut Self::StoredState,
    ) -> (*mut usize, ContextSwitchReason) {
        let process = match state.process {
            Some(p) => p,
            None => return (stack_pointer as *mut usize, ContextSwitchReason::Fault),
        };
        let return_value: Option<ipc::KernelReturn>;
        let transport = self.get_transport();

        if !process.was_started() {
            if let Err(e) = process.start(transport.rx_path(), transport.tx_path()) {
                panic!("KERN: Failed to start process {}", e);
            }
            transport.wait_for_connection();
            return_value = None;
        } else {
            return_value = Some(state.syscall_ret);
        }

        if let Some(ret) = return_value {
            transport.send_msg(process.get_id(), &ret);
            process.send_allows(transport);
        }
        let emulated_syscall: ipc::Syscall = transport.recv_msg();

        let syscall = kernel::syscall::arguments_to_syscall(
            emulated_syscall.syscall_number as u8,
            emulated_syscall.args[0],
            emulated_syscall.args[1],
            emulated_syscall.args[2],
            emulated_syscall.args[3],
        );

        let ret = match syscall {
            Some(mut s) => {
                // Due to memory address translation between app and kernel
                // handle ALLOW here. In received syscall there is pointer
                // to guest address space, we have to create host one
                if let kernel::syscall::Syscall::ALLOW {
                    driver_number,
                    subdriver_number,
                    mut allow_address,
                    allow_size,
                } = s
                {
                    // Receive AllowInfo and slice of guest memory and map it to host
                    // using allow_map
                    allow_address = process.recv_allow_data(allow_address as *const u8, transport);

                    s = kernel::syscall::Syscall::ALLOW {
                        driver_number: driver_number,
                        subdriver_number: subdriver_number,
                        allow_address: allow_address,
                        allow_size: allow_size,
                    };
                }
                ContextSwitchReason::SyscallFired { syscall: s }
            }
            None => ContextSwitchReason::Fault,
        };
        return (stack_pointer as *mut usize, ret);
    }

    unsafe fn print_context(
        &self,
        _stack_pointer: *const usize,
        _state: &Self::StoredState,
        _writer: &mut dyn Write,
    ) {
    }
}
