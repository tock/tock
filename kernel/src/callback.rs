//! Data structure for storing a callback to userspace or kernelspace.

use core::fmt;
use core::ptr::NonNull;

use crate::capabilities;
use crate::config;
use crate::debug;
use crate::process;
use crate::sched::Kernel;

/// Userspace app identifier.
///
/// This should be treated as an opaque type that can be used to represent an
/// application on the board without requiring an actual reference to a
/// `ProcessType` object. Having this `AppId` reference type is useful for
/// managing ownership and type issues in Rust, but more importantly `AppId`
/// serves as a tool for capsules to hold pointers to applications.
///
/// Since `AppId` implements `Copy`, having an `AppId` does _not_ ensure that
/// the process the `AppId` refers to is still valid. The process may have been
/// removed, terminated, or restarted as a new process. Therefore, all uses of
/// `AppId` in the kernel must check that the `AppId` is still valid. This check
/// happens automatically when `.index()` is called, as noted by the return
/// type: `Option<usize>`. `.index()` will return the index of the process in
/// the processes array, but if the process no longer exists then `None` is
/// returned.
///
/// Outside of the kernel crate, holders of an `AppId` may want to use `.id()`
/// to retrieve a simple identifier for the process that can be communicated
/// over a UART bus or syscall interface. This call is guaranteed to return a
/// suitable identifier for the `AppId`, but does not check that the
/// corresponding application still exists.
///
/// This type also provides capsules an interface for interacting with processes
/// since they otherwise would have no reference to a `ProcessType`. Very limited
/// operations are available through this interface since capsules should not
/// need to know the details of any given process. However, certain information
/// makes certain capsules possible to implement. For example, capsules can use
/// the `get_editable_flash_range()` function so they can safely allow an app
/// to modify its own flash.
#[derive(Clone, Copy)]
pub struct AppId {
    /// Reference to the main kernel struct. This is needed for checking on
    /// certain properties of the referred app (like its editable bounds), but
    /// also for checking that the index is valid.
    pub(crate) kernel: &'static Kernel,

    /// The index in the kernel.PROCESSES[] array where this app's state is
    /// stored. This makes for fast lookup of the process and helps with
    /// implementing IPC.
    ///
    /// This value is crate visible to enable optimizations in sched.rs. Other
    /// users should call `.index()` instead.
    pub(crate) index: usize,

    /// The unique identifier for this process. This can be used to refer to the
    /// process in situations where a single number is required, for instance
    /// when referring to specific applications across the syscall interface.
    ///
    /// The combination of (index, identifier) is used to check if the app this
    /// `AppId` refers to is still valid. If the stored identifier in the
    /// process at the given index does not match the value saved here, then the
    /// process moved or otherwise ended, and this `AppId` is no longer valid.
    identifier: usize,
}

impl PartialEq for AppId {
    fn eq(&self, other: &AppId) -> bool {
        self.identifier == other.identifier
    }
}

impl Eq for AppId {}

impl fmt::Debug for AppId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.identifier)
    }
}

impl AppId {
    /// Create a new `AppId` object based on the app identifier and its index
    /// in the processes array.
    pub(crate) fn new(kernel: &'static Kernel, identifier: usize, index: usize) -> AppId {
        AppId {
            kernel: kernel,
            identifier: identifier,
            index: index,
        }
    }

    /// Create a new `AppId` object based on the app identifier and its index
    /// in the processes array.
    ///
    /// This constructor is public but protected with a capability so that
    /// external implementations of `ProcessType` can use it.
    pub fn new_external(
        kernel: &'static Kernel,
        identifier: usize,
        index: usize,
        _capability: &dyn capabilities::ExternalProcessCapability,
    ) -> AppId {
        AppId {
            kernel: kernel,
            identifier: identifier,
            index: index,
        }
    }

    /// Get the location of this app in the processes array.
    ///
    /// This will return `Some(index)` if the identifier stored in this `AppId`
    /// matches the app saved at the known index. If the identifier does not
    /// match then `None` will be returned.
    pub(crate) fn index(&self) -> Option<usize> {
        // Do a lookup to make sure that the index we have is correct.
        if self.kernel.appid_is_valid(self) {
            Some(self.index)
        } else {
            None
        }
    }

