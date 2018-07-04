Since 1.2
=========

  * [#1044](https://github.com/tock/tock/pull/1044) creates a `Kernel` struct
    with a method for the kernel's main loop, instead of a global function in
    the kernel's base module. Board configurations (i.e. each board's
    `main.rs`), as a result needs to instantiate a statically allocate this new
    struct.  Arguments to the main loop haven't changed:

    ```rust
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new());

    board_kernel.kernel_loop(&hail, &mut chip, &mut PROCESSES, Some(&hail.ipc));
    ```