    /// Get a `usize` unique identifier for the app this `AppId` refers to.
    ///
    /// This function should not generally be used, instead code should just use
    /// the `AppId` object itself to refer to various apps on the system.
    /// However, getting just a `usize` identifier is particularly useful when
    /// referring to a specific app with things outside of the kernel, say for
    /// userspace (e.g. IPC) or tockloader (e.g. for debugging) where a concrete
    /// number is required.
    ///
    /// Note, this will always return the saved unique identifier for the app
    /// originally referred to, even if that app no longer exists. For example,
    /// the app may have restarted, or may have been ended or removed by the
    /// kernel. Therefore, calling `id()` is _not_ a valid way to check
    /// that an application still exists.
    pub fn id(&self) -> usize {
        self.identifier
    }

    /// Returns the full address of the start and end of the flash region that
    /// the app owns and can write to. This includes the app's code and data and
    /// any padding at the end of the app. It does not include the TBF header,
    /// or any space that the kernel is using for any potential bookkeeping.
    pub fn get_editable_flash_range(&self) -> (usize, usize) {
        self.kernel.process_map_or((0, 0), *self, |process| {
            let start = process.flash_non_protected_start() as usize;
            let end = process.flash_end() as usize;
            (start, end)
        })
    }
}

/// Type to uniquely identify a callback subscription across all drivers.
///
/// This contains the driver number and the subscribe number within the driver.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CallbackId {
    pub driver_num: u32,
    pub subscribe_num: u32,
}

/// A callback to userspace
///
/// This is essentially a wrapper around a function pointer with
/// additional data.
///
/// In contrast to the contained `ProcessCallback`, this type does
/// not actually have to point to a userspace process. This is the
/// case in the [default instances](Callback::default).
///
/// A default instance will never schedule an actual callback to
/// userspace.
pub struct Callback(Option<ProcessCallback>);

impl Callback {
    /// Construct a new [`Callback`] pointing to a process
    ///
    /// This will create a [`ProcessCallback`] internally, hence
    /// pointing to a userspace process, in constrast to the [default
    /// instances](<Callback as Default::default).
    ///
    /// The passed pointer may be `None`, in which case the Callback
    /// will not actually be scheduled (creating a _null callback_).
    pub(crate) fn new(
        app_id: AppId,
        callback_id: CallbackId,
        appdata: usize,
        fn_ptr: Option<NonNull<()>>,
    ) -> Self {
        Callback(Some(ProcessCallback::new(
            app_id,
            callback_id,
            appdata,
            fn_ptr,
        )))
    }

    /// Construct a new default [`Callback`]
    ///
    /// A default [`Callback`] instance will not point to any actual
    /// userspace process. Therefore, no actual callbacks will be
    /// scheduled by this instance.
    pub const fn const_default() -> Self {
        Callback(None)
    }

    /// Get the contained [`ProcessCallback`] struct
    ///
    /// If the [`Callback`] refers to a process, this returns the
    /// contained [`ProcessCallback`] struct.
    pub(crate) fn into_inner(self) -> Option<ProcessCallback> {
        self.0
    }

    /// Attempt to trigger the callback.
    ///
    /// If this callback refers to an actual function pointer (not a
    /// _null callback_) and the process is valid, this will queue the
    /// [`Callback`] for the associated process.
    ///
    /// ## Arguments
    /// The arguments (`r0-r2`) are the values passed back to the
    /// process and are specific to the individual `Driver`
    /// interfaces.
    ///
    /// ## Return value
    /// The following conditions are checked in this order to
    /// determine the return value:
    ///
    /// 1. If this [`Callback`] is a default instance (does not refer
    ///    to a process), it returns `true` and behaves as if the
    ///    callback was scheduled.
    ///
    /// 2. This function returns `false` if the process is no longer
    ///    active.
    ///
    /// 3. For a _null callback_, this function will return `true` and
    ///    behave as if the callback was scheduled.
    ///
    /// 4. This function returns `false` if the queue for the process
    ///    is full and the callback could not be scheduled.
    ///
    /// Otherwise, this function returns `true`.
    pub fn schedule(&mut self, r0: usize, r1: usize, r2: usize) -> bool {
        self.0.as_mut().map_or(true, |cb| cb.schedule(r0, r1, r2))
    }
}

impl Default for Callback {
    /// Construct a new default [`Callback`]
    ///
    /// A default [`Callback`] instance will not point to any actual
    /// userspace process. Therefore, no actual callbacks will be
    /// scheduled by this instance.
    fn default() -> Self {
        Callback::const_default()
    }
}

/// [`Callback`] referring to an actual process
///
/// This is essentially a wrapper around a function pointer with
/// additional data.
///
/// The [`ProcessCallback`], in contrast to [`Callback`], is always
/// explicity created by and for a specific process.
///
/// A [`ProcessCallback`] may be a _null callback_, not pointing to a
/// valid function. In this case, the callback won't actually be
/// called in userspace.
pub(crate) struct ProcessCallback {
    pub(crate) app_id: AppId,
    pub(crate) callback_id: CallbackId,
    pub(crate) appdata: usize,
    pub(crate) fn_ptr: Option<NonNull<()>>,
}

impl ProcessCallback {
    fn new(
        app_id: AppId,
        callback_id: CallbackId,
        appdata: usize,
        fn_ptr: Option<NonNull<()>>,
    ) -> ProcessCallback {
        ProcessCallback {
            app_id,
            callback_id,
            appdata,
            fn_ptr,
        }
    }

    /// Attempt to trigger the callback.
    ///
    /// If this callback refers to an actual function pointer (not a
    /// _null callback_) and the process is valid, this will queue the
    /// [`ProcessCallback`] for the associated process.
    ///
    /// ## Arguments
    /// The arguments (`r0-r2`) are the values passed back to the
    /// process and are specific to the individual `Driver`
    /// interfaces.
    ///
    /// ## Return value
    /// The following conditions are checked in this order to
    /// determine the return value:
    ///
    /// 1. This function returns `false` if the process is no longer
    ///    active.
    ///
    /// 2. For a _null callback_, this function will return `true` and
    ///    behave as if the callback was scheduled.
    ///
    /// 3. This function returns `false` if the queue for the process
    ///    is full and the callback could not be scheduled.
    ///
    /// Otherwise, this function returns `true`.
    fn schedule(&mut self, r0: usize, r1: usize, r2: usize) -> bool {
        self.app_id
            .kernel
            .process_map_or(false, self.app_id, |process| {
                if let Some(ptr) = self.fn_ptr {
                    // This is a process callback pointing to a valid
                    // address, try to schedule the callback
                    let task_res =
                        process.enqueue_task(process::Task::FunctionCall(process::FunctionCall {
                            source: process::FunctionCallSource::Driver(self.callback_id),
                            argument0: r0,
                            argument1: r1,
                            argument2: r2,
                            argument3: self.appdata,
                            pc: ptr.as_ptr() as usize,
                        }));

                    if config::CONFIG.trace_syscalls {
                        debug!(
                            "[{:?}] schedule[{:#x}:{}] @{:#x}({:#x}, {:#x}, {:#x}, {:#x}) = {}",
                            self.app_id,
                            self.callback_id.driver_num,
                            self.callback_id.subscribe_num,
                            ptr.as_ptr() as usize,
                            r0,
                            r1,
                            r2,
                            self.appdata,
                            task_res,
                        );
                    }

                    task_res
                } else {
                    // This is a null callback, behave as if the
                    // callback was scheduled
                    if config::CONFIG.trace_syscalls {
                        debug!(
                            "[{:?}] schedule[{:#x}:{}] @NULL({:#x}, {:#x}, {:#x}, {:#x}) (null-callback not scheduled!)",
                            self.app_id,
                            self.callback_id.driver_num,
                            self.callback_id.subscribe_num,
                            r0,
                            r1,
                            r2,
                            self.appdata,
                        );
                    }

                    true
                }
            })
    }
}
